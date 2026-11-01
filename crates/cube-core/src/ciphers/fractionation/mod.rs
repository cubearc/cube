//! Explicit historical coordinate fractionation. Not modern cryptography.
use crate::ciphers::transposition::wire;
mod catalog;
pub mod cm;
mod execution;
mod fixtures;
pub mod morse;
#[cfg(test)]
mod tests;
pub mod vic;
use crate::ciphers::{
    poly::{AZ, IJ, bound, invalid, letters, permutation},
    polybius::{Square, Token},
    transposition::{ColumnKey, apply, column_order},
};
use crate::errors::OperationError;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
const MAX_SYMBOLS: usize = 65536;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bifid {
    #[schemars(length(equal = 25))]
    pub square: String,
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub period: usize,
}
fn square(value: &str, op: &'static str) -> Result<Square, OperationError> {
    permutation(value, IJ, op, "square")?;
    Square::new(5, value, &[('J', 'I')]).map_err(|e| e.operation(op))
}
fn canonical(text: &str, op: &'static str) -> Result<String, OperationError> {
    Ok(letters(text, MAX_SYMBOLS, op, true)?
        .into_iter()
        .map(char::from)
        .collect())
}
fn pairs(
    sq: &Square,
    text: &str,
    base: u8,
    op: &'static str,
) -> Result<Vec<[u8; 2]>, OperationError> {
    let text = canonical(text, op)?;
    sq.encode(&text, base)
        .map(|v| {
            v.into_iter()
                .map(|p| p.expect("canonical text has no spaces"))
                .collect()
        })
        .map_err(|e| e.operation(op))
}
fn period(n: usize, op: &'static str) -> Result<(), OperationError> {
    if !(1..=4096).contains(&n) {
        return Err(invalid(op, "period", "period must be 1..4096"));
    }
    Ok(())
}
/// Row stream then column stream per period; final short group uses its own length.
pub fn bifid(text: &str, p: &Bifid, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.bifid.decode"
    } else {
        "cipher.bifid.encode"
    };
    period(p.period, op)?;
    let sq = square(&p.square, op)?;
    bifid_squares(text, p.period, &sq, &sq, decode, op)
}
fn bifid_squares(
    text: &str,
    period: usize,
    from: &Square,
    to: &Square,
    decode: bool,
    op: &'static str,
) -> Result<String, OperationError> {
    let coords = pairs(from, text, 0, op)?;
    let mut output = Vec::with_capacity(coords.len());
    for group in coords.chunks(period) {
        let n = group.len();
        if decode {
            let stream: Vec<_> = group.iter().flat_map(|v| v.iter().copied()).collect();
            for i in 0..n {
                output.push(Some([stream[i], stream[n + i]]));
            }
        } else {
            let stream: Vec<_> = (0..2)
                .flat_map(|d| group.iter().map(move |p| p[d]))
                .collect();
            output.extend(stream.as_chunks::<2>().0.iter().map(|v| Some(*v)));
        }
    }
    to.decode(&output, 0).map_err(|e| e.operation(op))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trifid {
    #[schemars(length(equal=27),extend("x-cube-max-utf8-bytes"=108))]
    pub alphabet: String,
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub period: usize,
}
/// Layer/row/column, row-major 3x3x3, explicit 27 distinct exact scalars.
pub fn trifid(text: &str, p: &Trifid, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.trifid.decode"
    } else {
        "cipher.trifid.encode"
    };
    period(p.period, op)?;
    bound(op, "alphabet", p.alphabet.len(), 108)?;
    let alphabet: Vec<_> = p.alphabet.chars().collect();
    let positions: std::collections::BTreeMap<_, _> =
        alphabet.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    if alphabet.len() != 27 || positions.len() != 27 {
        return Err(invalid(
            op,
            "alphabet",
            "cube needs exactly 27 distinct scalars",
        ));
    }
    let x = crate::ciphers::transposition::scalars(text, op)?;
    let mut coords = Vec::with_capacity(x.len());
    for c in x {
        let i = *positions
            .get(&c)
            .ok_or_else(|| invalid(op, "text", "symbol absent from cube"))?;
        coords.push([i / 9, i / 3 % 3, i % 3]);
    }
    let mut out = String::new();
    for group in coords.chunks(p.period) {
        let n = group.len();
        if decode {
            let stream: Vec<_> = group.iter().flat_map(|c| c.iter().copied()).collect();
            for i in 0..n {
                out.push(alphabet[stream[i] * 9 + stream[n + i] * 3 + stream[2 * n + i]]);
            }
        } else {
            let stream: Vec<_> = (0..3)
                .flat_map(|d| group.iter().map(move |c| c[d]))
                .collect();
            for triple in stream.as_chunks::<3>().0 {
                out.push(alphabet[triple[0] * 9 + triple[1] * 3 + triple[2]]);
            }
        }
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Adfgx {
    #[schemars(length(equal = 25))]
    pub square: String,
    pub key: ColumnKey,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Adfgvx {
    #[schemars(length(equal = 36))]
    pub square: String,
    pub key: ColumnKey,
}
fn adfg(
    text: &str,
    value: &str,
    key: &ColumnKey,
    labels: &[u8],
    decode: bool,
    op: &'static str,
) -> Result<String, OperationError> {
    let side = labels.len();
    permutation(
        value,
        if side == 5 {
            IJ
        } else {
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        },
        op,
        "square",
    )?;
    let sq = Square::new(
        side as u8,
        value,
        if side == 5 { &[('J', 'I')] } else { &[] },
    )
    .map_err(|e| e.operation(op))?;
    let key = key.order(op)?;
    if decode {
        bound(op, "text", text.len(), 2 * MAX_SYMBOLS)?;
        if !text.len().is_multiple_of(2) || !text.bytes().all(|c| labels.contains(&c)) {
            return Err(invalid(
                op,
                "text",
                "ciphertext requires an even number of uppercase coordinate labels",
            ));
        }
        let x: Vec<_> = text.chars().collect();
        let raw = apply(&x, &column_order(x.len(), &key), true);
        let coords: Vec<Token> = raw
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| {
                Some([
                    labels
                        .iter()
                        .position(|&c| c == p[0])
                        .expect("validated label") as u8,
                    labels
                        .iter()
                        .position(|&c| c == p[1])
                        .expect("validated label") as u8,
                ])
            })
            .collect();
        sq.decode(&coords, 0).map_err(|e| e.operation(op))
    } else {
        bound(op, "text", text.len(), MAX_SYMBOLS)?;
        if !text
            .bytes()
            .all(|b| b.is_ascii_uppercase() || (side == 6 && b.is_ascii_digit()))
        {
            return Err(invalid(
                op,
                "text",
                "canonical uppercase plaintext/digits required; no stripping",
            ));
        }
        let coords = sq.encode(text, 0).map_err(|e| e.operation(op))?;
        let x: Vec<_> = coords
            .into_iter()
            .flat_map(|p| {
                let [r, c] = p.expect("canonical no spaces");
                [
                    char::from(labels[r as usize]),
                    char::from(labels[c as usize]),
                ]
            })
            .collect();
        Ok(apply(&x, &column_order(x.len(), &key), false))
    }
}
/// I/J-merged 5x5 coordinate substitution followed by incomplete columnar.
pub fn adfgx(text: &str, p: &Adfgx, decode: bool) -> Result<String, OperationError> {
    adfg(
        text,
        &p.square,
        &p.key,
        b"ADFGX",
        decode,
        if decode {
            "cipher.adfgx.decode"
        } else {
            "cipher.adfgx.encode"
        },
    )
}
/// Explicit A-Z/0-9 square followed by incomplete columnar; J is independent.
pub fn adfgvx(text: &str, p: &Adfgvx, decode: bool) -> Result<String, OperationError> {
    adfg(
        text,
        &p.square,
        &p.key,
        b"ADFGVX",
        decode,
        if decode {
            "cipher.adfgvx.decode"
        } else {
            "cipher.adfgvx.encode"
        },
    )
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Nihilist {
    #[schemars(length(equal = 25))]
    pub square: String,
    #[schemars(length(min = 1, max = 4096))]
    pub key: String,
}
fn nihilist_key(p: &Nihilist, op: &'static str) -> Result<(Square, Vec<u16>), OperationError> {
    let sq = square(&p.square, op)?;
    if p.key.is_empty() {
        return Err(invalid(op, "key", "key must be nonempty canonical letters"));
    }
    bound(op, "key", p.key.len(), 4096)?;
    let key = pairs(&sq, &p.key, 1, op)?
        .into_iter()
        .map(|[r, c]| u16::from(r) * 10 + u16::from(c))
        .collect();
    Ok((sq, key))
}
/// Ordinary integer addition 22..110; explicitly does not wrap sums at 100.
pub fn nihilist_encode(text: &str, p: &Nihilist) -> Result<Vec<u16>, OperationError> {
    let op = "cipher.nihilist_substitution.encode";
    let (sq, key) = nihilist_key(p, op)?;
    Ok(pairs(&sq, text, 1, op)?
        .iter()
        .enumerate()
        .map(|(i, [r, c])| u16::from(*r) * 10 + u16::from(*c) + key[i % key.len()])
        .collect())
}
/// Decimal token boundaries are semantic; differences must be valid coordinates.
pub fn nihilist_decode(tokens: &[u16], p: &Nihilist) -> Result<String, OperationError> {
    let op = "cipher.nihilist_substitution.decode";
    bound(op, "tokens", tokens.len(), MAX_SYMBOLS)?;
    let (sq, key) = nihilist_key(p, op)?;
    let mut coords = Vec::with_capacity(tokens.len());
    for (i, &token) in tokens.iter().enumerate() {
        let v = token
            .checked_sub(key[i % key.len()])
            .filter(|&v| (11..=55).contains(&v) && v % 10 > 0 && v % 10 <= 5)
            .ok_or_else(|| {
                invalid(
                    op,
                    "tokens",
                    "sum minus repeating key must be an exact 11..55 square coordinate",
                )
            })?;
        coords.push(Some([(v / 10) as u8, (v % 10) as u8]));
    }
    sq.decode(&coords, 1).map_err(|e| e.operation(op))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bazeries {
    /// Decimal key 1..999999; every digit must be nonzero.
    #[schemars(range(min = 1, max = 999999))]
    #[serde(deserialize_with = "wire::integer")]
    pub key: u32,
    /// Explicit ACA ciphertext square, row-major. No locale-dependent spelling.
    #[schemars(length(equal = 25))]
    pub square: String,
}
/// ACA orientation: ordinary I/J plaintext alphabet goes down columns; keyed
/// ciphertext alphabet across rows. Reverse groups in repeating decimal sizes.
pub fn bazeries(text: &str, p: &Bazeries, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.bazeries.decode"
    } else {
        "cipher.bazeries.encode"
    };
    if !(1..=999999).contains(&p.key) || p.key.to_string().contains('0') {
        return Err(invalid(
            op,
            "key",
            "key must be 1..999999 with no zero digit",
        ));
    }
    let cells = permutation(&p.square, IJ, op, "square")?;
    let x = letters(text, MAX_SYMBOLS, op, true)?;
    let plain = IJ.as_bytes();
    let substitute = |c: u8| {
        let i = if decode {
            cells
                .iter()
                .position(|&v| v == c)
                .expect("canonical alphabet")
        } else {
            plain
                .iter()
                .position(|&v| v == c)
                .expect("canonical alphabet")
        };
        let transposed = i % 5 * 5 + i / 5;
        if decode {
            plain[transposed]
        } else {
            cells[transposed]
        }
    };
    let mut out = if decode {
        x.into_iter().map(substitute).collect::<Vec<_>>()
    } else {
        x
    };
    let digits: Vec<_> = p
        .key
        .to_string()
        .bytes()
        .map(|c| usize::from(c - b'0'))
        .collect();
    let mut pos = 0;
    let mut group = 0;
    while pos < out.len() {
        let end = (pos + digits[group % digits.len()]).min(out.len());
        out[pos..end].reverse();
        pos = end;
        group += 1;
    }
    if !decode {
        out = out.into_iter().map(substitute).collect();
    }
    Ok(out.into_iter().map(char::from).collect())
}

#[derive(Deserialize)]
struct DecimalToken(#[serde(deserialize_with = "wire::integer")] u16);
/// Bounded exact token transport, accepting integers or canonical decimal
/// strings as specified by the catalog; no concatenated-number guessing.
pub fn parse_nihilist_tokens(document: &str) -> Result<Vec<u16>, OperationError> {
    let values: Vec<DecimalToken> = crate::ciphers::poly::parse_parameters(document)?;
    bound(
        "cipher.nihilist_substitution.decode",
        "tokens",
        values.len(),
        MAX_SYMBOLS,
    )?;
    Ok(values.into_iter().map(|v| v.0).collect())
}
