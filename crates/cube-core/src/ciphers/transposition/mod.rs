//! Exact Unicode-scalar permutations. Historical ciphers provide no security.
mod catalog;
mod fixtures;
pub mod grille;
pub mod routes;
#[cfg(test)]
mod tests;
pub(crate) mod wire;
use crate::ciphers::poly::{bound, invalid};
use crate::errors::OperationError;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_SYMBOLS: usize = 65536;
pub const MAX_INPUT: usize = 1048576;
const MAX_KEY: usize = 4096;

pub(crate) fn scalars(text: &str, op: &'static str) -> Result<Vec<char>, OperationError> {
    bound(op, "text", text.len(), MAX_INPUT)?;
    let x: Vec<_> = text.chars().take(MAX_SYMBOLS + 1).collect();
    bound(op, "text", x.len(), MAX_SYMBOLS)?;
    Ok(x)
}
pub(crate) fn validate_order(
    order: &[usize],
    n: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    if order.len() != n {
        return Err(invalid(
            op,
            "order",
            "permutation length must equal the domain",
        ));
    }
    let mut seen = vec![false; n];
    for &i in order {
        if i >= n || seen[i] {
            return Err(invalid(
                op,
                "order",
                "indices must visit every position exactly once",
            ));
        }
        seen[i] = true;
    }
    Ok(())
}
pub(crate) fn apply(x: &[char], order: &[usize], decode: bool) -> String {
    if decode {
        let mut out = vec!['\0'; x.len()];
        for (i, &j) in order.iter().enumerate() {
            out[j] = x[i];
        }
        out.into_iter().collect()
    } else {
        order.iter().map(|&i| x[i]).collect()
    }
}
/// Keyword ties sort by Unicode scalar value then original column position.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ColumnKey {
    Keyword {
        #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=16384))]
        text: String,
    },
    Permutation {
        #[schemars(length(min = 1, max = 4096))]
        #[serde(deserialize_with = "wire::indices")]
        order: Vec<usize>,
    },
}
impl ColumnKey {
    pub(crate) fn order(&self, op: &'static str) -> Result<Vec<usize>, OperationError> {
        match self {
            Self::Keyword { text } => {
                bound(op, "key", text.len(), 4 * MAX_KEY)?;
                let chars: Vec<_> = text.chars().take(MAX_KEY + 1).collect();
                if chars.is_empty() {
                    return Err(invalid(op, "key", "keyword must be nonempty"));
                }
                bound(op, "key", chars.len(), MAX_KEY)?;
                let mut order: Vec<_> = (0..chars.len()).collect();
                order.sort_by_key(|&i| (chars[i], i));
                Ok(order)
            }
            Self::Permutation { order } => {
                if order.is_empty() {
                    return Err(invalid(op, "key", "column permutation must be nonempty"));
                }
                bound(op, "key", order.len(), MAX_KEY)?;
                validate_order(order, order.len(), op)?;
                Ok(order.clone())
            }
        }
    }
}
pub(crate) fn column_order(n: usize, key: &[usize]) -> Vec<usize> {
    key.iter()
        .flat_map(|&c| (c..n).step_by(key.len()))
        .collect()
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scytale {
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub columns: usize,
}
/// Row-major fill, column-major read; no padding, including a ragged last row.
pub fn scytale(text: &str, p: &Scytale, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.scytale.decode"
    } else {
        "cipher.scytale.encode"
    };
    let x = scalars(text, op)?;
    if !(1..=MAX_KEY).contains(&p.columns) {
        return Err(invalid(op, "columns", "columns must be 1..4096"));
    }
    Ok(apply(
        &x,
        &column_order(x.len(), &(0..p.columns).collect::<Vec<_>>()),
        decode,
    ))
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rails {
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub rails: usize,
    #[schemars(range(min = 0, max = 4095))]
    #[serde(deserialize_with = "wire::integer")]
    pub initial_rail: usize,
    pub direction: Direction,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Redefence {
    #[serde(flatten)]
    pub layout: Rails,
    #[schemars(length(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::indices")]
    pub order: Vec<usize>,
}
pub(crate) fn rail_order(
    n: usize,
    p: &Rails,
    readout: &[usize],
    op: &'static str,
) -> Result<Vec<usize>, OperationError> {
    if !(1..=MAX_KEY).contains(&p.rails) || p.initial_rail >= p.rails {
        return Err(invalid(
            op,
            "rails",
            "rails must be 1..4096 and initial rail must exist",
        ));
    }
    validate_order(readout, p.rails, op)?;
    let mut rows = vec![Vec::new(); p.rails];
    let mut rail = p.initial_rail;
    let mut down = matches!(p.direction, Direction::Down);
    for i in 0..n {
        rows[rail].push(i);
        if p.rails > 1 {
            if rail == 0 {
                down = true;
            } else if rail == p.rails - 1 {
                down = false;
            }
            rail = if down { rail + 1 } else { rail - 1 };
        }
    }
    Ok(readout
        .iter()
        .flat_map(|&r| rows[r].iter().copied())
        .collect())
}
/// Endpoint direction reflects before advancing; endpoints are visited once.
pub fn rail_fence(text: &str, p: &Rails, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.rail_fence.decode"
    } else {
        "cipher.rail_fence.encode"
    };
    let x = scalars(text, op)?;
    bound(op, "rails", p.rails, MAX_KEY)?;
    Ok(apply(
        &x,
        &rail_order(x.len(), p, &(0..p.rails).collect::<Vec<_>>(), op)?,
        decode,
    ))
}
/// Rail Fence layout with an explicit bijective rail readout.
pub fn redefence(text: &str, p: &Redefence, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.redefence.decode"
    } else {
        "cipher.redefence.encode"
    };
    let x = scalars(text, op)?;
    Ok(apply(
        &x,
        &rail_order(x.len(), &p.layout, &p.order, op)?,
        decode,
    ))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    #[schemars(length(max = 65536))]
    #[serde(deserialize_with = "wire::indices")]
    pub order: Vec<usize>,
}
/// Explicit output-to-input permutation of all scalar positions.
pub fn sequence(text: &str, p: &Sequence, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.sequence_transposition.decode"
    } else {
        "cipher.sequence_transposition.encode"
    };
    let x = scalars(text, op)?;
    validate_order(&p.order, x.len(), op)?;
    Ok(apply(&x, &p.order, decode))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Skip {
    #[serde(deserialize_with = "wire::integer")]
    pub start: i64,
    #[serde(deserialize_with = "wire::integer")]
    pub step: i64,
}
/// Affine circular permutation; empty/singleton domains accept all signed fields.
/// Steps are reduced before arithmetic, including i64::MIN. No removal occurs.
pub fn skip(text: &str, p: &Skip, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.skip.decode"
    } else {
        "cipher.skip.encode"
    };
    let x = scalars(text, op)?;
    let n = x.len();
    if n <= 1 {
        return Ok(text.into());
    }
    let start = p.start.rem_euclid(n as i64) as usize;
    let step = p.step.rem_euclid(n as i64) as usize;
    let (mut a, mut b) = (n, step);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    if a != 1 {
        return Err(invalid(
            op,
            "step",
            "step must be coprime with scalar length",
        ));
    }
    let order: Vec<_> = (0..n).map(|i| (start + i * step) % n).collect();
    Ok(apply(&x, &order, decode))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Columnar {
    pub key: ColumnKey,
}
/// Incomplete Columnar; stable ties and ragged lengths; never adds/removes padding.
pub fn columnar(text: &str, p: &Columnar, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.columnar.decode"
    } else {
        "cipher.columnar.encode"
    };
    let x = scalars(text, op)?;
    let key = p.key.order(op)?;
    Ok(apply(&x, &column_order(x.len(), &key), decode))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CompleteColumnar {
    pub key: ColumnKey,
    pub padding: Option<char>,
    /// Required for explicit unpadding; omitted means preserve all decoded symbols.
    #[schemars(range(min = 0, max = 65536))]
    #[serde(default, deserialize_with = "wire::optional")]
    pub original_length: Option<usize>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PaddingReport {
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub text: String,
    #[schemars(range(min = 0, max = 65536))]
    pub original_length: usize,
    #[schemars(range(min = 0, max = 4095))]
    pub padding_count: usize,
}
/// Complete rectangle. Decoding removes only an explicitly validated trailing pad.
pub fn complete_columnar(
    text: &str,
    p: &CompleteColumnar,
    decode: bool,
) -> Result<PaddingReport, OperationError> {
    let op = if decode {
        "cipher.complete_columnar.decode"
    } else {
        "cipher.complete_columnar.encode"
    };
    let mut x = scalars(text, op)?;
    let key = p.key.order(op)?;
    let width = key.len();
    let length = x.len();
    let pads = (width - length % width) % width;
    if decode {
        if pads != 0 {
            return Err(invalid(
                op,
                "text",
                "complete ciphertext length must be divisible by width",
            ));
        }
        let plain = apply(&x, &column_order(length, &key), true);
        x = plain.chars().collect();
        let original = p.original_length.unwrap_or(length);
        if original > length
            || length - original >= width
            || (width - original % width) % width != length - original
        {
            return Err(invalid(
                op,
                "original_length",
                "length must describe the minimum complete rectangle",
            ));
        }
        if original < length {
            let pad = p.padding.ok_or_else(|| {
                invalid(op, "padding", "unpadding requires the explicit pad scalar")
            })?;
            if x[original..].iter().any(|&c| c != pad) {
                return Err(invalid(
                    op,
                    "padding",
                    "declared padding does not match trailing plaintext",
                ));
            }
        }
        x.truncate(original);
        Ok(PaddingReport {
            text: x.into_iter().collect(),
            original_length: original,
            padding_count: length - original,
        })
    } else {
        if p.original_length.is_some_and(|n| n != length) {
            return Err(invalid(
                op,
                "original_length",
                "encode length must match source",
            ));
        }
        if pads > 0 {
            let c = p.padding.ok_or_else(|| {
                invalid(
                    op,
                    "padding",
                    "incomplete rectangle requires explicit padding",
                )
            })?;
            bound(op, "text", length + pads, MAX_SYMBOLS)?;
            x.resize(length + pads, c);
        }
        Ok(PaddingReport {
            text: apply(&x, &column_order(x.len(), &key), false),
            original_length: length,
            padding_count: pads,
        })
    }
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DoubleColumnar {
    pub first: ColumnKey,
    pub second: ColumnKey,
}
/// Two independent incomplete passes; decode reverses pass order.
pub fn double_columnar(
    text: &str,
    p: &DoubleColumnar,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.double_columnar.decode"
    } else {
        "cipher.double_columnar.encode"
    };
    let x = scalars(text, op)?;
    let a = p.first.order(op)?;
    let b = p.second.order(op)?;
    let (a, b) = if decode { (b, a) } else { (a, b) };
    let mid = apply(&x, &column_order(x.len(), &a), decode);
    let mid: Vec<_> = mid.chars().collect();
    Ok(apply(&mid, &column_order(mid.len(), &b), decode))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Myszkowski {
    #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=16384))]
    pub keyword: String,
}
/// Equal-ranked columns are read row by row, unlike stable-tie Columnar.
pub fn myszkowski(text: &str, p: &Myszkowski, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.myszkowski.decode"
    } else {
        "cipher.myszkowski.encode"
    };
    let x = scalars(text, op)?;
    bound(op, "keyword", p.keyword.len(), 4 * MAX_KEY)?;
    let key = ColumnKey::Keyword {
        text: p.keyword.clone(),
    }
    .order(op)?;
    let letters: Vec<_> = p.keyword.chars().collect();
    let width = letters.len();
    let mut order = Vec::with_capacity(x.len());
    let mut i = 0;
    while i < width {
        let mut end = i + 1;
        while end < width && letters[key[end]] == letters[key[i]] {
            end += 1;
        }
        for row in (0..x.len()).step_by(width) {
            for &c in &key[i..end] {
                if row + c < x.len() {
                    order.push(row + c);
                }
            }
        }
        i = end;
    }
    Ok(apply(&x, &order, decode))
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CellPattern {
    Aca,
    Continuous,
}
fn one() -> usize {
    1
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Amsco {
    pub key: ColumnKey,
    #[serde(default = "one")]
    #[schemars(range(min = 1, max = 2))]
    #[serde(deserialize_with = "wire::integer")]
    pub initial_cell: usize,
    pub pattern: CellPattern,
}
/// ACA alternates the first cell of every row; continuous alternates globally.
/// The final cell alone can truncate; its size is derived from total length.
pub fn amsco(text: &str, p: &Amsco, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.amsco.decode"
    } else {
        "cipher.amsco.encode"
    };
    let x = scalars(text, op)?;
    let key = p.key.order(op)?;
    if !(1..=2).contains(&p.initial_cell) {
        return Err(invalid(op, "initial_cell", "initial cell must be 1 or 2"));
    }
    let mut columns = vec![Vec::new(); key.len()];
    let mut pos = 0;
    let mut cell = 0;
    while pos < x.len() {
        let row = cell / key.len();
        let col = cell % key.len();
        let parity = match p.pattern {
            CellPattern::Aca => (row + col) % 2,
            CellPattern::Continuous => cell % 2,
        };
        let size = if parity == 0 {
            p.initial_cell
        } else {
            3 - p.initial_cell
        };
        let end = (pos + size).min(x.len());
        columns[col].extend(pos..end);
        pos = end;
        cell += 1;
    }
    let order: Vec<_> = key
        .iter()
        .flat_map(|&c| columns[c].iter().copied())
        .collect();
    Ok(apply(&x, &order, decode))
}
