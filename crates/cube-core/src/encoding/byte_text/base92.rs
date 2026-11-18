//! Original Base92 wire representation, implemented from its documented format.
use super::{MAX_BYTES, check, invalid};
use crate::errors::OperationError;

const ALPHABET: &[u8; 91] =
    b"!#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_abcdefghijklmnopqrstuvwxyz{|}";

pub(super) fn encode(data: &[u8]) -> String {
    if data.is_empty() {
        return "~".to_owned();
    }
    let mut output = String::with_capacity((data.len() * 16).div_ceil(13));
    let mut bits = 0;
    let mut pending = 0u32;
    for &byte in data {
        pending = (pending << 8) | u32::from(byte);
        bits += 8;
        if bits >= 13 {
            bits -= 13;
            pair(&mut output, (pending >> bits) as usize);
            pending &= (1 << bits) - 1;
        }
    }
    if bits > 6 {
        pair(&mut output, (pending << (13 - bits)) as usize);
    } else if bits != 0 {
        output.push(char::from(ALPHABET[(pending << (6 - bits)) as usize]));
    }
    output
}

fn pair(out: &mut String, value: usize) {
    out.push(char::from(ALPHABET[value / 91]));
    out.push(char::from(ALPHABET[value % 91]));
}

pub(super) fn decode(text: &str, id: &'static str) -> Result<Vec<u8>, OperationError> {
    if text == "~" {
        return Ok(Vec::new());
    }
    let mut output = Vec::with_capacity(text.len().min(MAX_BYTES));
    let mut bits = 0;
    let mut pending = 0u32;
    for (i, group) in text.as_bytes().chunks(2).enumerate() {
        let digit = |byte| {
            ALPHABET
                .binary_search(&byte)
                .map(|value| value as u32)
                .map_err(|_| invalid(id, 2 * i, "digit outside the original Base92 alphabet"))
        };
        let left = digit(group[0])?;
        let (value, width) = match group.get(1) {
            Some(&right) => (left * 91 + digit(right)?, 13),
            None => (left, 6),
        };
        if value >= 1 << width {
            return Err(invalid(id, 2 * i, "Base92 group is out of range"));
        }
        pending = (pending << width) | value;
        bits += width;
        while bits >= 8 {
            bits -= 8;
            check(output.len() + 1, MAX_BYTES, id)?;
            output.push((pending >> bits) as u8);
            pending &= (1 << bits) - 1;
        }
    }
    if pending != 0 || encode(&output) != text {
        return Err(invalid(
            id,
            text.len().saturating_sub(1),
            "noncanonical Base92 tail or empty marker",
        ));
    }
    Ok(output)
}
