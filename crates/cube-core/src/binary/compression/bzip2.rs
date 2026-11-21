//! Bounded BZh members with verified CRCs and explicit concatenation.
use super::{MAX_BYTES, check_limit, exhausted, malformed, parameter};
use crate::errors::OperationError;
use cube_bzip2_backend::BoundedError;

const MAX_BLOCKS: usize = 10_000;
pub const DEFAULT_LEVEL: u8 = 1;

fn backend(error: BoundedError) -> OperationError {
    match error {
        BoundedError::Limit => exhausted("bzip2 output, block or memory budget exceeded"),
        BoundedError::Invalid => malformed("invalid or incomplete bzip2 stream"),
    }
}

/// Encode with level 1 through 9, fixed work factor 30 and standard BZh framing.
pub fn encode(data: &[u8], level: u8) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MAX_BYTES, "bzip2 input exceeds 16 MiB")?;
    if !(1..=9).contains(&level) {
        return Err(parameter("bzip2 level must be in 1..=9").parameter("level"));
    }
    // libbzip2 documents input + 1% + 600 as sufficient for this buffer API.
    let capacity = (data.len() + data.len() / 100 + 600).min(MAX_BYTES);
    let mut output = vec![0; capacity];
    let size = cube_bzip2_backend::encode_bounded(data, &mut output, level).map_err(backend)?;
    output.truncate(size);
    Ok(output)
}

fn header(data: &[u8]) -> Result<(), OperationError> {
    if !matches!(data.get(..4), Some([b'B', b'Z', b'h', b'1'..=b'9'])) {
        return Err(malformed("expected a BZh member with a valid block size"));
    }
    // The stream level admits at most 900,000 coded bytes per block. Upstream
    // fast decoding uses one u32 per coded byte and its fixed decoder state.
    Ok(())
}

/// Decode one member or, when requested, concatenated members under one
/// 16 MiB input/output and 10,000-block budget.
pub fn decode(mut data: &[u8], concatenate: bool) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MAX_BYTES, "bzip2 input exceeds 16 MiB")?;
    header(data)?;
    let (mut output, consumed, mut blocks) =
        cube_bzip2_backend::decode_bounded(data, MAX_BYTES, MAX_BLOCKS).map_err(backend)?;
    data = &data[consumed..];
    if !concatenate && !data.is_empty() {
        return Err(malformed("trailing bytes after a bzip2 member"));
    }
    let mut members = 1;
    while !data.is_empty() {
        members += 1;
        check_limit(members, MAX_BLOCKS, "bzip2 member limit exceeded")?;
        header(data)?;
        let (member, consumed, count) =
            cube_bzip2_backend::decode_bounded(data, MAX_BYTES - output.len(), MAX_BLOCKS - blocks)
                .map_err(backend)?;
        blocks += count;
        output.extend_from_slice(&member);
        data = &data[consumed..];
    }
    Ok(output)
}
