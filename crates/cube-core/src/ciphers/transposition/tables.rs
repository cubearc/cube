//! Qualified ACA table permutations with explicit, reversible padding evidence.
use super::*;
pub(crate) mod fixtures;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cadenus {
    #[schemars(length(min = 1, max = 4096), regex(pattern = r"^[A-Z]+$"))]
    pub keyword: String,
    #[serde(default)]
    pub padding: Option<char>,
    #[serde(default, deserialize_with = "wire::optional")]
    #[schemars(range(min = 0, max = 65536))]
    pub original_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Swagman {
    /// A 4..8 square; each row and column permutes the one-based ranks.
    #[schemars(length(min = 4, max = 8), extend("items" = serde_json::json!({"type":"array","minItems":4,"maxItems":8,"uniqueItems":true,"items":{"type":"integer","minimum":1,"maximum":8}})))]
    pub key: Vec<Vec<u8>>,
    #[serde(default)]
    pub padding: Option<char>,
    #[serde(default, deserialize_with = "wire::optional")]
    #[schemars(range(min = 0, max = 65536))]
    pub original_length: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Convention {
    WritingIn,
    TakingOut,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Readout {
    Rows,
    Columns,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Nihilist {
    #[schemars(length(min = 1, max = 10), extend("uniqueItems" = true, "items" = serde_json::json!({"type":"integer","minimum":1,"maximum":10})))]
    pub key: Vec<u8>,
    pub convention: Convention,
    pub readout: Readout,
    #[serde(default)]
    pub padding: Option<char>,
    #[serde(default, deserialize_with = "wire::optional")]
    #[schemars(range(min = 0, max = 65536))]
    pub original_length: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BlockReport {
    #[schemars(length(max = 65536), extend("x-cube-max-utf8-bytes" = 262144))]
    pub text: String,
    #[schemars(range(min = 0, max = 65536))]
    pub original_length: usize,
    #[schemars(range(min = 0, max = 65535))]
    pub padding_count: usize,
}

fn prepared(
    text: &str,
    block: usize,
    padding: Option<char>,
    original: Option<usize>,
    decode: bool,
    op: &'static str,
) -> Result<Vec<char>, OperationError> {
    let mut x = scalars(text, op)?;
    let count = (block - x.len() % block) % block;
    if decode {
        if count != 0 {
            return Err(invalid(op, "text", "ciphertext must fill complete blocks"));
        }
    } else {
        if original.is_some_and(|n| n != x.len()) {
            return Err(invalid(
                op,
                "original_length",
                "encoding length must equal the exact source length",
            ));
        }
        bound(op, "cells", x.len() + count, MAX_SYMBOLS)?;
        if count != 0 {
            let pad = padding.ok_or_else(|| {
                invalid(
                    op,
                    "padding",
                    "an incomplete block requires an explicit pad scalar",
                )
            })?;
            x.resize(x.len() + count, pad);
        }
    }
    Ok(x)
}

fn finished(
    text: String,
    block: usize,
    padding: Option<char>,
    original: Option<usize>,
    source_length: usize,
    decode: bool,
    op: &'static str,
) -> Result<BlockReport, OperationError> {
    let mut x: Vec<_> = text.chars().collect();
    let n = if decode {
        original.unwrap_or(x.len())
    } else {
        source_length
    };
    if n > x.len() || x.len() - n >= block || (block - n % block) % block != x.len() - n {
        return Err(invalid(
            op,
            "original_length",
            "length must describe the minimum complete block",
        ));
    }
    let padding_count = x.len() - n;
    if decode && padding_count != 0 {
        let pad = padding
            .ok_or_else(|| invalid(op, "padding", "unpadding requires the declared scalar"))?;
        if x[n..].iter().any(|&c| c != pad) {
            return Err(invalid(
                op,
                "padding",
                "declared trailing padding does not match",
            ));
        }
        x.truncate(n);
    }
    Ok(BlockReport {
        text: x.into_iter().collect(),
        original_length: n,
        padding_count,
    })
}

/// Rotate each original 25-row column by its A,Z,...,B label, then read rows in
/// stable keyword order. V/W share a rotation only; plaintext scalars stay exact.
pub fn cadenus(text: &str, p: &Cadenus, decode: bool) -> Result<BlockReport, OperationError> {
    let op = if decode {
        "cipher.cadenus.decode"
    } else {
        "cipher.cadenus.encode"
    };
    bound(op, "keyword", p.keyword.len(), MAX_KEY)?;
    if p.keyword.is_empty() || !p.keyword.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(invalid(
            op,
            "keyword",
            "keyword must contain canonical uppercase A..Z labels",
        ));
    }
    let key = ColumnKey::Keyword {
        text: p.keyword.clone(),
    }
    .order(op)?;
    let block = 25 * key.len();
    bound(op, "cells", block, MAX_SYMBOLS)?;
    let x = prepared(text, block, p.padding, p.original_length, decode, op)?;
    let labels = b"AZYXWVUTSRQPONMLKJIHGFEDCB";
    let shifts: Vec<_> = p
        .keyword
        .bytes()
        .map(|b| {
            // Remove W from the row alphabet while retaining its V alias.
            let b = if b == b'W' { b'V' } else { b };
            labels
                .iter()
                .filter(|&&c| c != b'W')
                .position(|&c| c == b)
                .expect("validated label")
        })
        .collect();
    let order: Vec<_> = (0..x.len() / block)
        .flat_map(|b| {
            (0..25).flat_map({
                let key = &key;
                let shifts = &shifts;
                move |row| {
                    key.iter()
                        .map(move |&col| b * block + (row + shifts[col]) % 25 * key.len() + col)
                }
            })
        })
        .collect();
    finished(
        apply(&x, &order, decode),
        block,
        p.padding,
        p.original_length,
        text.chars().count(),
        decode,
        op,
    )
}

fn ranks(key: &[u8], n: usize, op: &'static str) -> Result<Vec<usize>, OperationError> {
    let order: Vec<_> = key
        .iter()
        .map(|&v| usize::from(v).wrapping_sub(1))
        .collect();
    validate_order(&order, n, op)
        .map_err(|_| invalid(op, "key", "ranks must permute the one-based domain"))?;
    Ok(order)
}

/// Row-fill a rectangle of key height; route each column by its periodic Latin
/// key column and read ciphertext columns. Partial key periods remain exact.
pub fn swagman(text: &str, p: &Swagman, decode: bool) -> Result<BlockReport, OperationError> {
    let op = if decode {
        "cipher.swagman.decode"
    } else {
        "cipher.swagman.encode"
    };
    let h = p.key.len();
    if !(4..=8).contains(&h) {
        return Err(invalid(op, "key", "Latin square height must be 4..8"));
    }
    for row in &p.key {
        ranks(row, h, op)?;
    }
    for col in 0..h {
        ranks(&p.key.iter().map(|r| r[col]).collect::<Vec<_>>(), h, op)?;
    }
    let inverse: Vec<_> = (0..h)
        .map(|col| {
            let mut rows = vec![0; h];
            for (r, row) in p.key.iter().enumerate() {
                rows[usize::from(row[col]) - 1] = r;
            }
            rows
        })
        .collect();
    let x = prepared(text, h, p.padding, p.original_length, decode, op)?;
    let width = x.len() / h;
    let order: Vec<_> = (0..width)
        .flat_map(|col| {
            let inverse = &inverse;
            (0..h).map(move |rank| inverse[col % h][rank] * width + col)
        })
        .collect();
    finished(
        apply(&x, &order, decode),
        h,
        p.padding,
        p.original_length,
        text.chars().count(),
        decode,
        op,
    )
}

/// The same one-based key permutes rows and columns of each square. Writing-in
/// maps source positions to key destinations; taking-out visits key positions.
pub fn nihilist(text: &str, p: &Nihilist, decode: bool) -> Result<BlockReport, OperationError> {
    let op = if decode {
        "cipher.nihilist_transposition.decode"
    } else {
        "cipher.nihilist_transposition.encode"
    };
    let n = p.key.len();
    if !(1..=10).contains(&n) {
        return Err(invalid(op, "key", "square side must be 1..10"));
    }
    let key = ranks(&p.key, n, op)?;
    let mut source = key.clone();
    if matches!(p.convention, Convention::WritingIn) {
        for (i, &j) in key.iter().enumerate() {
            source[j] = i;
        }
    }
    let block = n * n;
    let x = prepared(text, block, p.padding, p.original_length, decode, op)?;
    let order: Vec<_> = (0..x.len())
        .map(|i| {
            let offset = i % block;
            let (r, c) = if matches!(p.readout, Readout::Rows) {
                (offset / n, offset % n)
            } else {
                (offset % n, offset / n)
            };
            i / block * block + source[r] * n + source[c]
        })
        .collect();
    finished(
        apply(&x, &order, decode),
        block,
        p.padding,
        p.original_length,
        text.chars().count(),
        decode,
        op,
    )
}
