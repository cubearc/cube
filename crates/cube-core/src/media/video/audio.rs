//! Exact raw PCM8 track remuxing; no decoder, resampling or byte conversion.
use super::*;
use crate::{
    hashing,
    media::{pcm, wave},
    operations::{Parameter, ParameterType},
};
const AUDIO_OP: &str = "video.audio.extract";
#[derive(Debug, Serialize)]
pub struct SourceRange {
    pub offset: u64,
    pub length: u64,
}
#[derive(Debug, Serialize)]
pub struct Provenance {
    pub track_id: u64,
    pub codec: &'static str,
    pub format: wave::Format,
    pub frames: u64,
    pub decode_start_ticks: u64,
    pub duration_ticks: u64,
    pub time_base: TimeBase,
    pub source_ranges: Vec<SourceRange>,
    pub pcm_bytes: usize,
    pub pcm_sha256: String,
    pub wav_sha256: String,
    pub processing: &'static str,
}
#[derive(Debug, Serialize)]
pub struct AudioTrack {
    pub provenance: Provenance,
    #[serde(skip)]
    pub data: Vec<u8>,
}
/// Remux one explicitly selected unfragmented MOV/ISO-BMFF raw unsigned PCM8
/// mono/stereo track to minimal WAVE, preserving every interleaved sample byte.
/// Require one self-contained version-zero description, fixed frame-width stsz,
/// delta-1 stts and time scale equal to integer sample rate 1..=65535.
/// Shared indexing validates all ranges/tables/times before copying. Limits:
/// 1 GiB physical input, 2 MiB metadata reads, 10000 cumulative atoms/table rows,
/// 1 Mi frames, 300 seconds, 64 MiB temporary index, 8 MiB PCM/WAV payload and
/// 10000 coalesced ordered provenance ranges. No decoding or implicit writes.
pub fn extract(
    source: &mut (impl Read + Seek),
    track_id: u64,
) -> Result<AudioTrack, OperationError> {
    let result = (|| {
        if track_id == 0 || track_id > u64::from(u32::MAX) {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "track_id must be a nonzero 32-bit ID",
            )
            .parameter("track_id"));
        }
        let info = inspect(source)?;
        let index = mp4::index::load(source, &info, track_id, mp4::index::Profile::Pcm8)?;
        let mp4::index::Layout::Pcm8(format) = index.layout else {
            return Err(unsupported("raw PCM8 layout required for audio extraction"));
        };
        let frames = index.samples.len() as u64;
        let pcm_bytes = index
            .samples
            .len()
            .checked_mul(usize::from(format.block_align))
            .ok_or_else(|| limit("PCM output size overflows"))?;
        if pcm_bytes as u64 > pcm::MAX_PCM_BYTES {
            return Err(limit("PCM exceeds 8 MiB output budget"));
        }
        let mut ranges: Vec<SourceRange> = Vec::new();
        for sample in &index.samples {
            if let Some(previous) = ranges.last_mut()
                && previous.offset.checked_add(previous.length) == Some(sample.offset)
            {
                previous.length = previous
                    .length
                    .checked_add(u64::from(sample.length))
                    .ok_or_else(|| invalid("source range overflows"))?;
            } else {
                if ranges.len() == MAX_RECORDS {
                    return Err(limit("audio provenance exceeds 10000 ranges"));
                }
                ranges.push(SourceRange {
                    offset: sample.offset,
                    length: u64::from(sample.length),
                });
            }
        }
        let mut raw = vec![0; pcm_bytes];
        let mut position = 0usize;
        for range in &ranges {
            let length =
                usize::try_from(range.length).map_err(|_| limit("source range is too large"))?;
            let end = position
                .checked_add(length)
                .filter(|n| *n <= raw.len())
                .ok_or_else(|| invalid("source ranges disagree with PCM size"))?;
            source.seek(SeekFrom::Start(range.offset))?;
            source
                .read_exact(&mut raw[position..end])
                .map_err(|e| invalid("truncated selected PCM payload").caused_by(e))?;
            position = end;
        }
        if position != raw.len() {
            return Err(invalid("incomplete PCM source ranges"));
        }
        let mut data = wave::pcm_header(format, raw.len())?;
        data.extend(&raw);
        if !raw.len().is_multiple_of(2) {
            data.push(0);
        }
        Ok(AudioTrack {
            provenance: Provenance {
                track_id,
                codec: "raw ",
                format,
                frames,
                decode_start_ticks: 0,
                duration_ticks: frames,
                time_base: index.time_base,
                source_ranges: ranges,
                pcm_bytes,
                pcm_sha256: hashing::sha256(&raw),
                wav_sha256: hashing::sha256(&data),
                processing: "container_remux_only_unsigned_pcm8_samples_preserved",
            },
            data,
        })
    })();
    result.map_err(|e: OperationError| e.operation(AUDIO_OP))
}
pub fn extract_file(path: &Path, track_id: u64) -> Result<AudioTrack, OperationError> {
    forensics::open_regular(path)
        .and_then(|mut source| extract(&mut source, track_id))
        .map_err(|e| e.operation(AUDIO_OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: AUDIO_OP,
    category: "video",
    description: "Remux one explicit self-contained MOV raw-PCM8 mono/stereo track to bounded WAV bytes with exact source/time/hash provenance; no decoding, resampling or sample-byte changes; fixed stsz and delta-1 sample timing required.",
    input_type: "byte_reader",
    output_type: "video.audio_track",
    parameters: &[Parameter {
        name: "track_id",
        description: "Explicit nonzero raw PCM8 audio track ID.",
        required: true,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 1,
            max: u32::MAX as u64,
        },
    }],
    deterministic: true,
    reversible: Some(false),
    command: "video audio",
    example: "cube video audio clip.mov --track-id 2 --output track.wav",
}];
