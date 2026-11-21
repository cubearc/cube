//! Standard LZ4 frames and exact-sized raw blocks through the portable backend.
use super::{MAX_BYTES, MIB, Output, check_limit, exhausted, malformed, parameter};
use crate::errors::{ErrorCode, OperationError};
use lz4_flex::{
    block,
    frame::{BlockMode, BlockSize, FrameDecoder, FrameEncoder, FrameInfo},
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::io::{Read, Write};

const MAX_SCRATCH: usize = 64 * MIB;
const MAX_BLOCKS: usize = 10_000;
const FRAME_BLOCK: usize = 64 << 10;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    #[schemars(range(max = 16_777_216))]
    pub max_input: usize,
    #[schemars(range(max = 16_777_216))]
    pub max_output: usize,
    #[schemars(range(max = 67_108_864))]
    pub max_scratch: usize,
    #[schemars(range(max = 10_000))]
    pub max_blocks: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input: MAX_BYTES,
            max_output: MAX_BYTES,
            max_scratch: MAX_SCRATCH,
            max_blocks: MAX_BLOCKS,
        }
    }
}

impl Limits {
    fn check(self, length: usize) -> Result<(), OperationError> {
        if self.max_input > MAX_BYTES
            || self.max_output > MAX_BYTES
            || self.max_scratch > MAX_SCRATCH
            || self.max_blocks > MAX_BLOCKS
        {
            return Err(
                parameter("LZ4 limits can only lower the hard format ceilings").parameter("limits"),
            );
        }
        check_limit(length, self.max_input, "LZ4 input exceeds its bound")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Block,
    Frame,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decode {
    Block {
        #[schemars(range(max = 16_777_216))]
        expected_length: usize,
    },
    Frame,
}

fn invalid() -> OperationError {
    malformed("invalid, truncated or trailing data for the selected LZ4 format")
}

/// Encode raw LZ4 blocks or one frame with independent 64 KiB blocks, a
/// declared content size and both optional checksums.
pub fn encode(data: &[u8], mode: Mode, limits: Limits) -> Result<Vec<u8>, OperationError> {
    limits.check(data.len())?;
    match mode {
        Mode::Block => {
            // Safe encoder uses a fixed-sized hash table plus its worst-case
            // encoded buffer, independent of the caller's output limit.
            let bound = block::get_maximum_output_size(data.len());
            check_limit(
                bound + (32 << 10),
                limits.max_scratch,
                "LZ4 raw encoder scratch exceeds its bound",
            )?;
            let result = block::compress(data);
            check_limit(
                result.len(),
                limits.max_output,
                "LZ4 output exceeds its bound",
            )?;
            Ok(result)
        }
        Mode::Frame => {
            check_limit(
                data.len().div_ceil(FRAME_BLOCK),
                limits.max_blocks,
                "LZ4 block limit exceeded",
            )?;
            check_limit(
                3 * FRAME_BLOCK,
                limits.max_scratch,
                "LZ4 frame encoder scratch exceeds its bound",
            )?;
            let settings = FrameInfo::new()
                .block_size(BlockSize::Max64KB)
                .block_mode(BlockMode::Independent)
                .block_checksums(true)
                .content_checksum(true)
                .content_size(Some(data.len() as u64));
            let mut encoder =
                FrameEncoder::with_frame_info(settings, Output::new(limits.max_output));
            encoder.write_all(data).map_err(|_| invalid())?;
            let output = encoder.finish().map_err(|_| invalid())?;
            if output.overflowed {
                return Err(exhausted("LZ4 output exceeds its bound"));
            }
            Ok(output.data)
        }
    }
}

fn take<'a>(rest: &mut &'a [u8], length: usize) -> Result<&'a [u8], OperationError> {
    let (field, next) = rest.split_at_checked(length).ok_or_else(invalid)?;
    *rest = next;
    Ok(field)
}

fn frame_preflight(mut data: &[u8], limits: Limits) -> Result<(), OperationError> {
    if take(&mut data, 4)? != [4, 34, 77, 24] {
        return Err(OperationError::new(
            ErrorCode::UnsupportedFormat,
            "expected one standard LZ4 frame",
        ));
    }
    let [flags, bd] = *take(&mut data, 2)?
        .first_chunk::<2>()
        .expect("checked descriptor");
    if flags & 0xc2 != 0x40 || bd & 0x8f != 0 {
        return Err(invalid());
    }
    if flags & 1 != 0 {
        return Err(OperationError::new(
            ErrorCode::UnsupportedFormat,
            "LZ4 dictionaries are unsupported",
        ));
    }
    let max_block = match bd >> 4 {
        4 => FRAME_BLOCK,
        5 => 256 << 10,
        6 => MIB,
        7 => 4 * MIB,
        _ => return Err(invalid()),
    };
    let buffers = if flags & 0x20 != 0 {
        2 * max_block
    } else {
        3 * max_block + FRAME_BLOCK
    };
    check_limit(
        buffers + 8192,
        limits.max_scratch,
        "LZ4 frame buffers exceed the scratch bound",
    )?;
    if flags & 8 != 0 {
        let size = u64::from_le_bytes(take(&mut data, 8)?.try_into().expect("content size"));
        if size > limits.max_output as u64 {
            return Err(exhausted(
                "declared LZ4 content size exceeds the output bound",
            ));
        }
    }
    take(&mut data, 1)?;
    for blocks in 0..=limits.max_blocks {
        let raw = u32::from_le_bytes(take(&mut data, 4)?.try_into().expect("block extent"));
        if raw == 0 {
            if flags & 4 != 0 {
                take(&mut data, 4)?;
            }
            return if data.is_empty() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if blocks == limits.max_blocks {
            break;
        }
        let length = (raw & 0x7fff_ffff) as usize;
        if length > max_block {
            return Err(invalid());
        }
        take(&mut data, length)?;
        if flags & 0x10 != 0 {
            take(&mut data, 4)?;
        }
    }
    Err(exhausted("LZ4 block limit exceeded"))
}

/// Decode one exact raw block or one standard LZ4 frame. Verify every checksum
/// advertised in a frame and reject all trailing bytes.
pub fn decode(data: &[u8], mode: Decode, limits: Limits) -> Result<Vec<u8>, OperationError> {
    limits.check(data.len())?;
    match mode {
        Decode::Block { expected_length } => {
            check_limit(
                expected_length,
                limits.max_output,
                "declared LZ4 block length exceeds its output bound",
            )?;
            let mut output = vec![0; expected_length];
            let actual = block::decompress_into(data, &mut output).map_err(|_| invalid())?;
            if actual != expected_length {
                return Err(invalid());
            }
            Ok(output)
        }
        Decode::Frame => {
            frame_preflight(data, limits)?;
            let mut decoder = FrameDecoder::new(data);
            let mut output = Output::new(limits.max_output);
            let mut buffer = [0; 8192];
            loop {
                let space = buffer.len().min(limits.max_output - output.data.len() + 1);
                let count = decoder.read(&mut buffer[..space]).map_err(|_| invalid())?;
                if count == 0 {
                    return Ok(output.data);
                }
                if !output.append(&buffer[..count]) {
                    return Err(exhausted("LZ4 output exceeds its bound"));
                }
            }
        }
    }
}
