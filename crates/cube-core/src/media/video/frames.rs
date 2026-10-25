//! Independent baseline-JPEG samples; reuse the existing bounded image decoder.
use super::*;
use crate::{
    hashing,
    image::Image,
    operations::{Parameter, ParameterType},
};
pub const MAX_FRAMES: usize = 8;
pub const MAX_FRAME_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_TOTAL_DECODED: usize = 64 * 1024 * 1024;
const FRAME_OP: &str = "video.frames.extract";
#[derive(Clone, Debug, Serialize)]
pub struct Provenance {
    pub track_id: u64,
    pub frame_index: usize,
    pub decode_start_ticks: u64,
    pub duration_ticks: u32,
    pub time_base: TimeBase,
    pub offset: u64,
    pub length: u32,
    pub encoded_sha256: String,
    pub representation: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct Frame {
    pub provenance: Provenance,
    #[serde(skip)]
    pub image: Image,
}
/// Select 1..=8 unique ordered zero-based frame indexes from one unfragmented
/// MOV/ISO-BMFF `jpeg` track with self-contained version-zero descriptions.
/// Validate all sample tables, mdat bounds/nonoverlap and exact decode times first.
/// At most 10000 cumulative metadata/table records and 2 MiB reads, 10000 samples,
/// 2 MiB encoded/frame (16 MiB total), 16 MiB conservative decoded/frame and 64 MiB
/// total decoded samples. Only selected JPEGs are read/decoded; no timestamp seek.
/// Existing baseline JPEG framing/geometry/decoder caps apply. Pixels are decoded
/// lossy-source samples, without display matrix/orientation/color-profile changes.
pub fn extract(
    source: &mut (impl Read + Seek),
    track_id: u64,
    indexes: &[usize],
) -> Result<Vec<Frame>, OperationError> {
    let result = (|| {
        if track_id == 0 || track_id > u64::from(u32::MAX) {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "track_id must be a nonzero 32-bit ID",
            )
            .parameter("track_id"));
        }
        if indexes.is_empty()
            || indexes.len() > MAX_FRAMES
            || indexes
                .iter()
                .enumerate()
                .any(|(i, n)| *n >= MAX_RECORDS || indexes[..i].contains(n))
        {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "select 1..=8 unique frame indexes",
            )
            .parameter("indexes"));
        }
        let info = inspect(source)?;
        let index = mp4::index::jpeg(source, &info, track_id)?;
        let mut selected = Vec::with_capacity(indexes.len());
        let decoded_per = u64::from(index.width)
            .checked_mul(u64::from(index.height))
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| limit("decoded frame geometry overflows"))?;
        if decoded_per > crate::image::MAX_SAMPLES as u64
            || decoded_per * indexes.len() as u64 > MAX_TOTAL_DECODED as u64
        {
            return Err(limit("selected frames exceed decoded sample budgets"));
        }
        for &frame_index in indexes {
            let sample = index
                .samples
                .get(frame_index)
                .ok_or_else(|| invalid("frame index is out of bounds"))?;
            if sample.length as usize > MAX_FRAME_BYTES {
                return Err(limit("encoded frame exceeds 2 MiB"));
            }
            selected.push((frame_index, sample));
        }
        let mut encoded = Vec::with_capacity(selected.len());
        for &(_, sample) in &selected {
            source.seek(SeekFrom::Start(sample.offset))?;
            let mut data = vec![0; sample.length as usize];
            source
                .read_exact(&mut data)
                .map_err(|e| invalid("truncated selected JPEG sample").caused_by(e))?;
            if !data.starts_with(b"\xff\xd8") {
                return Err(invalid("selected sample is not JPEG"));
            }
            let boundary = forensics::boundaries::inspect(&mut std::io::Cursor::new(&data))?;
            if boundary.trailing_bytes != 0 {
                return Err(invalid(
                    "JPEG sample has trailing bytes outside its framing",
                ));
            }
            let geometry = crate::image::decode::header_geometry(&data)?
                .ok_or_else(|| invalid("JPEG geometry is unavailable"))?;
            if geometry.width != index.width || geometry.height != index.height {
                return Err(invalid("JPEG geometry differs from the sample description"));
            }
            encoded.push(data);
        }
        let mut output = Vec::with_capacity(selected.len());
        for ((frame_index, sample), data) in selected.into_iter().zip(encoded) {
            let image = Image::decode(&data)?;
            output.push(Frame {
                provenance: Provenance {
                    track_id,
                    frame_index,
                    decode_start_ticks: sample.start,
                    duration_ticks: sample.duration,
                    time_base: index.time_base,
                    offset: sample.offset,
                    length: sample.length,
                    encoded_sha256: hashing::sha256(&data),
                    representation: "decoded_jpeg_samples_no_display_transform",
                },
                image,
            });
        }
        Ok(output)
    })();
    result.map_err(|e: OperationError| e.operation(FRAME_OP))
}
pub fn extract_file(
    path: &Path,
    track_id: u64,
    indexes: &[usize],
) -> Result<Vec<Frame>, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut file| extract(&mut file, track_id, indexes))
        .map_err(|e| e.operation(FRAME_OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: FRAME_OP,
    category: "video",
    description: "Extract 1..=8 explicitly indexed independent baseline JPEG frames from bounded self-contained MOV/ISO-BMFF tables; exact range/time/hash provenance; decoded lossy pixels; no edits/composition/encryption/external refs or timestamp seeking.",
    input_type: "byte_reader",
    output_type: "video.frames",
    parameters: &[
        Parameter {
            name: "track_id",
            description: "Explicit nonzero video track ID.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned {
                min: 1,
                max: u32::MAX as u64,
            },
        },
        Parameter {
            name: "indexes",
            description: "Ordered unique zero-based frame indexes; 1..=8 selections, each <10000.",
            required: true,
            default: None,
            constraints: ParameterType::UnsignedList {
                min: 0,
                max: 9999,
                min_length: 1,
                max_length: 8,
                unique: true,
            },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "video frames",
    example: "cube video frames clip.mov --track-id 1 --indexes 0,2 --output-dir frames",
}];
