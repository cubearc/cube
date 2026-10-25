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
    let length =
        u32::try_from(samples.raw().len()).map_err(|_| limit(OP, "data exceeds RIFF size"))?;
    let riff = 36u32
        .checked_add(length)
        .and_then(|n| n.checked_add(length % 2))
        .ok_or_else(|| limit(OP, "RIFF extent overflow"))?;
    let mut out = Vec::with_capacity(riff as usize + 8);
    out.extend(b"RIFF");
    out.extend(riff.to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(format.sample_rate.to_le_bytes());
    out.extend(byte_rate.to_le_bytes());
    out.extend(align.to_le_bytes());
    out.extend(format.bits_per_sample.to_le_bytes());
    out.extend(b"data");
    out.extend(length.to_le_bytes());
    for frame in samples.raw().chunks_exact(usize::from(align)).rev() {
        out.extend(frame)
    }
    if length % 2 != 0 {
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
