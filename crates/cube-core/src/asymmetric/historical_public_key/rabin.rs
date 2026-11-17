//! Raw Rabin squaring and complete Blum-modulus root sets, for education only.
use super::{
    integer::{self, Factors, Limits, Report},
    *,
};
use num_bigint::BigUint;
use std::collections::BTreeSet;
use zeroize::{Zeroize, Zeroizing};
parameters!("rabin_blum_reference_v1");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicKey {
    n: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateKey {
    p: String,
    q: String,
}
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.p.zeroize();
        self.q.zeroize();
    }
}
/// Exact educational public modulus and per-call limits; not secure padding.
pub struct EncryptionSettings {
    n: BigUint,
    limits: Limits,
}
/// Validated distinct probable Blum primes; private bigint erasure is not assured.
pub struct DecryptionSettings {
    f: Factors,
    limits: Limits,
}
redact!(EncryptionSettings, DecryptionSettings);
impl EncryptionSettings {
    /// Parse the reference profile and a bounded JSON key {"n":"decimal"}.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PublicKey = integer::key(key)?;
        let n = integer::modulus(integer::number(&k.n).map_err(|_| invalid())?)?;
        Ok(Self { n, limits })
    }
}
impl DecryptionSettings {
    /// Parse {"p":"decimal","q":"decimal"}; require distinct probable primes
    /// both3 mod4 and a modulus of at most4096 bits. All64 MR rounds are bounded.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PrivateKey = integer::key(key)?;
        let f = integer::factors(&k.p, &k.q, &mut limits.work)?;
        if &f.p % BigUint::from(4u8) != BigUint::from(3u8)
            || &f.q % BigUint::from(4u8) != BigUint::from(3u8)
        {
            return Err(invalid());
        }
        Ok(Self { f, limits })
    }
}
/// Square exact canonical decimal messages in0..n-1. No randomness or padding.
pub fn encrypt(tokens: &[String], s: &EncryptionSettings) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let m = integer::number(token)?;
        if m >= s.n {
            return Err(failed());
        }
        let value = integer::pow(&m, &BigUint::from(2u8), &s.n, &mut work)?.to_string();
        report.add(&value)?;
        output.push(value);
    }
    Ok(std::mem::take(&mut *output))
}
fn roots(
    c: &BigUint,
    f: &Factors,
    work: &mut integer::Work,
) -> Result<Vec<BigUint>, OperationError> {
    let four = BigUint::from(4u8);
    let one = BigUint::from(1u8);
    let mp = integer::pow(c, &((&f.p + &one) / &four), &f.p, work)?;
    let mq = integer::pow(c, &((&f.q + &one) / &four), &f.q, work)?;
    if &mp * &mp % &f.p != c % &f.p || &mq * &mq % &f.q != c % &f.q {
        return Err(failed());
    }
    let qi = f.q.modinv(&f.p).ok_or_else(failed)?;
    let pi = f.p.modinv(&f.q).ok_or_else(failed)?;
    let a = &mp * &f.q * qi;
    let b = &mq * &f.p * pi;
    let r = (&a + &b) % &f.n;
    let t = (&a + &f.n - (&b % &f.n)) % &f.n;
    Ok(
        BTreeSet::from([r.clone(), (&f.n - &r) % &f.n, t.clone(), (&f.n - &t) % &f.n])
            .into_iter()
            .collect(),
    )
}
/// Return every numerically sorted distinct root for each ciphertext. Never
/// choose a root or infer text. Zero has one root; nonunits can have two.
pub fn decrypt(
    tokens: &[String],
    s: &DecryptionSettings,
) -> Result<Vec<Vec<String>>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let c = integer::number(token)?;
        if c >= s.f.n {
            return Err(failed());
        }
        let mut row = Vec::new();
        report.row()?;
        for root in roots(&c, &s.f, &mut work)? {
            let value = root.to_string();
            report.add(&value)?;
            row.push(value);
        }
        output.push(row);
    }
    Ok(std::mem::take(&mut *output))
}
