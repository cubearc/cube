//! Bounded exact integers, fallible sampling and maintained Miller-Rabin checks.
use super::*;
use crate::asymmetric::entropy::SystemRng;
use crypto_bigint::{BoxedUint, Odd};
use crypto_primes::hazmat::MillerRabin;
use num_bigint::BigUint;
use num_integer::Integer;
use rand_core::TryCryptoRng;
use zeroize::Zeroizing;
#[derive(Clone, Copy)]
pub(super) struct Work {
    maximum: u64,
    used: u64,
}
impl Work {
    pub(super) fn new(maximum: u64) -> Result<Self, OperationError> {
        if maximum == 0 || maximum > MAX_WORK {
            return Err(invalid());
        }
        Ok(Self { maximum, used: 0 })
    }
    pub(super) fn spend(&mut self, amount: u64) -> Result<(), OperationError> {
        self.used = self.used.checked_add(amount).ok_or_else(limit)?;
        if self.used > self.maximum {
            return Err(limit());
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub(super) work: Work,
    memory: u64,
    key_bytes: usize,
    lookup: usize,
}
impl Limits {
    pub(super) fn new(work: u64, memory: u64, key_bytes: usize) -> Result<Self, OperationError> {
        if memory == 0 || memory > MAX_MEMORY_BYTES {
            return Err(invalid());
        }
        let limits = Self {
            work: Work::new(work)?,
            memory,
            key_bytes,
            lookup: 0,
        };
        limits.memory(0, 0, 0)?;
        Ok(limits)
    }
    // Conservative owned-data reservation: codec/container copies, bigint scratch
    // and lookup storage. Caller/interpreter/process overhead is outside this cap.
    pub(super) fn reserve_lookup(&mut self, bytes: usize) -> Result<(), OperationError> {
        self.memory(0, 0, bytes)?;
        self.lookup = self.lookup.checked_add(bytes).ok_or_else(limit)?;
        Ok(())
    }
    pub(super) fn memory(
        self,
        input: usize,
        output: usize,
        lookup: usize,
    ) -> Result<(), OperationError> {
        let estimate = self
            .key_bytes
            .checked_mul(4)
            .and_then(|n| n.checked_add(65536))
            .and_then(|n| n.checked_add(input.checked_mul(8)?))
            .and_then(|n| n.checked_add(output.checked_mul(8)?))
            .and_then(|n| n.checked_add(self.lookup.checked_add(lookup)?.checked_mul(4)?))
            .ok_or_else(limit)?;
        if estimate as u64 > self.memory {
            return Err(limit());
        }
        Ok(())
    }
}
pub(super) fn config<T: serde::de::DeserializeOwned>(
    document: &str,
    key: &[u8],
) -> Result<T, OperationError> {
    if document.len() > MAX_PARAMETERS || key.len() > MAX_KEY_BYTES {
        return Err(limit());
    }
    serde_json::from_str(document).map_err(|_| invalid())
}
pub(super) fn key<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, OperationError> {
    if bytes.len() > MAX_KEY_BYTES {
        return Err(limit());
    }
    serde_json::from_slice(bytes).map_err(|_| invalid())
}
pub(super) fn number(text: &str) -> Result<BigUint, OperationError> {
    if text.len() > MAX_TOKEN_BYTES {
        return Err(limit());
    }
    if text.is_empty()
        || !text.bytes().all(|c| c.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(failed());
    }
    let value = BigUint::parse_bytes(text.as_bytes(), 10).ok_or_else(failed)?;
    if value.bits() > MAX_BITS {
        return Err(limit());
    }
    Ok(value)
}
pub(super) fn modulus(n: BigUint) -> Result<BigUint, OperationError> {
    if n < BigUint::from(15u8) || !n.bit(0) || n.bits() > MAX_BITS {
        return Err(invalid());
    }
    Ok(n)
}
pub(super) fn tokens(tokens: &[String], limits: Limits) -> Result<usize, OperationError> {
    if tokens.len() > MAX_TOKENS {
        return Err(limit());
    }
    let mut size = 2usize;
    for token in tokens {
        if token.len() > MAX_TOKEN_BYTES {
            return Err(limit());
        }
        size = size.checked_add(token.len() + 3).ok_or_else(limit)?;
    }
    if size > MAX_REPORT_BYTES {
        return Err(limit());
    }
    limits.memory(size, 0, 0)?;
    Ok(size)
}
pub(super) struct Report {
    limits: Limits,
    input: usize,
    bytes: usize,
}
impl Report {
    pub(super) fn new(limits: Limits, input: usize) -> Self {
        Self {
            limits,
            input,
            bytes: 64,
        }
    }
    pub(super) fn add(&mut self, token: &str) -> Result<(), OperationError> {
        self.bytes = self.bytes.checked_add(token.len() + 3).ok_or_else(limit)?;
        self.check()
    }
    pub(super) fn row(&mut self) -> Result<(), OperationError> {
        self.bytes = self.bytes.checked_add(3).ok_or_else(limit)?;
        self.check()
    }
    fn check(&self) -> Result<(), OperationError> {
        if self.bytes > MAX_REPORT_BYTES {
            return Err(limit());
        }
        self.limits.memory(self.input, self.bytes, 0)
    }
}
pub(super) fn pow(
    base: &BigUint,
    exponent: &BigUint,
    n: &BigUint,
    work: &mut Work,
) -> Result<BigUint, OperationError> {
    work.spend(exponent.bits().max(1))?;
    Ok(base.modpow(exponent, n))
}
pub(super) fn uniform(
    bound: &BigUint,
    rng: &mut impl TryCryptoRng,
    work: &mut Work,
) -> Result<BigUint, OperationError> {
    if bound.bits() == 0 {
        return Err(invalid());
    }
    if bound == &BigUint::from(1u8) {
        return Ok(BigUint::from(0u8));
    }
    let bits = bound.bits();
    let mut bytes = Zeroizing::new(vec![0; bits.div_ceil(8) as usize]);
    for _ in 0..128 {
        work.spend(bits)?;
        rng.try_fill_bytes(&mut bytes).map_err(|_| failed())?;
        if !bits.is_multiple_of(8) {
            bytes[0] &= (1u8 << (bits % 8)) - 1;
        }
        let candidate = BigUint::from_bytes_be(&bytes);
        if &candidate < bound {
            return Ok(candidate);
        }
    }
    Err(failed())
}
pub(super) fn unit(
    n: &BigUint,
    rng: &mut impl TryCryptoRng,
    work: &mut Work,
) -> Result<BigUint, OperationError> {
    for _ in 0..128 {
        let x = uniform(n, rng, work)?;
        work.spend(n.bits())?;
        if x >= BigUint::from(1u8) && x.gcd(n) == BigUint::from(1u8) {
            return Ok(x);
        }
    }
    Err(failed())
}
fn prime(n: &BigUint, rng: &mut impl TryCryptoRng, work: &mut Work) -> Result<(), OperationError> {
    if n == &BigUint::from(3u8) {
        return Ok(());
    }
    if n < &BigUint::from(3u8) || !n.bit(0) {
        return Err(invalid());
    }
    work.spend(128 * n.bits())?;
    let precision = (n.bits() as u32).next_multiple_of(64);
    let candidate = BoxedUint::from_be_slice(&n.to_bytes_be(), precision).map_err(|_| invalid())?;
    let test = MillerRabin::new(Odd::new(candidate).expect("odd candidate checked"));
    let bound = n - BigUint::from(3u8);
    for _ in 0..64 {
        let base = uniform(&bound, rng, work)? + BigUint::from(2u8);
        let base =
            BoxedUint::from_be_slice(&base.to_bytes_be(), precision).map_err(|_| invalid())?;
        if test.test(&base).is_composite() {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(super) struct Factors {
    pub(super) p: BigUint,
    pub(super) q: BigUint,
    pub(super) n: BigUint,
    pub(super) phi: BigUint,
}
pub(super) fn factors(p: &str, q: &str, work: &mut Work) -> Result<Factors, OperationError> {
    let p = number(p).map_err(|_| invalid())?;
    let q = number(q).map_err(|_| invalid())?;
    if p == q {
        return Err(invalid());
    }
    let n = modulus(&p * &q)?;
    let mut rng = SystemRng::bounded(MAX_ENTROPY);
    prime(&p, &mut rng, work)?;
    prime(&q, &mut rng, work)?;
    let phi = (&p - BigUint::from(1u8)) * (&q - BigUint::from(1u8));
    Ok(Factors { p, q, n, phi })
}
pub(super) fn jacobi(
    mut a: BigUint,
    mut n: BigUint,
    work: &mut Work,
) -> Result<i8, OperationError> {
    let mut sign = 1;
    a %= &n;
    while a.bits() != 0 {
        work.spend(n.bits().max(1))?;
        while !a.bit(0) {
            a >>= 1u32;
            let rem = (&n % BigUint::from(8u8))
                .to_u32_digits()
                .first()
                .copied()
                .unwrap_or(0);
            if rem == 3 || rem == 5 {
                sign = -sign;
            }
        }
        std::mem::swap(&mut a, &mut n);
        if a.bit(1) && n.bit(1) {
            sign = -sign;
        }
        a %= &n;
    }
    Ok(if n == BigUint::from(1u8) { sign } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::TryRng;
    struct FailedEntropy;
    impl TryRng for FailedEntropy {
        type Error = crate::asymmetric::entropy::EntropyError;
        fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), Self::Error> {
            Err(crate::asymmetric::entropy::EntropyError)
        }
        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Err(crate::asymmetric::entropy::EntropyError)
        }
        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Err(crate::asymmetric::entropy::EntropyError)
        }
    }
    impl TryCryptoRng for FailedEntropy {}
    struct RejectedEntropy {
        calls: usize,
    }
    impl TryRng for RejectedEntropy {
        type Error = crate::asymmetric::entropy::EntropyError;
        fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
            self.calls += 1;
            bytes.fill(255);
            Ok(())
        }
        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok(u32::MAX)
        }
        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok(u64::MAX)
        }
    }
    impl TryCryptoRng for RejectedEntropy {}
    #[test]
    fn primality_and_sampling_fail_cleanly_on_unavailable_entropy_or_bounded_rejection() {
        assert!(
            prime(
                &BigUint::from(499u16),
                &mut FailedEntropy,
                &mut Work::new(MAX_WORK).unwrap()
            )
            .is_err()
        );
        let mut rng = RejectedEntropy { calls: 0 };
        assert!(
            uniform(
                &BigUint::from(3u8),
                &mut rng,
                &mut Work::new(MAX_WORK).unwrap()
            )
            .is_err()
        );
        assert_eq!(rng.calls, 128);
    }
    #[test]
    fn exact_integer_work_memory_and_complete_report_boundaries_are_enforced() {
        for token in ["", "01", "-1", "+1", "1.0", "1e3", "١"] {
            assert!(number(token).is_err());
        }
        let max = (BigUint::from(1u8) << MAX_BITS) - BigUint::from(1u8);
        assert_eq!(number(&max.to_string()).unwrap(), max);
        assert!(number(&(BigUint::from(1u8) << MAX_BITS).to_string()).is_err());
        let mut work = Work::new(1).unwrap();
        work.spend(1).unwrap();
        assert!(work.spend(1).is_err());
        assert!(Work::new(0).is_err());
        assert!(Work::new(MAX_WORK + 1).is_err());
        assert!(Limits::new(MAX_WORK, 1, 0).is_err());
        assert!(Limits::new(MAX_WORK, MAX_MEMORY_BYTES + 1, 0).is_err());
        let limits = Limits::new(MAX_WORK, MAX_MEMORY_BYTES, 0).unwrap();
        assert!(tokens(&vec!["0".into(); MAX_TOKENS + 1], limits).is_err());
        let mut report = Report::new(limits, 0);
        report.bytes = MAX_REPORT_BYTES - 4;
        report.add("0").unwrap();
        assert!(report.add("0").is_err());
    }
    #[test]
    fn composite_duplicate_even_and_noncanonical_factors_are_refused() {
        for (p, q) in [
            ("561", "547"),
            ("499", "499"),
            ("2", "547"),
            ("SECRET", "547"),
            ("0499", "547"),
        ] {
            let error = factors(p, q, &mut Work::new(MAX_WORK).unwrap())
                .err()
                .unwrap();
            assert!(!format!("{error:?}").contains("SECRET"));
        }
    }
}
