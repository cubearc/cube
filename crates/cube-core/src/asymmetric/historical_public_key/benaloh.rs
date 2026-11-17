//! Prime-r Benaloh reference profile with bounded exact decryption table.
use super::{
    integer::{self, Factors, Limits, Report},
    *,
};
use crate::asymmetric::entropy::SystemRng;
use num_bigint::BigUint;
use num_integer::Integer;
use rand_core::TryCryptoRng;
use std::collections::BTreeMap;
use zeroize::{Zeroize, Zeroizing};
parameters!("benaloh_prime_r_reference_v1");
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicKey {
    n: String,
    y: String,
    r: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateKey {
    p: String,
    q: String,
    y: String,
    r: u16,
}
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.p.zeroize();
        self.q.zeroize();
        self.y.zeroize();
    }
}
/// Public candidate; valid generation must establish the private order condition.
pub struct EncryptionSettings {
    n: BigUint,
    y: BigUint,
    r: u16,
    limits: Limits,
}
/// Checked factors, exponent and an exact finite prime-r message table.
pub struct DecryptionSettings {
    n: BigUint,
    exponent: BigUint,
    table: BTreeMap<BigUint, String>,
    limits: Limits,
}
redact!(EncryptionSettings, DecryptionSettings);
fn prime_r(r: u16) -> bool {
    (3..=4093).contains(&r)
        && r % 2 == 1
        && (3..=r / 3)
            .step_by(2)
            .take_while(|d| u32::from(*d) * u32::from(*d) <= u32::from(r))
            .all(|d| !r.is_multiple_of(d))
}
fn public(
    n: BigUint,
    y: BigUint,
    r: u16,
    work: &mut integer::Work,
) -> Result<(BigUint, BigUint), OperationError> {
    let n = integer::modulus(n)?;
    work.spend(n.bits())?;
    if !prime_r(r) || y <= BigUint::from(1u8) || y >= n || y.gcd(&n) != BigUint::from(1u8) {
        return Err(invalid());
    }
    Ok((n, y))
}
impl EncryptionSettings {
    /// Parse {"n":"decimal","y":"decimal","r":odd_prime}, r<=4093.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PublicKey = integer::key(key)?;
        let (n, y) = public(
            integer::number(&k.n).map_err(|_| invalid())?,
            integer::number(&k.y).map_err(|_| invalid())?,
            k.r,
            &mut limits.work,
        )?;
        Ok(Self {
            n,
            y,
            r: k.r,
            limits,
        })
    }
}
fn order(
    f: &Factors,
    y: &BigUint,
    r: u16,
    work: &mut integer::Work,
) -> Result<(BigUint, BigUint), OperationError> {
    let r = BigUint::from(r);
    let one = BigUint::from(1u8);
    let pm = &f.p - &one;
    let qm = &f.q - &one;
    if &pm % &r != BigUint::from(0u8) || (&pm / &r).gcd(&r) != one || qm.gcd(&r) != one {
        return Err(invalid());
    }
    let exponent = &f.phi / &r;
    let x = integer::pow(y, &exponent, &f.n, work)?;
    if x == one || integer::pow(&x, &r, &f.n, work)? != one {
        return Err(invalid());
    }
    Ok((exponent, x))
}
impl DecryptionSettings {
    /// Validate prime-r key conditions, including exact order r, and build a
    /// bounded table. Composite-r compatibility and unbounded DLP are refused.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: Parameters = integer::config(document, key)?;
        let _ = p.profile;
        let mut limits = Limits::new(p.max_work, p.max_memory_bytes, key.len())?;
        let k: PrivateKey = integer::key(key)?;
        let f = integer::factors(&k.p, &k.q, &mut limits.work)?;
        let (_, y) = public(
            f.n.clone(),
            integer::number(&k.y).map_err(|_| invalid())?,
            k.r,
            &mut limits.work,
        )?;
        let (exponent, x) = order(&f, &y, k.r, &mut limits.work)?;
        let reservation = (f.n.bits().div_ceil(8) as usize + 96) * usize::from(k.r);
        limits.reserve_lookup(reservation)?;
        let mut table = BTreeMap::new();
        let mut value = BigUint::from(1u8);
        for m in 0..k.r {
            limits.work.spend(f.n.bits())?;
            if table.insert(value.clone(), m.to_string()).is_some() {
                return Err(invalid());
            }
            value = value * &x % &f.n;
        }
        if value != BigUint::from(1u8) {
            return Err(invalid());
        }
        Ok(Self {
            n: f.n,
            exponent,
            table,
            limits,
        })
    }
}
/// Encrypt exact decimal m in0..r-1 with fresh uniformly sampled unit coins.
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
    let r = BigUint::from(s.r);
    for token in tokens {
        let m = integer::number(token)?;
        if m >= r {
            return Err(failed());
        }
        let u = integer::unit(&s.n, rng, &mut work)?;
        let c = integer::pow(&s.y, &m, &s.n, &mut work)? * integer::pow(&u, &r, &s.n, &mut work)?
            % &s.n;
        let value = c.to_string();
        report.add(&value)?;
        output.push(value);
    }
    Ok(std::mem::take(&mut *output))
}
/// Recover exact decimal messages using only the prevalidated finite table.
/// Results are unauthenticated; malformed/out-of-table ciphertexts fail wholly.
pub fn decrypt(tokens: &[String], s: &DecryptionSettings) -> Result<Vec<String>, OperationError> {
    let size = integer::tokens(tokens, s.limits)?;
    let mut work = s.limits.work;
    let mut report = Report::new(s.limits, size);
    let mut output = Zeroizing::new(Vec::new());
    for token in tokens {
        let c = integer::number(token)?;
        if c < BigUint::from(1u8) || c >= s.n || c.gcd(&s.n) != BigUint::from(1u8) {
            return Err(failed());
        }
        let index = integer::pow(&c, &s.exponent, &s.n, &mut work)?;
        let value = s.table.get(&index).ok_or_else(failed)?;
        report.add(value)?;
        output.push(value.clone());
    }
    Ok(std::mem::take(&mut *output))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_entropy_cannot_return_partial_ciphertext() {
        let settings = EncryptionSettings::parse(
            r#"{"profile":"benaloh_prime_r_reference_v1"}"#,
            br#"{"n":"91","y":"2","r":3}"#,
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
