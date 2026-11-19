//! Bounded integer predicates, congruences, factor searches, and recurrences.
use super::{Work, invalid, overflow, parameter, resource};
use crate::{
    errors::OperationError,
    numbers::{modular, primes},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Congruence {
    pub residue: u64,
    pub modulus: u64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CrtRequest {
    #[schemars(length(max = 256))]
    pub pairs: Vec<Congruence>,
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CrtReport {
    Solution {
        residue: u64,
        modulus: u64,
    },
    Inconsistent {
        index: usize,
        merged: Congruence,
        pair: Congruence,
        gcd: u64,
    },
}

/// Merge generalized CRT constraints in caller order.
pub fn crt(request: &CrtRequest) -> Result<CrtReport, OperationError> {
    if request.pairs.len() > 256 {
        return Err(resource("CRT accepts at most 256 pairs"));
    }
    if request.pairs.iter().any(|p| p.modulus == 0) {
        return Err(parameter("CRT modulus must be positive"));
    }
    let mut merged = Congruence {
        residue: 0,
        modulus: 1,
    };
    for (index, &pair) in request.pairs.iter().enumerate() {
        let b = pair.residue % pair.modulus;
        // Existing u64 extended Euclid performs fewer than 128 divisions.
        let bezout = modular::extended_gcd(merged.modulus, pair.modulus)?;
        let delta = i128::from(b) - i128::from(merged.residue);
        let g = i128::from(bezout.gcd);
        if delta % g != 0 {
            return Ok(CrtReport::Inconsistent {
                index,
                merged,
                pair,
                gcd: bezout.gcd,
            });
        }
        let reduced = pair.modulus / bezout.gcd;
        let modulus = merged.modulus.checked_mul(reduced).ok_or_else(overflow)?;
        let n = i128::from(reduced);
        let difference = (delta / g).rem_euclid(n) as u128;
        let inverse = bezout.x.rem_euclid(n) as u128;
        let offset = (difference * inverse % u128::from(reduced)) as u64;
        merged = Congruence {
            residue: merged.residue + merged.modulus * offset,
            modulus,
        };
    }
    Ok(CrtReport::Solution {
        residue: merged.residue,
        modulus: merged.modulus,
    })
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrimeMethod {
    #[default]
    TrialDivision,
    MillerRabin,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrimeRequest {
    pub value: u32,
    #[serde(default)]
    pub method: PrimeMethod,
}
#[derive(Serialize, JsonSchema)]
pub struct PrimeReport {
    pub method: PrimeMethod,
    pub prime: bool,
}

fn is_trial(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let last = n.isqrt();
    if n.is_multiple_of(2) {
        return n == 2;
    }
    (3..=last)
        .step_by(2)
        .all(|divisor| !n.is_multiple_of(divisor))
}

fn is_miller_rabin(n: u32) -> bool {
    if n < 4 || n.is_multiple_of(2) {
        return n == 2 || n == 3;
    }
    let powers = (n - 1).trailing_zeros();
    let odd = (n - 1) >> powers;
    let modulus = u64::from(n);
    [2_u64, 7, 61].into_iter().all(|base| {
        let base = base % modulus;
        if base == 0 {
            return true;
        }
        let mut x =
            modular::mod_pow(base, u64::from(odd), modulus).expect("nonzero fixed-width modulus");
        if x == 1 || x == modulus - 1 {
            return true;
        }
        (1..powers).any(|_| {
            x = x * x % modulus;
            x == modulus - 1
        })
    })
}

/// Exact primality of a u32 with a declared method.
pub fn primality(request: &PrimeRequest) -> Result<PrimeReport, OperationError> {
    let prime = match request.method {
        PrimeMethod::TrialDivision => is_trial(request.value),
        PrimeMethod::MillerRabin => is_miller_rabin(request.value),
    };
    Ok(PrimeReport {
        method: request.method,
        prime,
    })
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SieveMethod {
    #[default]
    Eratosthenes,
    Atkin,
}
fn prime_limit() -> usize {
    100_000
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SieveRequest {
    #[schemars(range(max = 1000000))]
    pub limit: u32,
    #[serde(default)]
    pub method: SieveMethod,
    #[serde(default = "prime_limit")]
    #[schemars(range(max = 100000))]
    pub max_results: usize,
}
#[derive(Serialize, JsonSchema)]
pub struct SieveReport {
    pub method: SieveMethod,
    pub primes: Vec<u32>,
}

fn atkin(limit: usize, work: &mut Work) -> Result<Vec<u8>, OperationError> {
    work.charge(limit + 1)?;
    let mut flags = vec![0; limit + 1];
    for p in [2, 3] {
        if p <= limit {
            flags[p] = 1;
        }
    }
    for x in 1..=limit.isqrt() {
        let xx = x * x;
        for y in 1..=limit.isqrt() {
            work.charge(3)?;
            let yy = y * y;
            let n = 4 * xx + yy;
            if n <= limit && matches!(n % 12, 1 | 5) {
                flags[n] ^= 1;
            }
            let n = 3 * xx + yy;
            if n <= limit && n % 12 == 7 {
                flags[n] ^= 1;
            }
            if x > y {
                let n = 3 * xx - yy;
                if n <= limit && n % 12 == 11 {
                    flags[n] ^= 1;
                }
            }
        }
    }
    for p in 5..=limit.isqrt() {
        work.charge(1)?;
        if flags[p] != 0 {
            for multiple in (p * p..=limit).step_by(p * p) {
                work.charge(1)?;
                flags[multiple] = 0;
            }
        }
    }
    Ok(flags)
}

/// List the complete ascending prime sequence through an inclusive bound.
pub fn sieve(request: &SieveRequest) -> Result<SieveReport, OperationError> {
    if request.limit > primes::MAX_BOUND || request.max_results > 100_000 {
        return Err(parameter("sieve limit exceeds its declared maximum"));
    }
    let limit = request.limit as usize;
    let mut work = Work(33_554_432);
    let flags = match request.method {
        SieveMethod::Eratosthenes => {
            let mut charged = 0;
            let marked = primes::eratosthenes(request.limit, &mut charged, "number.sieve")?;
            work.charge(charged as usize)?;
            marked
        }
        SieveMethod::Atkin => atkin(limit, &mut work)?,
    };
    let expected = match request.method {
        SieveMethod::Eratosthenes => 0,
        SieveMethod::Atkin => 1,
    };
    work.charge(flags.len())?;
    let mut primes = Vec::new();
    for (value, &flag) in flags.iter().enumerate().skip(2) {
        if flag == expected {
            if primes.len() == request.max_results {
                return Err(resource("prime list exceeds max_results"));
            }
            primes.push(value as u32);
        }
    }
    Ok(SieveReport {
        method: request.method,
        primes,
    })
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    pub x0: u64,
    pub c: u64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum FactorRequest {
    TrialDivision {
        value: u32,
    },
    PollardRho {
        value: u64,
        x0: u64,
        c: u64,
        #[schemars(range(max = 1000000))]
        steps: usize,
        #[serde(default)]
        #[schemars(length(max = 16))]
        retries: Vec<Seed>,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct Factor {
    pub prime: u32,
    pub multiplicity: u8,
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum FactorReport {
    TrialDivision {
        factors: Vec<Factor>,
    },
    PollardRho {
        factor: Option<u64>,
        complete: bool,
        stop_reason: &'static str,
        attempts: usize,
        steps: usize,
    },
}

fn trial_factors(mut value: u32) -> Vec<Factor> {
    let mut factors = Vec::new();
    let mut prime = 2;
    while prime <= value / prime {
        let mut multiplicity = 0;
        while value.is_multiple_of(prime) {
            value /= prime;
            multiplicity += 1;
        }
        if multiplicity != 0 {
            factors.push(Factor {
                prime,
                multiplicity,
            });
        }
        prime = if prime == 2 { 3 } else { prime + 2 };
    }
    if value > 1 {
        factors.push(Factor {
            prime: value,
            multiplicity: 1,
        });
    }
    factors
}

fn metered_gcd(mut a: u64, mut b: u64, work: &mut Work) -> Result<u64, OperationError> {
    while b != 0 {
        work.charge(1)?;
        (a, b) = (b, a % b);
    }
    Ok(a)
}
fn search_factor(
    n: u64,
    seed: Seed,
    steps: usize,
    visits: &mut usize,
    work: &mut Work,
) -> Result<Option<u64>, OperationError> {
    let (mut x, mut y) = (seed.x0 % n, seed.x0 % n);
    let c = u128::from(seed.c % n);
    let step = |x| ((u128::from(x) * u128::from(x) + c) % u128::from(n)) as u64;
    for _ in 0..steps {
        work.charge(3)?;
        x = step(x);
        y = step(step(y));
        *visits += 1;
        let d = metered_gcd(x.abs_diff(y), n, work)?;
        if d == n {
            break;
        }
        if d > 1 && d < n && n.is_multiple_of(d) {
            return Ok(Some(d));
        }
    }
    Ok(None)
}

/// Factor one u32 fully or search one bounded proper divisor of a u64.
pub fn factor(request: &FactorRequest) -> Result<FactorReport, OperationError> {
    match request {
        FactorRequest::TrialDivision { value: 0 } => {
            Err(invalid("zero has no finite prime factorization"))
        }
        FactorRequest::TrialDivision { value } => Ok(FactorReport::TrialDivision {
            factors: trial_factors(*value),
        }),
        FactorRequest::PollardRho {
            value,
            x0,
            c,
            steps,
            retries,
        } => {
            if *value < 4 || *steps > 1_000_000 || retries.len() > 16 {
                return Err(parameter(
                    "rho requires n >= 4, at most 1000000 steps per attempt and 16 retries",
                ));
            }
            let mut work = Work(16 << 20);
            let mut visits = 0;
            let mut attempts = 0;
            let mut factor = None;
            let mut reason = "no_factor";
            for seed in std::iter::once(Seed { x0: *x0, c: *c }).chain(retries.iter().copied()) {
                attempts += 1;
                match search_factor(*value, seed, *steps, &mut visits, &mut work) {
                    Ok(Some(found)) => {
                        factor = Some(found);
                        reason = "factor_found";
                        break;
                    }
                    Ok(None) => {}
                    Err(_) => {
                        reason = "work_limit";
                        break;
                    }
                }
            }
            Ok(FactorReport::PollardRho {
                factor,
                complete: factor.is_some(),
                stop_reason: reason,
                attempts,
                steps: visits,
            })
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "sequence", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecurrenceRequest {
    Arithmetic {
        first: i64,
        difference: i64,
        count: usize,
    },
    Geometric {
        first: i64,
        ratio: i64,
        count: usize,
    },
    Fibonacci {
        count: usize,
    },
    Lucas {
        count: usize,
    },
    Collatz {
        start: u64,
        steps: usize,
        retain: usize,
    },
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecurrenceReport {
    Signed {
        values: Vec<i64>,
    },
    Unsigned {
        values: Vec<u64>,
    },
    Collatz {
        values: Vec<u64>,
        steps: usize,
        complete: bool,
        stop_reason: &'static str,
        omitted: usize,
    },
}

fn prefix(count: usize) -> Result<(), OperationError> {
    if count > 4096 {
        return Err(resource("recurrence prefix exceeds 4096 entries"));
    }
    Ok(())
}
fn signed_prefix(
    first: i64,
    second: i64,
    count: usize,
    multiply: bool,
) -> Result<RecurrenceReport, OperationError> {
    prefix(count)?;
    let mut values = Vec::with_capacity(count);
    let mut current = i128::from(first);
    for index in 0..count {
        values.push(i64::try_from(current).map_err(|_| overflow())?);
        if index + 1 < count {
            current = if multiply {
                current * i128::from(second)
            } else {
                current + i128::from(second)
            };
        }
    }
    Ok(RecurrenceReport::Signed { values })
}
fn fib_prefix(first: u64, count: usize) -> Result<RecurrenceReport, OperationError> {
    prefix(count)?;
    let mut values = Vec::with_capacity(count);
    values.extend([first, 1].into_iter().take(count));
    while values.len() < count {
        let n = values.len();
        values.push(
            values[n - 2]
                .checked_add(values[n - 1])
                .ok_or_else(overflow)?,
        );
    }
    Ok(RecurrenceReport::Unsigned { values })
}
fn collatz(start: u64, limit: usize, retain: usize) -> Result<RecurrenceReport, OperationError> {
    if start == 0 || limit > 1_000_000 || retain > 4096 {
        return Err(parameter(
            "Collatz requires positive start, at most 1000000 steps and 4096 retained values",
        ));
    }
    let mut values = Vec::with_capacity(retain);
    let (mut value, mut checkpoint) = (start, start);
    let (mut block, mut since) = (1, 0);
    let mut steps = 0;
    let reason = loop {
        if values.len() < retain {
            values.push(value);
        }
        if value == 1 {
            break "terminal";
        }
        if steps != 0 && checkpoint == value {
            break "cycle";
        }
        if steps == limit {
            break "step_limit";
        }
        if since == block {
            checkpoint = value;
            block *= 2;
            since = 0;
        }
        let next = if value.is_multiple_of(2) {
            Some(value / 2)
        } else {
            value.checked_mul(3).and_then(|n| n.checked_add(1))
        };
        let Some(next) = next else {
            break "overflow";
        };
        value = next;
        since += 1;
        steps += 1;
    };
    let omitted = steps + 1 - values.len();
    Ok(RecurrenceReport::Collatz {
        values,
        steps,
        complete: reason == "terminal",
        stop_reason: reason,
        omitted,
    })
}

/// Generate a checked integer prefix or a bounded Collatz trajectory.
pub fn recurrence(request: &RecurrenceRequest) -> Result<RecurrenceReport, OperationError> {
    match *request {
        RecurrenceRequest::Arithmetic {
            first,
            difference,
            count,
        } => signed_prefix(first, difference, count, false),
        RecurrenceRequest::Geometric {
            first,
            ratio,
            count,
        } => signed_prefix(first, ratio, count, true),
        RecurrenceRequest::Fibonacci { count } => fib_prefix(0, count),
        RecurrenceRequest::Lucas { count } => fib_prefix(2, count),
        RecurrenceRequest::Collatz {
            start,
            steps,
            retain,
        } => collatz(start, steps, retain),
    }
}
