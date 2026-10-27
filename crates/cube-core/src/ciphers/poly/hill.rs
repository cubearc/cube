//! Exact modular Hill matrices; Bareiss/adjugate arithmetic never uses floats.
use super::*;
use num_bigint::BigInt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Padding {
    pub symbol: char,
    #[schemars(range(min = 0, max = 7))]
    pub count: u8,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 65536))]
pub struct Hill {
    /// Exact unique scalars, no Unicode normalization; the modulus is its length.
    #[schemars(length(min = 2, max = 256))]
    pub alphabet: String,
    #[schemars(range(min = 2, max = 8))]
    pub dimension: u8,
    /// Row-major signed 32-bit entries, multiplied by column vectors.
    #[schemars(length(min = 4, max = 64))]
    pub matrix: Vec<i32>,
    /// Encode only. Decode returns every recovered symbol, including padding.
    pub padding: Option<Padding>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HillResult {
    #[schemars(length(max = 65536))]
    pub text: String,
    #[schemars(range(min = 0, max = 7))]
    pub padding_count: u8,
}
fn determinant(mut a: Vec<Vec<BigInt>>) -> BigInt {
    let n = a.len();
    if n == 0 {
        return BigInt::from(1);
    }
    let zero = BigInt::from(0);
    let mut previous = BigInt::from(1);
    let mut sign = 1;
    for k in 0..n - 1 {
        if a[k][k] == zero {
            let Some(row) = (k + 1..n).find(|&r| a[r][k] != zero) else {
                return zero;
            };
            a.swap(k, row);
            sign = -sign;
        }
        let pivot = a[k][k].clone();
        for i in k + 1..n {
            for j in k + 1..n {
                a[i][j] = (&a[i][j] * &pivot - &a[i][k] * &a[k][j]) / &previous;
            }
            a[i][k] = BigInt::from(0);
        }
        previous = pivot;
    }
    &a[n - 1][n - 1] * sign
}
fn residue(value: BigInt, modulus: usize) -> u64 {
    let m = BigInt::from(modulus);
    ((value % &m + &m) % m)
        .to_string()
        .parse()
        .expect("residue <=255")
}
/// <=65536 scalars, alphabet 2..256, dimension 2..8, <=64 KiB definition.
pub fn hill(text: &str, p: &Hill, decode: bool) -> Result<HillResult, OperationError> {
    let op = if decode {
        "cipher.hill.decode"
    } else {
        "cipher.hill.encode"
    };
    json_bound(p, MAX_KEY, op)?;
    let alphabet: Vec<char> = p.alphabet.chars().collect();
    let modulus = alphabet.len();
    let positions: BTreeMap<char, usize> = alphabet
        .iter()
        .copied()
        .enumerate()
        .map(|(i, c)| (c, i))
        .collect();
    if !(2..=256).contains(&modulus) || positions.len() != modulus {
        return Err(invalid(
            op,
            "alphabet",
            "alphabet must contain 2..256 unique exact scalars",
        ));
    }
    let n = usize::from(p.dimension);
    if !(2..=8).contains(&n) || p.matrix.len() != n * n {
        return Err(invalid(
            op,
            "matrix",
            "dimension must be 2..8 with exactly dimension squared row-major entries",
        ));
    }
    let matrix: Vec<Vec<BigInt>> = p
        .matrix
        .chunks_exact(n)
        .map(|row| {
            row.iter()
                .map(|&v| BigInt::from(i64::from(v).rem_euclid(modulus as i64)))
                .collect()
        })
        .collect();
    let det = residue(determinant(matrix.clone()), modulus);
    let inverse = crate::numbers::modular::mod_inverse(det, modulus as u64).map_err(|_| {
        invalid(
            op,
            "matrix",
            "matrix determinant must be coprime to the alphabet modulus",
        )
    })?;
    let transform: Vec<Vec<u64>> = if decode {
        (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        let minor = matrix
                            .iter()
                            .enumerate()
                            .filter(|(r, _)| *r != j)
                            .map(|(_, row)| {
                                row.iter()
                                    .enumerate()
                                    .filter(|(c, _)| *c != i)
                                    .map(|(_, v)| v.clone())
                                    .collect()
                            })
                            .collect();
                        let signed = determinant(minor) * if (i + j) % 2 == 0 { 1 } else { -1 };
                        residue(signed, modulus) * inverse % modulus as u64
                    })
                    .collect()
            })
            .collect()
    } else {
        matrix
            .into_iter()
            .map(|row| row.into_iter().map(|v| residue(v, modulus)).collect())
            .collect()
    };
    bound(op, "text", text.len(), 4 * MAX_SYMBOLS)?;
    let mut input = Vec::new();
    for (offset, c) in text.char_indices() {
        input.push(*positions.get(&c).ok_or_else(|| input_error(op, offset))?);
    }
    bound(op, "symbols", input.len(), MAX_SYMBOLS)?;
    if let Some(pad) = &p.padding
        && (usize::from(pad.count) >= n || !positions.contains_key(&pad.symbol))
    {
        return Err(invalid(
            op,
            "padding",
            "padding requires a declared symbol and count smaller than dimension",
        ));
    }
    let count = if decode {
        0
    } else {
        p.padding.as_ref().map_or(0, |v| v.count)
    };
    if (input.len() + usize::from(count)) % n != 0 {
        return Err(invalid(
            op,
            "padding",
            "text plus explicit padding count must fill complete matrix blocks",
        ));
    }
    bound(
        op,
        "prepared_symbols",
        input.len() + usize::from(count),
        MAX_SYMBOLS,
    )?;
    if count > 0 {
        input.extend(std::iter::repeat_n(
            positions[&p.padding.as_ref().expect("count set").symbol],
            usize::from(count),
        ));
    }
    let mut out = String::new();
    for block in input.chunks_exact(n) {
        for row in &transform {
            let index = row
                .iter()
                .zip(block)
                .map(|(&a, &b)| a * b as u64)
                .sum::<u64>()
                % modulus as u64;
            out.push(alphabet[index as usize]);
        }
    }
    Ok(HillResult {
        text: out,
        padding_count: count,
    })
}
