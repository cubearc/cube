//! Exact native integer PCM bit streams, independent of amplitude interpretation.
use super::{
    pcm::Samples,
    spectrum::{invalid, limit},
};
pub use crate::binary::packing::{BitOrder, Incomplete};
use crate::{
    binary::packing::store,
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
pub const MAX_BITS: usize = 16 * 1024 * 1024;
#[derive(Clone, Debug)]
pub struct Options {
    pub bits: Vec<u8>,
    pub reverse_frames: bool,
    pub bit_order: BitOrder,
    pub incomplete: Incomplete,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            bits: vec![0],
            reverse_frames: false,
            bit_order: BitOrder::Msb,
            incomplete: Incomplete::Reject,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct BitStream {
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub channels: Vec<u16>,
    pub start_frame: u64,
    pub frames: u64,
    pub bits: Vec<u8>,
    pub reverse_frames: bool,
    pub bit_order: BitOrder,
    pub incomplete: Incomplete,
    pub storage: &'static str,
    pub traversal: &'static str,
    pub available_bits: usize,
    pub valid_bits: usize,
    pub dropped_bits: usize,
    #[serde(skip)]
    pub data: Vec<u8>,
}
/// Visit frames (optionally reverse time), then selected channels in their existing
/// order, then requested unique bit indexes. Bit 0 is the low bit of the native
/// little-endian storage, including unsigned 8-bit and signed two's-complement
/// 16/24/32-bit samples; no centering, sign extension, rescaling or UTF-8 conversion.
/// MSB/LSB packing and reject/drop/zero-pad tails share the image contracts.
/// At most 16 Mi source bits/2 MiB packed bytes; counts are checked before allocation.
pub fn extract(samples: &Samples, options: &Options) -> Result<BitStream, OperationError> {
    const OP: &str = "audio.bits.extract";
    let width = usize::from(samples.format().bits_per_sample / 8);
    if options.bits.is_empty()
        || options.bits.len() > width * 8
        || options
            .bits
            .iter()
            .enumerate()
            .any(|(i, b)| usize::from(*b) >= width * 8 || options.bits[..i].contains(b))
    {
        return Err(invalid(
            OP,
            "bits must be nonempty, unique and within native sample width",
        )
        .parameter("bits"));
    }
    let count = (samples.frames() as usize)
        .checked_mul(samples.channels().len())
        .and_then(|n| n.checked_mul(options.bits.len()))
        .ok_or_else(|| limit(OP, "bit count overflows"))?;
    if count > MAX_BITS {
        return Err(limit(
            OP,
            "extraction exceeds 16 Mi bits/2 MiB packed output",
        ));
    }
    let remainder = count % 8;
    let valid_bits = match options.incomplete {
        Incomplete::Reject if remainder != 0 => {
            return Err(
                invalid(OP, "incomplete output byte; select drop or pad explicitly")
                    .parameter("incomplete"),
            );
        }
        Incomplete::Drop => count - remainder,
        _ => count,
    };
    let mut data = vec![0; valid_bits.div_ceil(8)];
    let mut position = 0;
    let frame_bytes = width * samples.channels().len();
    for step in 0..samples.frames() as usize {
        let frame = if options.reverse_frames {
            samples.frames() as usize - 1 - step
        } else {
            step
        };
        for sample in
            samples.raw()[frame * frame_bytes..(frame + 1) * frame_bytes].chunks_exact(width)
        {
            for b in &options.bits {
                if position < valid_bits {
                    store(
                        &mut data,
                        position,
                        (sample[usize::from(*b) / 8] >> (*b % 8)) & 1,
                        options.bit_order,
                    );
                }
                position += 1;
            }
        }
    }
    Ok(BitStream {
        sample_rate: samples.format().sample_rate,
        bits_per_sample: samples.format().bits_per_sample,
        channels: samples.channels().to_vec(),
        start_frame: samples.start_frame(),
        frames: samples.frames(),
        bits: options.bits.clone(),
        reverse_frames: options.reverse_frames,
        bit_order: options.bit_order,
        incomplete: options.incomplete,
        storage: if width == 1 {
            "unsigned_8"
        } else {
            "signed_twos_complement"
        },
        traversal: "frame_channel_bit",
        available_bits: count,
        valid_bits,
        dropped_bits: count - valid_bits,
        data,
    })
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "audio.bits.extract",
    category: "audio",
    description: "Exact raw bit extraction from native PCM storage; frame/channel/bit order, optional reverse frames, shared MSB/LSB and reject/drop/pad semantics; <=16 Mi bits.",
    input_type: "audio.pcm_samples",
    output_type: "bytes",
    parameters: &[
        Parameter {
            name: "bits",
            description: "Ordered unique native sample bit indexes, zero is LSB; must fit selected sample width.",
            required: false,
            default: Some(ParameterValue::UnsignedList(&[0])),
            constraints: ParameterType::UnsignedList {
                min: 0,
                max: 31,
                min_length: 1,
                max_length: 32,
                unique: true,
            },
        },
        Parameter {
            name: "reverse_frames",
            description: "Reverse time/frame traversal, preserving channel order inside each frame.",
            required: false,
            default: Some(ParameterValue::Boolean(false)),
            constraints: ParameterType::Boolean,
        },
        Parameter {
            name: "bit_order",
            description: "First bit goes into bit 7 (msb) or bit 0 (lsb) of each output byte.",
            required: false,
            default: Some(ParameterValue::Text("msb")),
            constraints: ParameterType::Text {
                choices: &["msb", "lsb"],
            },
        },
        Parameter {
            name: "incomplete",
            description: "Policy for final incomplete output byte: reject, drop or zero-pad.",
            required: false,
            default: Some(ParameterValue::Text("reject")),
            constraints: ParameterType::Text {
                choices: &["reject", "drop", "pad"],
            },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "audio bits",
    example: "cube audio bits sample.wav --channels 0 --bits 0 --frames 8 --stdout",
}];
