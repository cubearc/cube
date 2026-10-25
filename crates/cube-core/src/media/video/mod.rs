//! Bounded declared container metadata, not decoding or complete media validation.
pub mod comparison;
pub mod frames;
mod mp4;
mod webm;
use crate::{
    errors::{ErrorCode, OperationError},
    forensics,
    operations::Descriptor,
};
use serde::Serialize;
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};
pub const MAX_FILE_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_RECORDS: usize = 10000;
pub const MAX_READ_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_TRACKS: usize = 32;
const OP: &str = "video.container.inspect";
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message).operation(OP)
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation(OP)
}
fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message).operation(OP)
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct TimeBase {
    pub numerator: u64,
    pub denominator: u64,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(untagged)]
pub enum Duration {
    Integer(u64),
    Float(f64),
}
impl Duration {
    fn seconds(self, base: TimeBase) -> Result<f64, OperationError> {
        let ticks = match self {
            Self::Integer(n) => n as f64,
            Self::Float(n) => n,
        };
        let seconds = ticks * (base.numerator as f64 / base.denominator as f64);
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(invalid("invalid or excessive declared duration"));
        }
        Ok(seconds)
    }
}
#[derive(Debug, Serialize)]
pub struct Track {
    pub id: u64,
    pub kind: &'static str,
    pub codec_id: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub display_dimensions_fixed_16_16: Option<[u32; 2]>,
    pub channels: Option<u64>,
    pub sample_rate: Option<f64>,
    pub time_base: Option<TimeBase>,
    pub duration_ticks: Option<Duration>,
    pub duration_seconds: Option<f64>,
}
impl Default for Track {
    fn default() -> Self {
        Self {
            id: 0,
            kind: "unknown",
            codec_id: None,
            width: None,
            height: None,
            display_dimensions_fixed_16_16: None,
            channels: None,
            sample_rate: None,
            time_base: None,
            duration_ticks: None,
            duration_seconds: None,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Info {
    pub container: &'static str,
    pub major_brand: Option<String>,
    pub file_bytes: u64,
    pub time_base: Option<TimeBase>,
    pub duration_ticks: Option<Duration>,
    pub duration_seconds: Option<f64>,
    pub tracks: Vec<Track>,
    pub records_inspected: usize,
    pub metadata_bytes_read: usize,
    pub skipped_records: usize,
    pub verification: &'static str,
    pub limitations: &'static str,
}
impl Info {
    fn new(size: u64) -> Self {
        Self {
            container: "unknown",
            major_brand: None,
            file_bytes: size,
            time_base: None,
            duration_ticks: None,
            duration_seconds: None,
            tracks: Vec::new(),
            records_inspected: 0,
            metadata_bytes_read: 0,
            skipped_records: 0,
            verification: "declared_container_metadata_media_payload_unverified",
            limitations: "No decoding, sample/packet validation, checksum verification, seeking or extraction; absent fields are null. Derived seconds use floating point; original durations/time bases remain available.",
        }
    }
}
struct Reader<'a, R> {
    source: &'a mut R,
    size: u64,
    records: usize,
    read_bytes: usize,
    skipped: usize,
}
impl<R: Read + Seek> Reader<'_, R> {
    fn at(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, OperationError> {
        let end = offset
            .checked_add(size as u64)
            .ok_or_else(|| invalid("metadata offset overflows"))?;
        if end > self.size {
            return Err(invalid("metadata extends beyond source"));
        }
        self.read_bytes = self
            .read_bytes
            .checked_add(size)
            .ok_or_else(|| limit("read budget overflows"))?;
        if self.read_bytes > MAX_READ_BYTES || size > 4096 {
            return Err(limit(
                "container metadata exceeds 2 MiB cumulative reads or 4096-byte field",
            ));
        }
        self.source.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0; size];
        self.source.read_exact(&mut data).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                invalid("truncated container metadata").caused_by(e)
            } else {
                OperationError::io(e).operation(OP)
            }
        })?;
        Ok(data)
    }
    fn record(&mut self) -> Result<(), OperationError> {
        self.records += 1;
        if self.records > MAX_RECORDS {
            return Err(limit("container exceeds 10000 inspected records"));
        }
        Ok(())
    }
}
fn fourcc(bytes: &[u8]) -> String {
    if bytes.iter().all(|b| (32..=126).contains(b)) {
        String::from_utf8(bytes.to_vec()).expect("ASCII")
    } else {
        format!("hex:{}", hex::encode(bytes))
    }
}
/// Read declared MP4/MOV (unfragmented, one sample description per track) or WebM
/// metadata. At most 1 GiB source, 10000 records, 32 tracks, depth 8, 2 MiB total
/// metadata reads and 4096 bytes per field. Known header paths only are traversed;
/// unknown bodies/media payloads are skipped by checked extents. Unsupported
/// fragmentation/encryption, unknown-sized children, versions and profiles fail
/// explicitly. This does not establish that media packets or codec data are valid.
pub fn inspect(source: &mut (impl Read + Seek)) -> Result<Info, OperationError> {
    let result = (|| {
        let size = source.seek(SeekFrom::End(0))?;
        if size > MAX_FILE_BYTES {
            return Err(limit("container source exceeds 1 GiB"));
        }
        if size < 8 {
            return Err(invalid("container header is truncated"));
        }
        let mut reader = Reader {
            source,
            size,
            records: 0,
            read_bytes: 0,
            skipped: 0,
        };
        let signature = reader.at(0, 8)?;
        let mut info = Info::new(size);
        if signature[..4] == [0x1a, 0x45, 0xdf, 0xa3] {
            webm::inspect(&mut reader, &mut info)?;
        } else if [b"ftyp", b"moov", b"mdat", b"free", b"wide"]
            .contains(&<&[u8; 4]>::try_from(&signature[4..8]).expect("four bytes"))
        {
            mp4::inspect(&mut reader, &mut info)?;
        } else {
            return Err(unsupported(
                "only supported MP4/MOV and WebM metadata profiles are recognized",
            ));
        }
        if info.tracks.is_empty() {
            return Err(invalid("supported container requires track metadata"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for track in &mut info.tracks {
            if track.id == 0 || !ids.insert(track.id) {
                return Err(invalid("track identifiers must be nonzero and unique"));
            }
            if let (Some(duration), Some(base)) = (track.duration_ticks, track.time_base) {
                track.duration_seconds = Some(duration.seconds(base)?);
            }
        }
        if let (Some(duration), Some(base)) = (info.duration_ticks, info.time_base) {
            info.duration_seconds = Some(duration.seconds(base)?)
        }
        info.records_inspected = reader.records;
        info.metadata_bytes_read = reader.read_bytes;
        info.skipped_records = reader.skipped;
        Ok(info)
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
pub fn inspect_file(path: &Path) -> Result<Info, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut source| inspect(&mut source))
        .map_err(|e| e.operation(OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "video",
    description: "Bounded declared MP4/MOV or WebM container/track/codec/dimension/time metadata; no decoding or packet validation; fragmentation/encryption and unsupported profiles explicit.",
    input_type: "byte_reader",
    output_type: "video.container_metadata",
    parameters: &[],
    deterministic: true,
    reversible: None,
    command: "video inspect",
    example: "cube video inspect sample.mp4 --json",
}];
