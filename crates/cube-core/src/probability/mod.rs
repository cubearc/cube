//! Exact finite supplied masses. No inference, sampling or public rational engine.
mod dice;
mod finite;
mod metadata;
use crate::errors::{ErrorCode, OperationError};
pub use dice::{Comparison, DiceModel, DicePair, Face, Point, SumReport, dice_compare, dice_sum};
pub use finite::{FiniteModel, FiniteReport, Outcome, OutcomeProbability, finite_analyze};
pub use metadata::OPERATIONS;
use serde::{Deserialize, Serialize};
pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_SUPPORT: usize = 4096;
pub const MAX_WORK: usize = 16 * 1024 * 1024;
pub const MAX_MEMORY: usize = 32 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Limits {
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct Fraction {
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub numerator: u128,
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub denominator: u128,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct SignedFraction {
    pub negative: bool,
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub numerator: u128,
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub denominator: u128,
}
fn decimal<S: serde::Serializer>(v: &u128, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}
fn weight<S: serde::Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}
fn read_weight<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Integer(u64),
        Text(String),
    }
    match Value::deserialize(d)? {
        Value::Integer(v) => Ok(v),
        Value::Text(s)
            if !s.is_empty()
                && s.bytes().all(|b| b.is_ascii_digit())
                && (s == "0" || !s.starts_with('0')) =>
        {
            s.parse().map_err(serde::de::Error::custom)
        }
        _ => Err(serde::de::Error::custom(
            "weight must be canonical unsigned u64 decimal",
        )),
    }
}
fn err(op: &'static str, code: ErrorCode, name: &'static str, msg: &str) -> OperationError {
    OperationError::new(code, msg).operation(op).parameter(name)
}
fn invalid(op: &'static str, name: &'static str, msg: &str) -> OperationError {
    err(op, ErrorCode::InvalidInput, name, msg)
}
fn resource(op: &'static str, name: &'static str, msg: &str) -> OperationError {
    err(op, ErrorCode::ResourceLimit, name, msg)
}
fn add(a: u128, b: u128, op: &'static str) -> Result<u128, OperationError> {
    a.checked_add(b)
        .ok_or_else(|| resource(op, "mass", "exact u128 mass overflow"))
}
fn mul(a: u128, b: u128, op: &'static str) -> Result<u128, OperationError> {
    a.checked_mul(b)
        .ok_or_else(|| resource(op, "mass", "exact u128 mass product overflow"))
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
fn fraction(n: u128, d: u128) -> Fraction {
    debug_assert!(d > 0);
    let g = gcd(n, d);
    Fraction {
        numerator: n / g,
        denominator: d / g,
    }
}
struct Budget {
    op: &'static str,
    limits: Limits,
    work: usize,
    memory: usize,
    report: usize,
}
impl Budget {
    fn new(
        op: &'static str,
        l: Limits,
        memory: usize,
        report: usize,
    ) -> Result<Self, OperationError> {
        if l.max_work > MAX_WORK
            || l.max_memory_bytes > MAX_MEMORY
            || l.max_report_bytes > MAX_REPORT
        {
            return Err(err(
                op,
                ErrorCode::InvalidParameter,
                "limits",
                "limits may only lower hard ceilings",
            ));
        }
        if memory > l.max_memory_bytes || report > l.max_report_bytes {
            return Err(resource(
                op,
                "limits",
                "conservative complete report/scratch exceeds budget",
            ));
        }
        Ok(Self {
            op,
            limits: l,
            work: 0,
            memory,
            report,
        })
    }
    fn charge(&mut self, n: usize) -> Result<(), OperationError> {
        self.work = self
            .work
            .checked_add(n)
            .filter(|n| *n <= self.limits.max_work)
            .ok_or_else(|| {
                resource(
                    self.op,
                    "max_work",
                    "exact finite-mass work budget exhausted",
                )
            })?;
        Ok(())
    }
}
fn parse<T: serde::de::DeserializeOwned>(s: &str, op: &'static str) -> Result<T, OperationError> {
    let v = crate::pipeline::json::parse_with_limits(s, MAX_INPUT, 8, 65536)
        .map_err(|e| e.error.operation(op))?;
    serde_json::from_value(v)
        .map_err(|e| invalid(op, "model", "invalid finite-mass schema").caused_by(e))
}
/// Unique-key bounded finite model JSON; unknown fields and lossy types fail.
pub fn parse_finite(s: &str) -> Result<FiniteModel, OperationError> {
    parse(s, "probability.finite.analyze")
}
/// Unique-key bounded array-of-dice model JSON.
pub fn parse_dice(s: &str) -> Result<DiceModel, OperationError> {
    parse(s, "probability.dice.sum")
}
/// Unique-key bounded left/right dice model JSON.
pub fn parse_pair(s: &str) -> Result<DicePair, OperationError> {
    parse(s, "probability.dice.compare")
}
