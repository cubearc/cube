//! Canonical ACA tableau and column schedules.
use super::schedules::Key;
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Vigenere Slidefair, one key letter per adjacent digraph.
pub fn slidefair(text: &str, p: &Key, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.slidefair.decode"
    } else {
        "cipher.slidefair.encode"
    };
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    let x = letters(text, MAX_INPUT, op, false)?;
    if x.len() % 2 != 0 {
        return Err(invalid(
            op,
            "text",
            "Slidefair requires even canonical text; no implicit padding",
        ));
    }
    let mut out = String::with_capacity(x.len());
    for (i, pair) in x.as_chunks::<2>().0.iter().enumerate() {
        let k = key[i % key.len()];
        let a = pair[0] - b'A';
        let b = (pair[1] - b'A' + 26 - k) % 26;
        let (c, d) = if a == b {
            let shift = if decode { 25 } else { 1 };
            ((a + shift) % 26, (b + shift + k) % 26)
        } else {
            (b, (a + k) % 26)
        };
        out.push(char::from(b'A' + c));
        out.push(char::from(b'A' + d));
    }
    Ok(out)
}
/// Reciprocal Portax; final even short group uses the initial key letters.
pub fn portax(text: &str, p: &Key) -> Result<String, OperationError> {
    let op = "cipher.portax";
    let key = letter_key(&p.key, 4096, false, op)?;
    let x = letters(text, MAX_INPUT, op, false)?;
    if x.len() % 2 != 0 {
        return Err(invalid(op, "text", "Portax requires even canonical text"));
    }
    let mut out = Vec::with_capacity(x.len());
    for block in x.chunks(2 * key.len()) {
        let m = block.len() / 2;
        let mut top = Vec::with_capacity(m);
        let mut bottom = Vec::with_capacity(m);
        for j in 0..m {
            let k = key[j] / 2;
            let a = block[j] - b'A';
            let b = block[m + j] - b'A';
            let ar = a >= 13;
            let ac = if ar { (a - 13 + 13 - k) % 13 } else { a };
            let br = b % 2;
            let bc = (b / 2 + 13 - k) % 13;
            let (c, d) = if ac == bc {
                (
                    if ar { ac } else { 13 + (ac + k) % 13 },
                    2 * ((bc + k) % 13) + 1 - br,
                )
            } else {
                (
                    if ar { 13 + (bc + k) % 13 } else { bc },
                    2 * ((ac + k) % 13) + br,
                )
            };
            top.push(b'A' + c);
            bottom.push(b'A' + d);
        }
        out.extend(top);
        out.extend(bottom);
    }
    Ok(out.into_iter().map(char::from).collect())
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Nicodemus {
    #[schemars(length(min = 1, max = 4096), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
    #[serde(default = "default_depth")]
    #[schemars(range(min = 1, max = 64))]
    pub depth: u8,
}
fn default_depth() -> u8 {
    5
}
/// Stable alphabetical column ordering; each block takes depth rows per column.
pub fn nicodemus(text: &str, p: &Nicodemus, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.nicodemus.decode"
    } else {
        "cipher.nicodemus.encode"
    };
    let key = letter_key(&p.key, 4096, false, op)?;
    if !(1..=64).contains(&p.depth) {
        return Err(invalid(op, "depth", "depth must be 1..64"));
    }
    let x = letters(text, MAX_INPUT, op, false)?;
    let mut order: Vec<usize> = (0..key.len()).collect();
    order.sort_by_key(|&i| (key[i], i));
    let mut out = Vec::with_capacity(x.len());
    for block in x.chunks(key.len() * usize::from(p.depth)) {
        if decode {
            let mut restored = vec![0; block.len()];
            let mut cursor = 0;
            for &col in &order {
                for i in (col..block.len()).step_by(key.len()) {
                    restored[i] = b'A' + (block[cursor] - b'A' + 26 - key[col]) % 26;
                    cursor += 1;
                }
            }
            out.extend(restored);
        } else {
            for &col in &order {
                for i in (col..block.len()).step_by(key.len()) {
                    out.push(b'A' + (block[i] - b'A' + key[col]) % 26);
                }
            }
        }
    }
    Ok(out.into_iter().map(char::from).collect())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuagmireProfile {
    I,
    Ii,
    Iii,
    Iv,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Quagmire {
    pub profile: QuagmireProfile,
    #[schemars(length(equal = 26))]
    pub plain_alphabet: String,
    #[schemars(length(equal = 26))]
    pub cipher_alphabet: String,
    pub indicator: char,
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub key: String,
}
/// Indicator aligns each repeated key letter in the cipher alphabet.
pub fn quagmire(text: &str, p: &Quagmire, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.quagmire.decode"
    } else {
        "cipher.quagmire.encode"
    };
    let a = permutation(&p.plain_alphabet, AZ, op, "plain_alphabet")?;
    let b = permutation(&p.cipher_alphabet, AZ, op, "cipher_alphabet")?;
    match p.profile {
        QuagmireProfile::I if p.cipher_alphabet != AZ => {
            return Err(invalid(
                op,
                "cipher_alphabet",
                "Quagmire I requires the straight cipher alphabet",
            ));
        }
        QuagmireProfile::Ii if p.plain_alphabet != AZ => {
            return Err(invalid(
                op,
                "plain_alphabet",
                "Quagmire II requires the straight plain alphabet",
            ));
        }
        QuagmireProfile::Iii if a != b => {
            return Err(invalid(
                op,
                "cipher_alphabet",
                "Quagmire III requires identical mixed alphabets",
            ));
        }
        _ => {}
    }
    if !p.indicator.is_ascii_uppercase() {
        return Err(invalid(op, "indicator", "indicator must be uppercase A..Z"));
    }
    let pos = |alphabet: &[u8], c: u8| {
        alphabet
            .iter()
            .position(|&v| v == c)
            .expect("validated alphabet")
    };
    let anchor = pos(&a, p.indicator as u8);
    let key = letter_key(&p.key, MAX_KEY, false, op)?;
    let x = letters(text, MAX_INPUT, op, false)?;
    Ok(x.into_iter()
        .enumerate()
        .map(|(i, c)| {
            let shift = pos(&b, key[i % key.len()] + b'A');
            char::from(if decode {
                a[(pos(&b, c) + 26 + anchor - shift) % 26]
            } else {
                b[(pos(&a, c) + 26 - anchor + shift) % 26]
            })
        })
        .collect())
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PhillipsProfile {
    Rows,
    Columns,
    RowsColumns,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Phillips {
    #[schemars(length(equal = 25))]
    pub square: String,
    pub profile: PhillipsProfile,
}
const PERMS: [[usize; 5]; 8] = [
    [0, 1, 2, 3, 4],
    [1, 0, 2, 3, 4],
    [1, 2, 0, 3, 4],
    [1, 2, 3, 0, 4],
    [1, 2, 3, 4, 0],
    [2, 1, 3, 4, 0],
    [2, 3, 1, 4, 0],
    [2, 3, 4, 1, 0],
];
/// Eight five-letter states; rows/columns explicitly selected, no hidden merging.
pub fn phillips(text: &str, p: &Phillips, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.phillips.decode"
    } else {
        "cipher.phillips.encode"
    };
    let a = permutation(&p.square, IJ, op, "square")?;
    let x = letters(text, 327680, op, false)?;
    let mut out = String::with_capacity(x.len());
    for (i, c) in x.into_iter().enumerate() {
        let perm = PERMS[(i / 5) % 8];
        let rows = match p.profile {
            PhillipsProfile::Columns => [0, 1, 2, 3, 4],
            _ => perm,
        };
        let cols = match p.profile {
            PhillipsProfile::Rows => [0, 1, 2, 3, 4],
            _ => perm,
        };
        let idx = a
            .iter()
            .position(|&v| v == c)
            .ok_or_else(|| input_error(op, i))?;
        let r = rows
            .iter()
            .position(|&v| v == idx / 5)
            .expect("permutation");
        let col = cols
            .iter()
            .position(|&v| v == idx % 5)
            .expect("permutation");
        let delta = if decode { 4 } else { 1 };
        out.push(char::from(
            a[5 * rows[(r + delta) % 5] + cols[(col + delta) % 5]],
        ));
    }
    Ok(out)
}
