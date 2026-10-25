//! Reverse exact selected PCM frames into a new minimal RIFF/WAVE representation.
use super::{pcm::Samples, spectrum::limit};
use crate::{errors::OperationError, operations::Descriptor};
/// Return a minimal PCM WAVE with fmt/data only, reversing frames while retaining
/// every native sample byte and selected channel grouping/order. Ancillary source
/// chunks are deliberately omitted. Existing immutable PCM bounds apply; RIFF
/// extents/byte rate are checked before allocating. Odd data gets one zero pad.
/// This function performs no filesystem writes; adapters create exclusive outputs.
pub fn wave(samples: &Samples) -> Result<Vec<u8>, OperationError> {
    const OP: &str = "audio.reverse";
    let format = samples.format();
    let channels = samples.channels().len() as u16;
    let align = channels
        .checked_mul(format.bits_per_sample / 8)
        .ok_or_else(|| limit(OP, "block alignment overflow"))?;
    let byte_rate = format
        .sample_rate
        .checked_mul(u32::from(align))
        .ok_or_else(|| limit(OP, "byte rate overflow"))?;
    let length = samples.raw().len();
    let format = super::wave::Format {
        channels,
        block_align: align,
        byte_rate,
        ..format
    };
    let mut out = super::wave::pcm_header(format, length).map_err(|e| e.operation(OP))?;
    for frame in samples.raw().chunks_exact(usize::from(align)).rev() {
        out.extend(frame)
    }
    if !length.is_multiple_of(2) {
        out.push(0)
    }
    Ok(out)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "audio.reverse",
    category: "audio",
    description: "Reverse selected PCM frames without changing sample bytes/channel grouping; generate bounded minimal WAV fmt/data with checked padding; source ancillary chunks are omitted.",
    input_type: "audio.pcm_samples",
    output_type: "bytes",
    parameters: &[],
    deterministic: true,
    reversible: Some(true),
    command: "audio reverse",
    example: "cube audio reverse sample.wav --output reversed.wav",
}];
