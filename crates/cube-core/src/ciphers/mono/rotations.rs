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
