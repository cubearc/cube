//! Bellaso's fixed 1553 table and ordinary continuous countersign.
use super::*;
use crate::ciphers::mono::params::Unmapped;
use crate::operations::execution::{self as e, Entry, Failure, Representation::Text};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Canonical capitals: printed u/V -> V and long s -> S; no implicit aliases.
pub const ALPHABET: &str = "ABCDEFGHILMNOPQRSTVXYZ";
// Left rotations of NOPQRSTVXYZ under AB,CD,EF,GH,IL,MN,OP,QR,ST,VX,YZ.
// Numeric correspondences transcribed from La cifra (1553), BEIC IE2960948.
const ROTATIONS: [u8; 11] = [0, 6, 10, 5, 9, 4, 8, 3, 2, 7, 1];
const OP: &str = "cipher.bellaso";

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bellaso1553 {
    /// Nonempty repeated countersign in the exact 22-symbol canonical alphabet.
    #[schemars(
        length(min = 1, max = 65536),
        regex(pattern = r"^[ABCDEFGHILMNOPQRSTVXYZ]+$")
    )]
    pub key: String,
    /// Preserved scalars remain exact and never consume a key symbol.
    #[serde(default)]
    pub unmapped: Unmapped,
}

/// Reciprocal fixed 1553 profile, resetting at the first key symbol each call.
/// Text/output <=1 MiB, key <=64 KiB. Canonical capitals only unless unmapped
/// preservation is explicit. No normalization, in-band separators or line resets.
pub fn transform(text: &str, p: &Bellaso1553) -> Result<String, OperationError> {
    bound(OP, "text", text.len(), MAX_INPUT)?;
    bound(OP, "key", p.key.len(), MAX_KEY)?;
    if p.key.is_empty() {
        return Err(invalid(
            OP,
            "key",
            "a nonempty canonical countersign is required",
        ));
    }
    let shifts = p
        .key
        .bytes()
        .map(|b| {
            ALPHABET
                .bytes()
                .position(|c| c == b)
                .map(|i| ROTATIONS[i / 2])
                .ok_or_else(|| {
                    invalid(
                        OP,
                        "key",
                        "key must use the exact 22-symbol uppercase alphabet",
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut cursor = 0;
    text.char_indices()
        .map(|(offset, c)| {
            let Some(x) = ALPHABET.chars().position(|a| a == c) else {
                return match p.unmapped {
                    Unmapped::Preserve => Ok(c),
                    Unmapped::Reject => Err(input_error(OP, offset)),
                };
            };
            let shift = usize::from(shifts[cursor]);
            cursor = (cursor + 1) % shifts.len();
            let y = if x < 11 {
                11 + (x + shift) % 11
            } else {
                (x - 11 + 11 - shift) % 11
            };
            Ok(char::from(ALPHABET.as_bytes()[y]))
        })
        .collect()
}

pub(crate) const EXECUTORS: &[Entry] = &[Entry::pure(OP, &[Text], &[Text], |i, p, _| {
    let p = parse_parameters(&p.to_string()).map_err(|error| Failure(error.code.as_str()))?;
    transform(e::text(i, "text")?, &p)
        .map(|text| serde_json::json!({"text": text}))
        .map_err(|error| Failure(error.code.as_str()))
})];
