//! Fixed 2010 English VIC: supplied keys, split position, indicator and pad digits.
//! Exact A..Z/period text; no numeric mode, case folding, RNG or Cyrillic claim.
mod disruption;
pub(crate) mod fixtures;
mod keys;
use crate::{
    ciphers::{
        poly::{bound, invalid},
        transposition::{apply, column_order, wire},
    },
    errors::{ErrorCode, OperationError},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_PLAINTEXT: usize = 65534;
pub const MAX_CIPHERTEXT: usize = 131080;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VicEnglish2010 {
    #[schemars(length(equal = 20), regex(pattern = r"^[A-Z]{20}$"))]
    pub phrase: String,
    #[schemars(length(equal = 8), regex(pattern = r"^[A-Z]{8}$"))]
    pub mnemonic: String,
    #[schemars(length(equal = 6), regex(pattern = r"^[0-9]{6}$"))]
    pub date_digits: String,
    #[schemars(range(min = 1, max = 24))]
    #[serde(deserialize_with = "wire::integer")]
    pub personal_number: usize,
    #[schemars(length(equal = 5), regex(pattern = r"^[0-9]{5}$"))]
    pub message_indicator: String,
    /// Retain this exact plaintext byte/scalar split for both directions.
    #[schemars(range(min = 0, max = 65534))]
    #[serde(deserialize_with = "wire::integer")]
    pub split_at: usize,
    #[schemars(length(max = 4), regex(pattern = r"^[0-9]{0,4}$"))]
    pub padding: String,
}

fn input_error(op: &'static str, field: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "input or declared frame evidence is inconsistent",
    )
    .operation(op)
    .parameter(field)
}
fn plain(text: &str, op: &'static str) -> Result<(), OperationError> {
    bound(op, "text", text.len(), MAX_PLAINTEXT)?;
    if !text.bytes().all(|c| c.is_ascii_uppercase() || c == b'.') {
        return Err(input_error(op, "text"));
    }
    Ok(())
}
fn transpose(text: &str, key: &[usize], decode: bool) -> String {
    let x: Vec<_> = text.chars().collect();
    apply(&x, &column_order(x.len(), key), decode)
}
fn indicator_position(
    length: usize,
    p: &VicEnglish2010,
    op: &'static str,
) -> Result<usize, OperationError> {
    let digit = usize::from(p.date_digits.as_bytes()[5] - b'0');
    let distance = if digit == 0 { 10 } else { digit };
    length
        .checked_sub(distance * 5)
        .ok_or_else(|| input_error(op, "message_indicator"))
}

fn encode(
    text: &str,
    p: &VicEnglish2010,
    k: &keys::Keys,
    op: &'static str,
) -> Result<String, OperationError> {
    plain(text, op)?;
    if p.split_at > text.len() {
        return Err(invalid(
            op,
            "split_at",
            "split must lie within the canonical plaintext",
        ));
    }
    let rotated = format!("{}..{}", &text[p.split_at..], &text[..p.split_at]);
    let mut digits = k.board.encode(&rotated).map_err(|e| e.operation(op))?;
    if p.padding.len() != (5 - digits.len() % 5) % 5 {
        return Err(invalid(
            op,
            "padding",
            "provide exactly the minimum number of pad digits",
        ));
    }
    digits.push_str(&p.padding);
    let first = transpose(&digits, &k.first, false);
    let mut cipher = disruption::transform(&first, &k.second, false);
    let at = indicator_position(cipher.len() + 5, p, op)?;
    cipher.insert_str(at, &p.message_indicator);
    bound(op, "output", cipher.len(), MAX_CIPHERTEXT)?;
    Ok(cipher)
}
fn decode(
    text: &str,
    p: &VicEnglish2010,
    k: &keys::Keys,
    op: &'static str,
) -> Result<String, OperationError> {
    bound(op, "text", text.len(), MAX_CIPHERTEXT)?;
    if text.len() < 10 || !text.len().is_multiple_of(5) || !text.bytes().all(|c| c.is_ascii_digit())
    {
        return Err(input_error(op, "text"));
    }
    let at = indicator_position(text.len(), p, op)?;
    if text[at..at + 5] != p.message_indicator {
        return Err(input_error(op, "message_indicator"));
    }
    let cipher = format!("{}{}", &text[..at], &text[at + 5..]);
    let first = disruption::transform(&cipher, &k.second, true);
    let mut digits = transpose(&first, &k.first, true);
    if !digits.ends_with(&p.padding) {
        return Err(input_error(op, "padding"));
    }
    digits.truncate(digits.len() - p.padding.len());
    let rotated = k.board.decode(&digits).map_err(|e| e.operation(op))?;
    let n = rotated
        .len()
        .checked_sub(2)
        .filter(|&n| p.split_at <= n)
        .ok_or_else(|| input_error(op, "split_at"))?;
    let marker = n - p.split_at;
    if &rotated[marker..marker + 2] != ".." {
        return Err(input_error(op, "split_at"));
    }
    let plaintext = format!("{}{}", &rotated[marker + 2..], &rotated[..marker]);
    plain(&plaintext, op)?;
    Ok(plaintext)
}

/// Deterministic end-to-end encryption/decryption from exact supplied fields.
/// The split and pad evidence are required for inverse transport; zero in the
/// final date position denotes the tenth five-digit group counting from the end.
pub fn transform(
    text: &str,
    p: &VicEnglish2010,
    decode_message: bool,
) -> Result<String, OperationError> {
    let op = if decode_message {
        "cipher.vic.decode"
    } else {
        "cipher.vic.encode"
    };
    if p.padding.len() > 4 || !p.padding.bytes().all(|c| c.is_ascii_digit()) {
        return Err(invalid(
            op,
            "padding",
            "padding must contain zero to four exact digits",
        ));
    }
    if p.split_at > MAX_PLAINTEXT {
        return Err(invalid(
            op,
            "split_at",
            "split exceeds the source profile limit",
        ));
    }
    let keys = keys::derive(p, op)?;
    if decode_message {
        decode(text, p, &keys, op)
    } else {
        encode(text, p, &keys, op)
    }
}
