//! TIFF 6.0 PackBits over a contiguous byte buffer.
use super::{MIB, check_limit, malformed};
use crate::errors::OperationError;

/// Encode TIFF 6.0 PackBits with greedy runs and literals for pairs.
pub fn encode(data: &[u8]) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MIB, "PackBits input exceeds 1 MiB")?;
    let mut output = Vec::with_capacity(data.len() + data.len().div_ceil(128));
    let mut input = data;
    while !input.is_empty() {
        let run = input
            .iter()
            .take(128)
            .take_while(|&&value| value == input[0])
            .count();
        let consumed = if run >= 3 {
            output.extend([((1 - run as i16) as i8) as u8, input[0]]);
            run
        } else {
            let literal = (1..128.min(input.len()))
                .find(|&i| {
                    input
                        .get(i..i + 3)
                        .is_some_and(|triple| triple[0] == triple[1] && triple[1] == triple[2])
                })
                .unwrap_or(128.min(input.len()));
            output.push((literal - 1) as u8);
            output.extend_from_slice(&input[..literal]);
            literal
        };
        input = &input[consumed..];
    }
    Ok(output)
}

fn packets(mut input: &[u8]) -> impl Iterator<Item = Result<(&[u8], usize), OperationError>> {
    std::iter::from_fn(move || {
        let (&header, tail) = input.split_first()?;
        let (extent, repeat) = match header as i8 {
            0..=127 => (usize::from(header) + 1, 1),
            -128 => (0, 0),
            header => (1, (1 - i16::from(header)) as usize),
        };
        let Some((payload, remaining)) = tail.split_at_checked(extent) else {
            input = &[];
            return Some(Err(malformed("truncated PackBits packet")));
        };
        input = remaining;
        Some(Ok((payload, repeat)))
    })
}

/// Validate each packet and any expected length, then decode the bounded stream.
pub fn decode(data: &[u8], expected_length: Option<usize>) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MIB, "PackBits input exceeds 1 MiB")?;
    if let Some(expected) = expected_length {
        check_limit(expected, 16 * MIB, "PackBits output exceeds 16 MiB")?;
    }
    let mut length = 0;
    for packet in packets(data) {
        let (payload, repeat) = packet?;
        length += payload.len() * repeat;
        check_limit(length, 16 * MIB, "PackBits output exceeds 16 MiB")?;
    }
    if expected_length.is_some_and(|expected| expected != length) {
        return Err(malformed(
            "PackBits output does not match the expected length",
        ));
    }
    let mut output = Vec::with_capacity(length);
    for packet in packets(data) {
        let (payload, repeat) = packet?;
        if repeat > 1 {
            output.resize(output.len() + repeat, payload[0]);
        } else {
            output.extend_from_slice(payload);
        }
    }
    Ok(output)
}
