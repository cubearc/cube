//! Broken Merkle-Hellman subset-sum construction, educational reference only.
use super::{
    integer::{self, Limits, Report},
    *,
};
use num_bigint::BigUint;
use num_integer::Integer;
use std::collections::BTreeSet;
use zeroize::{Zeroize, Zeroizing};
parameters!("merkle_hellman_permuted_reference_v1");
pub const MAX_WEIGHTS: usize = 256;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicKey {
    weights: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateKey {
    weights: Vec<String>,
    modulus: String,
    multiplier: String,
    permutation: Vec<u16>,
}
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.weights.zeroize();
        self.modulus.zeroize();
        self.multiplier.zeroize();
        self.permutation.zeroize();
    }
}
/// Exact positive public weights; no inference from text or arbitrary knapsacks.
pub struct EncryptionSettings {
    weights: Vec<BigUint>,
    sum: BigUint,
    limits: Limits,
}
/// Explicit superincreasing sequence, invertible multiplier and full permutation.
pub struct DecryptionSettings {
    weights: Vec<BigUint>,
    public: Vec<BigUint>,
    sum: BigUint,
    modulus: BigUint,
    inverse: BigUint,
    permutation: Vec<usize>,
    limits: Limits,
}
redact!(EncryptionSettings, DecryptionSettings);
fn weights(
    tokens: &[String],
    superincreasing: bool,
    work: &mut integer::Work,
) -> Result<(Vec<BigUint>, BigUint), OperationError> {
    if !(2..=MAX_WEIGHTS).contains(&tokens.len()) {
        return Err(invalid());
    }
    let mut sum = BigUint::from(0u8);
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    for token in tokens {
        let value = integer::number(token).map_err(|_| invalid())?;
        work.spend(value.bits().max(1))?;
        if value.bits() == 0 || (superincreasing && value <= sum) || !seen.insert(value.clone()) {
            return Err(invalid());
        }
        sum += &value;
        if sum.bits() > MAX_BITS {
            return Err(limit());
        }
        values.push(value);
    }
    Ok((values, sum))
}
impl EncryptionSettings {
    /// Parse JSON {"weights":["decimal",...]}, with2..256 distinct positive values
    /// and a total subset sum fitting4096 bits. It does not solve the knapsack.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PublicKey = integer::key(key)?;
        let (weights, sum) = weights(&k.weights, false, &mut limits.work)?;
        Ok(Self {
            weights,
            sum,
            limits,
        })
    }
}
impl DecryptionSettings {
    /// Require positive superincreasing weights, modulus>sum, a coprime multiplier
    /// in2..modulus-1 and an explicit zero-based bijective permutation.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PrivateKey = integer::key(key)?;
        let (weights, total) = weights(&k.weights, true, &mut limits.work)?;
        let modulus = integer::number(&k.modulus).map_err(|_| invalid())?;
        let multiplier = integer::number(&k.multiplier).map_err(|_| invalid())?;
        limits.work.spend(4 * modulus.bits())?;
        if modulus <= total
            || multiplier < BigUint::from(2u8)
            || multiplier >= modulus
            || multiplier.gcd(&modulus) != BigUint::from(1u8)
            || k.permutation.len() != weights.len()
        {
            return Err(invalid());
        }
        let permutation = k
            .permutation
            .iter()
            .map(|&i| usize::from(i))
            .collect::<Vec<_>>();
        let unique = permutation.iter().copied().collect::<BTreeSet<_>>();
        if unique != (0..weights.len()).collect() {
            return Err(invalid());
        }
        let inverse = multiplier.modinv(&modulus).ok_or_else(invalid)?;
        let public = permutation
            .iter()
            .map(|&i| &multiplier * &weights[i] % &modulus)
            .collect::<Vec<_>>();
        let sum = public.iter().fold(BigUint::from(0u8), |a, b| a + b);
        if sum.bits() > MAX_BITS {
            return Err(limit());
        }
        Ok(Self {
            weights,
            public,
            sum,
            modulus,
            inverse,
            permutation,
            limits,
        })
    }
    /// Return the exact public weights in the selected permutation order.
    pub fn public_weights(&self) -> Vec<String> {
        self.public.iter().map(ToString::to_string).collect()
    }
}
fn subset(
    block: &str,
    weights: &[BigUint],
    work: &mut integer::Work,
) -> Result<BigUint, OperationError> {
    if block.len() != weights.len() || !block.bytes().all(|b| matches!(b, b'0' | b'1')) {
        return Err(failed());
    }
    let mut sum = BigUint::from(0u8);
    for (bit, weight) in block.bytes().zip(weights) {
        work.spend(weight.bits().max(1))?;
        if bit == b'1' {
            sum += weight;
        }
    }
    Ok(sum)
}
/// Encrypt fixed-width binary blocks as exact decimal subset sums, with no RNG.
/// Leading zero bits and the explicit public weight order remain meaningful.
pub fn encrypt(tokens: &[String], s: &EncryptionSettings) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let value = subset(token, &s.weights, &mut work)?;
        debug_assert!(value <= s.sum);
        let value = value.to_string();
        report.add(&value)?;
        output.push(value);
    }
    Ok(std::mem::take(&mut *output))
}
/// Undo the modular transform, use bounded private greedy decoding, then check
/// exact public re-encryption. Never search general subset sums or infer text.
pub fn decrypt(tokens: &[String], s: &DecryptionSettings) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let c = integer::number(token)?;
        if c > s.sum {
            return Err(failed());
        }
        work.spend(s.modulus.bits())?;
        let mut residue = &c * &s.inverse % &s.modulus;
        let mut bits = vec![b'0'; s.weights.len()];
        for (i, w) in s.weights.iter().enumerate().rev() {
            work.spend(w.bits().max(1))?;
            if &residue >= w {
                residue -= w;
                bits[i] = b'1';
            }
        }
        if residue.bits() != 0 {
            return Err(failed());
        }
        let block = s
            .permutation
            .iter()
            .map(|&i| char::from(bits[i]))
            .collect::<String>();
        if subset(&block, &s.public, &mut work)? != c {
            return Err(failed());
        }
        report.add(&block)?;
        output.push(block);
    }
    Ok(std::mem::take(&mut *output))
}
