//! Explicit one-bit-per-residue Goldwasser-Micali educational reference profile.
use super::{
    integer::{self, Factors, Limits, Report},
    *,
};
use crate::asymmetric::entropy::SystemRng;
use num_bigint::BigUint;
use num_integer::Integer;
use rand_core::TryCryptoRng;
use zeroize::{Zeroize, Zeroizing};
parameters!("goldwasser_micali_reference_v1");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicKey {
    n: String,
    y: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateKey {
    p: String,
    q: String,
    y: String,
}
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.p.zeroize();
        self.q.zeroize();
        self.y.zeroize();
    }
}
/// Public modulus/nonresidue candidate; public validation cannot prove its factors.
pub struct EncryptionSettings {
    n: BigUint,
    y: BigUint,
    limits: Limits,
}
/// Prime factors and a y proven nonresidue modulo both probable primes.
pub struct DecryptionSettings {
    f: Factors,
    limits: Limits,
}
redact!(EncryptionSettings, DecryptionSettings);
fn public(
    n: BigUint,
    y: BigUint,
    work: &mut integer::Work,
) -> Result<(BigUint, BigUint), OperationError> {
    let n = integer::modulus(n)?;
    work.spend(n.bits())?;
    if y <= BigUint::from(1u8)
        || y >= n
        || y.gcd(&n) != BigUint::from(1u8)
        || integer::jacobi(y.clone(), n.clone(), work)? != 1
    {
        return Err(invalid());
    }
    Ok((n, y))
}
impl EncryptionSettings {
    /// Require JSON {"n":"decimal","y":"decimal"}, with unit y and Jacobi+1.
    /// Correct public-key generation must also establish nonresiduosity privately.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PublicKey = integer::key(key)?;
        let (n, y) = public(
            integer::number(&k.n).map_err(|_| invalid())?,
            integer::number(&k.y).map_err(|_| invalid())?,
            &mut limits.work,
        )?;
        Ok(Self { n, y, limits })
    }
}
fn legendre(c: &BigUint, p: &BigUint, work: &mut integer::Work) -> Result<BigUint, OperationError> {
    integer::pow(c, &((p - BigUint::from(1u8)) / BigUint::from(2u8)), p, work)
}
impl DecryptionSettings {
    /// Require JSON {"p":"decimal","q":"decimal","y":"decimal"}; validate
    /// both factors and y's Legendre symbols before accepting the private key.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PrivateKey = integer::key(key)?;
        let f = integer::factors(&k.p, &k.q, &mut limits.work)?;
        let (_, y) = public(
            f.n.clone(),
            integer::number(&k.y).map_err(|_| invalid())?,
            &mut limits.work,
        )?;
        if legendre(&y, &f.p, &mut limits.work)? != &f.p - BigUint::from(1u8)
            || legendre(&y, &f.q, &mut limits.work)? != &f.q - BigUint::from(1u8)
        {
            return Err(invalid());
        }
        Ok(Self { f, limits })
    }
}
/// Encrypt explicit bit tokens "0"/"1", each as one decimal residue with fresh
/// uniformly sampled unit coins. Expansion is one modulus-sized token per bit.
pub fn encrypt(tokens: &[String], s: &EncryptionSettings) -> Result<Vec<String>, OperationError> {
    encrypt_with_rng(tokens, s, &mut SystemRng::bounded(MAX_ENTROPY))
}
fn encrypt_with_rng(
    tokens: &[String],
    s: &EncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        if !matches!(token.as_str(), "0" | "1") {
            return Err(failed());
        }
        let u = integer::unit(&s.n, rng, &mut work)?;
        let mut c = integer::pow(&u, &BigUint::from(2u8), &s.n, &mut work)?;
        if token == "1" {
            work.spend(s.n.bits())?;
            c = c * &s.y % &s.n;
        }
        let value = c.to_string();
        report.add(&value)?;
        output.push(value);
    }
    Ok(std::mem::take(&mut *output))
}
/// Recover one bit per unit ciphertext. Opposite or zero Legendre symbols fail;
/// successful plaintext is unauthenticated and no bytes/text are inferred.
pub fn decrypt(tokens: &[String], s: &DecryptionSettings) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let c = integer::number(token)?;
        if c >= s.f.n || c < BigUint::from(1u8) || c.gcd(&s.f.n) != BigUint::from(1u8) {
            return Err(failed());
        }
        let p = legendre(&c, &s.f.p, &mut work)?;
        let q = legendre(&c, &s.f.q, &mut work)?;
        let value = if p == BigUint::from(1u8) && q == BigUint::from(1u8) {
            "0"
        } else if p == &s.f.p - BigUint::from(1u8) && q == &s.f.q - BigUint::from(1u8) {
            "1"
        } else {
            return Err(failed());
        }
        .to_string();
        report.add(&value)?;
        output.push(value);
    }
    Ok(std::mem::take(&mut *output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_entropy_cannot_return_partial_ciphertext() {
        let settings = EncryptionSettings::parse(
            r#"{"profile":"goldwasser_micali_reference_v1"}"#,
            br#"{"n":"272953","y":"2"}"#,
        )
        .unwrap();
        assert!(
            encrypt_with_rng(
                &["0".into(), "1".into()],
                &settings,
                &mut SystemRng::bounded(0)
            )
            .is_err()
        );
    }
}
