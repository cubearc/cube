//! Concrete Morse group tables. Padding evidence is always explicit.
use super::*;
use crate::ciphers::transposition::wire;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FractionatedMorse {
    #[schemars(length(equal = 26))]
    pub alphabet: String,
    /// Decode only: exact count of terminal x padding (0..2). None preserves
    /// the raw stream and returns no guessed plaintext.
    #[schemars(range(min = 0, max = 2))]
    #[serde(default, deserialize_with = "wire::optional")]
    pub padding_count: Option<u8>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Morbit {
    /// Digits 1..9 assigned to lexicographic pairs over dot,dash,x.
    #[schemars(length(equal = 9))]
    pub digits: String,
    #[schemars(range(min = 0, max = 1))]
    #[serde(default, deserialize_with = "wire::optional")]
    pub padding_count: Option<u8>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pollux {
    /// Classes for digits 0..9, using '.'/'-'/'x'; all classes required.
    #[schemars(length(equal = 10))]
    pub classes: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MorseReport {
    /// Canonical plaintext when determinable; null means unresolved terminal pad.
    #[schemars(length(max = 65536))]
    pub text: Option<String>,
    #[schemars(length(max = 65536))]
    pub ciphertext: String,
    /// Exact grouped dot/dash/x stream, including any reported terminal padding.
    #[schemars(length(max = 1048576))]
    pub morse: String,
    #[schemars(range(min = 0, max = 2))]
    pub padding_count: u8,
    pub padding_ambiguous: bool,
}
fn encode_stream(text: &str, op: &'static str) -> Result<String, OperationError> {
    bound(op, "text", text.len(), 65536)?;
    if text.starts_with(' ')
        || text.ends_with(' ')
        || text.contains("  ")
        || !text
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b' ')
    {
        return Err(invalid(
            op,
            "text",
            "canonical A-Z/0-9 and single interior spaces required",
        ));
    }
    let stream = crate::symbols::morse_encode(text)
        .map_err(|e| e.operation(op))?
        .replace(" / ", "xx")
        .replace(' ', "x");
    bound(op, "morse", stream.len(), 1048576)?;
    Ok(stream)
}
fn decode_stream(raw: &str, pad: usize, op: &'static str) -> Result<String, OperationError> {
    if pad > raw.len() || !raw.as_bytes()[raw.len() - pad..].iter().all(|&c| c == b'x') {
        return Err(invalid(
            op,
            "padding_count",
            "padding count does not match terminal x symbols",
        ));
    }
    let raw = &raw[..raw.len() - pad];
    if raw.starts_with('x') || raw.ends_with('x') || raw.contains("xxx") {
        return Err(invalid(
            op,
            "morse",
            "malformed separators after explicit padding removal",
        ));
    }
    let standard = raw.replace("xx", " / ").replace('x', " ");
    crate::symbols::morse_decode(&standard).map_err(|e| e.operation(op))
}
fn grouped(
    text: &str,
    table: &str,
    width: usize,
    pad: Option<u8>,
    decode: bool,
    op: &'static str,
) -> Result<MorseReport, OperationError> {
    if pad.is_some_and(|n| usize::from(n) >= width) {
        return Err(invalid(
            op,
            "padding_count",
            "padding count must be smaller than group width",
        ));
    }
    let table = table.as_bytes();
    if decode {
        bound(op, "text", text.len(), 65536)?;
        let mut raw = String::with_capacity(text.len() * width);
        for c in text.bytes() {
            let mut value = table
                .iter()
                .position(|&v| v == c)
                .ok_or_else(|| invalid(op, "text", "ciphertext symbol absent from table"))?;
            let mut group = vec![b'.'; width];
            for i in (0..width).rev() {
                group[i] = b".-x"[value % 3];
                value /= 3;
            }
            raw.extend(group.into_iter().map(char::from));
        }
        let plain = pad
            .map(|n| decode_stream(&raw, usize::from(n), op))
            .transpose()?;
        Ok(MorseReport {
            text: plain,
            ciphertext: text.into(),
            morse: raw,
            padding_count: pad.unwrap_or(0),
            padding_ambiguous: pad.is_none(),
        })
    } else {
        if pad.is_some() {
            return Err(invalid(
                op,
                "padding_count",
                "encode calculates padding; omit decode-only count",
            ));
        }
        let mut raw = encode_stream(text, op)?;
        let count = (width - raw.len() % width) % width;
        raw.extend(std::iter::repeat_n('x', count));
        bound(op, "ciphertext", raw.len() / width, 65536)?;
        let mut cipher = String::new();
        for group in raw.as_bytes().chunks(width) {
            let mut index = 0;
            for c in group {
                index = index * 3 + b".-x".iter().position(|v| v == c).expect("encoded stream");
            }
            if index >= table.len() {
                return Err(invalid(
                    op,
                    "morse",
                    "xxx has no Fractionated Morse table entry",
                ));
            }
            cipher.push(char::from(table[index]));
        }
        Ok(MorseReport {
            text: Some(text.into()),
            ciphertext: cipher,
            morse: raw,
            padding_count: count as u8,
            padding_ambiguous: false,
        })
    }
}
/// Lexicographic triples dot/dash/x, excluding xxx. Output alphabet is explicit.
pub fn fractionated_morse(
    text: &str,
    p: &FractionatedMorse,
    decode: bool,
) -> Result<MorseReport, OperationError> {
    let op = if decode {
        "cipher.fractionated_morse.decode"
    } else {
        "cipher.fractionated_morse.encode"
    };
    permutation(&p.alphabet, AZ, op, "alphabet")?;
    grouped(text, &p.alphabet, 3, p.padding_count, decode, op)
}
/// All nine Morse pairs have independent digit assignments; no homophonic choice.
pub fn morbit(text: &str, p: &Morbit, decode: bool) -> Result<MorseReport, OperationError> {
    let op = if decode {
        "cipher.morbit.decode"
    } else {
        "cipher.morbit.encode"
    };
    permutation(&p.digits, "123456789", op, "digits")?;
    grouped(text, &p.digits, 2, p.padding_count, decode, op)
}
/// Digits encode Morse classes round-robin in ascending digit order per class.
/// Decode accepts any owned digit and preserves its exact ciphertext evidence.
pub fn pollux(text: &str, p: &Pollux, decode: bool) -> Result<MorseReport, OperationError> {
    let op = if decode {
        "cipher.pollux.decode"
    } else {
        "cipher.pollux.encode"
    };
    if p.classes.len() != 10
        || !p.classes.bytes().all(|c| b".-x".contains(&c))
        || !b".-x".iter().all(|&c| p.classes.as_bytes().contains(&c))
    {
        return Err(invalid(
            op,
            "classes",
            "ten digit classes must contain dot,dash,x and no other symbol",
        ));
    }
    if decode {
        bound(op, "text", text.len(), 65536)?;
        if !text.bytes().all(|c| c.is_ascii_digit()) {
            return Err(invalid(
                op,
                "text",
                "Pollux ciphertext requires digits 0..9",
            ));
        }
        let raw: String = text
            .bytes()
            .map(|c| char::from(p.classes.as_bytes()[usize::from(c - b'0')]))
            .collect();
        let plain = decode_stream(&raw, 0, op)?;
        Ok(MorseReport {
            text: Some(plain),
            ciphertext: text.into(),
            morse: raw,
            padding_count: 0,
            padding_ambiguous: false,
        })
    } else {
        let raw = encode_stream(text, op)?;
        bound(op, "ciphertext", raw.len(), 65536)?;
        let mut counts = [0; 3];
        let mut cipher = String::new();
        let classes: Vec<Vec<usize>> = b".-x"
            .iter()
            .map(|&c| {
                p.classes
                    .bytes()
                    .enumerate()
                    .filter_map(|(i, v)| (c == v).then_some(i))
                    .collect()
            })
            .collect();
        for c in raw.bytes() {
            let i = b".-x".iter().position(|&v| v == c).expect("encoded stream");
            let digits = &classes[i];
            cipher.push(char::from(b'0' + digits[counts[i] % digits.len()] as u8));
            counts[i] += 1;
        }
        Ok(MorseReport {
            text: Some(text.into()),
            ciphertext: cipher,
            morse: raw,
            padding_count: 0,
            padding_ambiguous: false,
        })
    }
}
