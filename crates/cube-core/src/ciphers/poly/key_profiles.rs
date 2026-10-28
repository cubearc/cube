//! Explicit Vigenere schedules; ASCII letters advance the key and other scalars survive.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_RESETS: usize = 65536;

/// Restart the keyword before each specified zero-based ASCII-letter offset.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct InterruptedKey {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
    /// Strictly increasing offsets, each smaller than the number of input ASCII letters.
    #[schemars(length(max = 65536), with = "Vec<LetterOffset>")]
    pub resets: Vec<u32>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct LetterOffset(#[schemars(range(min = 0, max = 1048575))] u32);

/// Add increment * floor(letter_offset / key_length), modulo 26; the first group adds zero.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct ProgressiveKey {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
    pub increment: i32,
}

/// Biswas/Sakeef/Siddique, Research Square rs-58356/v1, section VI.B: a * (P + K).
/// This classical composition supplies no modern confidentiality or authentication.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Multiplicative {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
    #[schemars(range(min = 1, max = 25))]
    #[schemars(extend("enum" = [1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25]))]
    pub multiplier: u8,
}

fn shift(letter: u8, key: u8, decode: bool) -> u8 {
    if decode {
        (letter + 26 - key) % 26
    } else {
        (letter + key) % 26
    }
}

/// ACA Interrupted Key transform with caller-supplied resets; no random/word inference.
/// Empty input requires an empty schedule and a valid key. Offset zero is permitted.
/// Text <=1 MiB, key <=64 KiB, <=65536 resets; all validation precedes output.
pub fn interrupted_key(
    text: &str,
    p: &InterruptedKey,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.interrupted_key.decode"
    } else {
        "cipher.interrupted_key.encode"
    };
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    bound(op, "text", text.len(), MAX_INPUT)?;
    bound(op, "resets", p.resets.len(), MAX_RESETS)?;
    let letters = text.bytes().filter(u8::is_ascii_alphabetic).count();
    if p.resets.windows(2).any(|w| w[0] >= w[1])
        || p.resets.last().is_some_and(|&n| n as usize >= letters)
    {
        return Err(invalid(
            op,
            "resets",
            "resets must increase strictly and address existing ASCII letters",
        ));
    }
    let mut next = 0;
    let mut start = 0;
    preserve(text, op, |i, x| {
        if p.resets.get(next).is_some_and(|&n| n as usize == i) {
            start = i;
            next += 1;
        }
        shift(x, key[(i - start) % key.len()], decode)
    })
}

/// ACA Progressive Key: C=P+K[i mod L]+increment*floor(i/L) mod26, A=0.
/// Signed increments are reduced before multiplication; case/nonletters are preserved.
pub fn progressive_key(
    text: &str,
    p: &ProgressiveKey,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.progressive_key.decode"
    } else {
        "cipher.progressive_key.encode"
    };
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    let step = p.increment.rem_euclid(26) as usize;
    preserve(text, op, |i, x| {
        let extra = step * ((i / key.len()) % 26);
        let k = (usize::from(key[i % key.len()]) + extra) % 26;
        shift(x, k as u8, decode)
    })
}

/// Fixed rs-58356/v1 profile: C=a*(P+K) mod26; P=a^-1*C-K mod26, A=0.
/// Require a unit multiplier independently of the repeating additive letter key.
pub fn multiplicative(
    text: &str,
    p: &Multiplicative,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.vigenere_multiplicative.decode"
    } else {
        "cipher.vigenere_multiplicative.encode"
    };
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    let inverse = crate::ciphers::classical::validate_affine_keys(i32::from(p.multiplier), 0, op)
        .map_err(|e| e.parameter("multiplier"))? as u16;
    preserve(text, op, |i, x| {
        let k = key[i % key.len()];
        if decode {
            shift((u16::from(x) * inverse % 26) as u8, k, true)
        } else {
            (u16::from(shift(x, k, false)) * u16::from(p.multiplier) % 26) as u8
        }
    })
}
