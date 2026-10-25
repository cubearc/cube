//! Bounded frame pairs, using Image::compare for every pixel decision.
use super::{
    frames::{self, Frame, Provenance},
    *,
};
use crate::{
    image::{Channel, Comparison},
    operations::{Parameter, ParameterType, ParameterValue},
};
pub const MAX_PAIRS: usize = 8;
pub const MAX_PAIR_PIXELS: u64 = 8 * 1024 * 1024;
const COMPARE_OP: &str = "video.frames.compare";
#[derive(Debug, Serialize)]
pub struct PairComparison {
    pub left: Provenance,
    pub right: Provenance,
    pub comparison: Comparison,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub pairs: Vec<PairComparison>,
    pub compared_pair_pixels: u64,
    pub retained_differences: usize,
    pub provenance_origin: &'static str,
    pub limitations: &'static str,
}
fn parameters(
    pairs: &[(usize, usize)],
    channels: Option<&[Channel]>,
    max_results: usize,
) -> Result<(), OperationError> {
    if pairs.is_empty()
        || pairs.len() > MAX_PAIRS
        || pairs
            .iter()
            .any(|&(a, b)| a >= MAX_RECORDS || b >= MAX_RECORDS)
    {
        return Err(invalid("select 1..=8 frame pairs with indexes below 10000"));
    }
    if !(1..=10000).contains(&max_results) {
        return Err(invalid(
            "max_results must be 1..=10000 across the entire batch",
        ));
    }
    if let Some(channels) = channels
        && (channels.is_empty()
            || channels.len() > 4
            || channels
                .iter()
                .enumerate()
                .any(|(i, c)| channels[..i].contains(c)))
    {
        return Err(invalid("select 1..=4 unique channels"));
    }
    Ok(())
}
/// Compare supplied extracted frames by positions in a <=8-frame slice. Caller
/// provenance is retained as supplied data, not authenticated. Each call permits
/// <=8 pairs, <=8 Mi compared pixel pairs, <=64 MiB retained input samples and a
/// global 1..=10000 difference-record budget. Counts remain exact when truncated.
/// Dimensions must match, with the existing Image model/channel rules; no resize,
/// masks, conversion, perceptual scoring or inference occurs.
pub fn compare_selected(
    frames: &[Frame],
    pairs: &[(usize, usize)],
    channels: Option<&[Channel]>,
    max_results: usize,
) -> Result<Report, OperationError> {
    let result = (|| {
        parameters(pairs, channels, max_results)?;
        if frames.is_empty()
            || frames.len() > frames::MAX_FRAMES
            || frames
                .iter()
                .map(|f| f.image.samples().len())
                .sum::<usize>()
                > frames::MAX_TOTAL_DECODED
        {
            return Err(limit("comparison input exceeds frame/sample budgets"));
        }
        let mut pixels = 0u64;
        for &(left, right) in pairs {
            let left = frames
                .get(left)
                .ok_or_else(|| invalid("left frame selection is out of bounds"))?;
            let right = frames
                .get(right)
                .ok_or_else(|| invalid("right frame selection is out of bounds"))?;
            if left.image.width() != right.image.width()
                || left.image.height() != right.image.height()
            {
                return Err(invalid("frame dimensions differ; comparison never resizes"));
            }
            pixels += u64::from(left.image.width()) * u64::from(left.image.height());
        }
        if pixels > MAX_PAIR_PIXELS {
            return Err(limit("comparison exceeds 8 Mi pixel-pair work budget"));
        }
        for frame in frames {
            let p = &frame.provenance;
            if p.encoded_sha256.len() != 64
                || !p.encoded_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || p.time_base.numerator == 0
                || p.time_base.denominator == 0
                || p.offset
                    .checked_add(u64::from(p.length))
                    .is_none_or(|end| end > MAX_FILE_BYTES)
            {
                return Err(invalid("invalid supplied frame provenance"));
            }
        }
        let mut output = Vec::with_capacity(pairs.len());
        let mut retained = 0;
        for &(left, right) in pairs {
            let left = &frames[left];
            let right = &frames[right];
            let remaining = max_results - retained;
            let mut comparison =
                left.image
                    .compare(&right.image, None, channels, remaining.max(1), false)?;
            comparison.differences.truncate(remaining);
            comparison.complete =
                comparison.differences.len() as u64 == comparison.differing_pixels;
            retained += comparison.differences.len();
            output.push(PairComparison {
                left: left.provenance.clone(),
                right: right.provenance.clone(),
                comparison,
            });
        }
        Ok(Report {
            pairs: output,
            compared_pair_pixels: pixels,
            retained_differences: retained,
            provenance_origin: "provided_frame_records_not_authenticated",
            limitations: "Exact decoded-sample comparison; JPEG decoding is lossy-source interpretation, not original encoded-byte equality or perceptual similarity. No display transforms or resampling.",
        })
    })();
    result.map_err(|e: OperationError| e.operation(COMPARE_OP))
}
/// Extract the union of explicitly requested source frame indexes once and reuse
/// compare_selected. Source JPEG extraction has its own bounds; no full decode.
pub fn compare(
    source: &mut (impl Read + Seek),
    track_id: u64,
    pairs: &[(usize, usize)],
    channels: Option<&[Channel]>,
    max_results: usize,
) -> Result<Report, OperationError> {
    let result = (|| {
        parameters(pairs, channels, max_results)?;
        let mut indexes = Vec::new();
        for &(left, right) in pairs {
            for n in [left, right] {
                if !indexes.contains(&n) {
                    indexes.push(n);
                }
            }
        }
        if indexes.len() > frames::MAX_FRAMES {
            return Err(limit("frame-pair union exceeds eight selected JPEGs"));
        }
        let selected = frames::extract(source, track_id, &indexes)?;
        let remapped: Vec<_> = pairs
            .iter()
            .map(|(a, b)| {
                (
                    indexes.iter().position(|i| i == a).expect("selected left"),
                    indexes.iter().position(|i| i == b).expect("selected right"),
                )
            })
            .collect();
        let mut report = compare_selected(&selected, &remapped, channels, max_results)?;
        report.provenance_origin = "validated_selected_sample_ranges_and_encoded_hashes";
        Ok(report)
    })();
    result.map_err(|e: OperationError| e.operation(COMPARE_OP))
}
pub fn compare_file(
    path: &Path,
    track_id: u64,
    pairs: &[(usize, usize)],
    channels: Option<&[Channel]>,
    max_results: usize,
) -> Result<Report, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut file| compare(&mut file, track_id, pairs, channels, max_results))
        .map_err(|e| e.operation(COMPARE_OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: COMPARE_OP,
    category: "video",
    description: "Compare <=8 explicit MOV/JPEG frame pairs through existing Image comparison, retaining exact extraction provenance; <=8 selected frames, 8 Mi pixel pairs, 10000 difference records; no resizing, perception or lossy-source equivalence claims.",
    input_type: "byte_reader",
    output_type: "video.frame_comparisons",
    parameters: &[
        Parameter {
            name: "track_id",
            description: "Explicit nonzero JPEG video track ID.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned {
                min: 1,
                max: u32::MAX as u64,
            },
        },
        Parameter {
            name: "pairs",
            description: "Flattened left/right frame indexes, 1..=8 pairs (even length); CLI uses left:right and SDK uses tuples.",
            required: true,
            default: None,
            constraints: ParameterType::UnsignedList {
                min: 0,
                max: 9999,
                min_length: 2,
                max_length: 16,
                unique: false,
            },
        },
        Parameter {
            name: "channels",
            description: "Optional exact channel selection; omitted means existing native image-model semantics.",
            required: false,
            default: None,
            constraints: ParameterType::TextList {
                choices: crate::image::analyze::CHANNEL_NAMES,
                min_length: 1,
                max_length: 4,
                unique: true,
            },
        },
        Parameter {
            name: "max_results",
            description: "Global retained difference-record budget; exact pixel counts remain available.",
            required: false,
            default: Some(ParameterValue::Unsigned(1000)),
            constraints: ParameterType::Unsigned { min: 1, max: 10000 },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "video compare",
    example: "cube video compare clip.mov --track-id 1 --pairs 0:1,1:2 --max-results 100",
}];
