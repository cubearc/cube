//! Adobe ASCII85 packing, with canonical zero and tail forms.
use super::{MAX_BYTES, invalid};
use crate::errors::{ErrorCode, OperationError};

fn digits(mut word: u32) -> [u8; 5] {
    let mut digits = [b'!'; 5];
    for byte in digits.iter_mut().rev() {
        *byte += (word % 85) as u8;
        word /= 85;
    }
    digits
}

pub(super) fn encode(data: &[u8], framed: bool) -> String {
    let mut output = String::with_capacity(data.len().div_ceil(4) * 5 + usize::from(framed) * 4);
    if framed {
        output.push_str("<~");
    }
    for chunk in data.chunks(4) {
        let mut word = [0; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        if chunk.len() == 4 && word == [0; 4] {
            output.push('z');
        } else {
            let digits = digits(u32::from_be_bytes(word));
            output.extend(digits[..chunk.len() + 1].iter().copied().map(char::from));
        }
    }
    if framed {
        output.push_str("~>");
    }
    output
}

pub(super) fn decode(
    text: &str,
    framed: bool,
    id: &'static str,
) -> Result<Vec<u8>, OperationError> {
    let body = if framed {
        text.strip_prefix("<~")
            .and_then(|tail| tail.strip_suffix("~>"))
            .ok_or_else(|| invalid(id, 0, "expected Adobe Ascii85 framing"))?
    } else {
        text
    };
    let start = usize::from(framed) * 2;
    let mut input = body.as_bytes();
    let mut output = Vec::with_capacity(input.len().min(MAX_BYTES));
    while !input.is_empty() {
        let offset = start + body.len() - input.len();
        let (bytes, length, count) = if input[0] == b'z' {
            ([0; 4], 4, 1)
        } else {
            let count = input.len().min(5);
            let group = &input[..count];
            if count == 1 || !group.iter().all(|byte| (b'!'..=b'u').contains(byte)) {
                return Err(invalid(
                    id,
                    offset,
                    "incomplete group or invalid Ascii85 digit",
                ));
            }
            let word = group
                .iter()
                .copied()
                .chain(std::iter::repeat_n(b'u', 5 - count))
                .fold(0u64, |word, byte| word * 85 + u64::from(byte - b'!'));
            let word = u32::try_from(word)
                .map_err(|_| invalid(id, offset, "Ascii85 group exceeds 32 bits"))?;
            let mut bytes = word.to_be_bytes();
            bytes[count - 1..].fill(0);
            if (count == 5 && word == 0) || digits(u32::from_be_bytes(bytes))[..count] != *group {
                return Err(invalid(id, offset, "noncanonical Ascii85 group"));
            }
            (bytes, count - 1, count)
        };
        if output.len() + length > MAX_BYTES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "Ascii85 output exceeds 1 MiB",
            )
            .operation(id));
        }
        output.extend_from_slice(&bytes[..length]);
        input = &input[count..];
    }
    Ok(output)
}
