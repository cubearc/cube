//! Self-inverse ASCII profiles; nonmatching scalars are preserved exactly.
use super::*;
fn transform(text: &str, profile: u8) -> Result<String, OperationError> {
    let op = match profile {
        5 => "cipher.rot5",
        18 => "cipher.rot18",
        _ => "cipher.rot47",
    };
    bound(op, "text", text.len(), MAX_INPUT)?;
    let letters = (profile == 18).then(|| crate::ciphers::rot13(text));
    let text = letters.as_deref().unwrap_or(text);
    let output = text
        .chars()
        .map(|c| match profile {
            5 | 18 if c.is_ascii_digit() => char::from(b'0' + (c as u8 - b'0' + 5) % 10),
            47 if ('!'..='~').contains(&c) => char::from(b'!' + (c as u8 - b'!' + 47) % 94),
            _ => c,
        })
        .collect();
    Ok(output)
}
/// Rotate only ASCII digits by five; preserve all other scalars, <=1 MiB.
pub fn rot5(text: &str) -> Result<String, OperationError> {
    transform(text, 5)
}
/// Compose letter ROT13 and digit ROT5; preserve all other scalars, <=1 MiB.
pub fn rot18(text: &str) -> Result<String, OperationError> {
    transform(text, 18)
}
/// Rotate printable ASCII U+0021..U+007E by 47; preserve space/Unicode, <=1 MiB.
pub fn rot47(text: &str) -> Result<String, OperationError> {
    transform(text, 47)
}

/// Apply the fixed ROT8000 BMP profile from rottytooth revision c4790563.
/// Rotates the 63,404 eligible BMP units by 31,702; excluded units and
/// non-BMP scalars are preserved, with no normalization. Valid UTF-8 only.
/// Input is at most 1 MiB; output is at most 3 MiB. Uses fixed range state.
pub fn rot8000(text: &str) -> Result<String, OperationError> {
    bound("cipher.rot8000", "text", text.len(), MAX_INPUT)?;
    Ok(text.chars().map(rotate_bmp).collect())
}
// Numeric profile facts, independently implemented; no upstream code/table.
const BMP_RANGES: [(u32, u32); 9] = [
    (0x21, 0x7e),
    (0xa1, 0x167f),
    (0x1681, 0x1fff),
    (0x200b, 0x2027),
    (0x202a, 0x202e),
    (0x2030, 0x205e),
    (0x2060, 0x2fff),
    (0x3001, 0xd7ff),
    (0xe000, 0xffff),
];
fn rotate_bmp(c: char) -> char {
    let code = u32::from(c);
    let mut rank = 0;
    for (start, end) in BMP_RANGES {
        if (start..=end).contains(&code) {
            return bmp_at((rank + code - start + 31702) % 63404);
        }
        rank += end - start + 1;
    }
    c
}
fn bmp_at(mut rank: u32) -> char {
    for (start, end) in BMP_RANGES {
        let length = end - start + 1;
        if rank < length {
            return char::from_u32(start + rank).expect("eligible BMP range excludes surrogates");
        }
        rank -= length;
    }
    unreachable!("ROT8000 rank is below the eligible population")
}
