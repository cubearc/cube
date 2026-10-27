//! Letter-only advancement preserves all other scalars exactly.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Key {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Gronsfeld {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[0-9]+$"))]
    pub key: String,
}
/// Signed endpoints are reduced modulo 26 before multiplication, avoiding overflow.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Trithemius {
    pub start: i32,
    pub step: i32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Feedback {
    Plaintext,
    Ciphertext,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Autokey {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub seed: String,
    pub feedback: Feedback,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct RunningKey {
    /// Caller-owned material; empty is valid only when no letters require a key.
    #[schemars(length(max = 1048576), regex(pattern = r"^[A-Za-z]*$"))]
    pub key: String,
}
/// C=P+digit mod26; key repeats over ASCII letters only.
pub fn gronsfeld(text: &str, p: &Gronsfeld, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.gronsfeld.decode"
    } else {
        "cipher.gronsfeld.encode"
    };
    bound(op, "key", p.key.len(), MAX_KEY)?;
    if p.key.is_empty() || !p.key.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(
            op,
            "key",
            "Gronsfeld requires a nonempty ASCII digit key",
        ));
    }
    preserve(text, op, |i, x| {
        let k = p.key.as_bytes()[i % p.key.len()] - b'0';
        if decode {
            (x + 26 - k) % 26
        } else {
            (x + k) % 26
        }
    })
}
/// C=P+start+step*i mod26; i counts transformed ASCII letters from zero.
pub fn trithemius(text: &str, p: &Trithemius, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.trithemius.decode"
    } else {
        "cipher.trithemius.encode"
    };
    let start = p.start.rem_euclid(26) as u8;
    let step = p.step.rem_euclid(26) as usize;
    preserve(text, op, |i, x| {
        let k = ((usize::from(start) + step * (i % 26)) % 26) as u8;
        if decode {
            (x + 26 - k) % 26
        } else {
            (x + k) % 26
        }
    })
}
/// Seed feedback is local to this call; the ring stores at most 64 KiB.
pub fn autokey(text: &str, p: &Autokey, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.autokey.decode"
    } else {
        "cipher.autokey.encode"
    };
    let mut stream = letter_key(&p.seed, MAX_KEY, false, op)?;
    preserve(text, op, |i, x| {
        let n = i % stream.len();
        let k = stream[n];
        let plain = if decode { (x + 26 - k) % 26 } else { x };
        let cipher = if decode { x } else { (x + k) % 26 };
        stream[n] = match p.feedback {
            Feedback::Plaintext => plain,
            Feedback::Ciphertext => cipher,
        };
        if decode { plain } else { cipher }
    })
}
/// Running material never wraps, and nonletters do not consume it.
pub fn running_key(text: &str, p: &RunningKey, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.running_key.decode"
    } else {
        "cipher.running_key.encode"
    };
    let key = letter_key(&p.key, MAX_INPUT, true, op)?;
    bound(op, "text", text.len(), MAX_INPUT)?;
    if text.bytes().filter(u8::is_ascii_alphabetic).count() > key.len() {
        return Err(invalid(
            op,
            "key",
            "running key has insufficient ASCII letters; it never wraps",
        ));
    }
    preserve(text, op, |i, x| {
        let k = key[i];
        if decode {
            (x + 26 - k) % 26
        } else {
            (x + k) % 26
        }
    })
}
/// Reciprocal ACA Porta: key AB/CD/.../YZ selects one of thirteen tableaux.
pub fn porta(text: &str, p: &Key) -> Result<String, OperationError> {
    let op = "cipher.porta";
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    preserve(text, op, |i, x| {
        let k = key[i % key.len()] / 2;
        if x < 13 {
            13 + (x + k) % 13
        } else {
            (x - 13 + 13 - k) % 13
        }
    })
}
