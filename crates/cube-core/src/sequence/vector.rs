//! Stable numeric comparisons over explicit nonnegative coordinates.
use super::*;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Weight {
    pub token: String,
    pub weight: f64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum VectorPair {
    Dense {
        left: Vec<f64>,
        right: Vec<f64>,
    },
    Sparse {
        left: Vec<Weight>,
        right: Vec<Weight>,
    },
}
#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VectorMethod {
    Cosine,
    Kl,
    Js,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogBase {
    #[default]
    Two,
    E,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VectorRequest {
    pub input: VectorPair,
    pub method: VectorMethod,
    #[serde(default)]
    pub log_base: LogBase,
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScoreStatus {
    Finite,
    Undefined,
    PositiveInfinity,
}
#[derive(Serialize, JsonSchema)]
pub struct VectorReport {
    pub method: VectorMethod,
    pub status: ScoreStatus,
    pub value: Option<f64>,
    pub reason: Option<&'static str>,
}

fn check(value: f64) -> bool {
    value.is_finite() && (0.0..=1e150).contains(&value)
}

fn sparse_valid(items: &[Weight]) -> bool {
    items.len() <= 10_000
        && items.iter().map(|p| p.token.len()).sum::<usize>() <= TEXT_BYTES
        && items.iter().all(|p| check(p.weight))
        && items.windows(2).all(|pair| pair[0].token < pair[1].token)
}

fn coordinates(input: &VectorPair) -> Result<Vec<(f64, f64)>, OperationError> {
    match input {
        VectorPair::Dense { left, right } => {
            if left.len() != right.len() {
                return Err(invalid("dense vectors must have equal lengths"));
            }
            if left.len() > 10_000 {
                return Err(resource("dense vectors exceed 10000 coordinates"));
            }
            if !left.iter().chain(right).copied().all(check) {
                return Err(invalid("weights must be finite and within 0..=1e150"));
            }
            Ok(left.iter().copied().zip(right.iter().copied()).collect())
        }
        VectorPair::Sparse { left, right } => {
            if !sparse_valid(left) || !sparse_valid(right) {
                return Err(invalid(
                    "sparse arrays require sorted unique keys, finite weights and bounded lengths",
                ));
            }
            serialized_bound(
                left,
                TEXT_BYTES,
                "left sparse coordinates exceed 1 MiB encoded",
            )?;
            serialized_bound(
                right,
                TEXT_BYTES,
                "right sparse coordinates exceed 1 MiB encoded",
            )?;
            let (mut a, mut b) = (left.iter().peekable(), right.iter().peekable());
            let mut out = Vec::new();
            loop {
                let pair = match (a.peek(), b.peek()) {
                    (None, None) => break,
                    (Some(l), Some(r)) if l.token == r.token => {
                        (a.next().unwrap().weight, b.next().unwrap().weight)
                    }
                    (Some(l), Some(r)) if l.token > r.token => (0.0, b.next().unwrap().weight),
                    (Some(_), _) => (a.next().unwrap().weight, 0.0),
                    (None, Some(_)) => (0.0, b.next().unwrap().weight),
                };
                if out.len() == 10_000 {
                    return Err(resource("merged sparse vectors exceed 10000 coordinates"));
                }
                out.push(pair);
            }
            Ok(out)
        }
    }
}

fn sum(terms: impl Iterator<Item = f64>) -> f64 {
    let (mut total, mut error) = (0.0, 0.0);
    for term in terms {
        let adjusted = term - error;
        let next = total + adjusted;
        error = (next - total) - adjusted;
        total = next;
    }
    total
}

fn scales(values: &[(f64, f64)]) -> (f64, f64) {
    values
        .iter()
        .fold((0.0, 0.0), |(a, b), &(x, y)| (a.max(x), b.max(y)))
}

pub(super) fn cosine(values: &[(f64, f64)]) -> Option<f64> {
    let (ax, ay) = scales(values);
    if ax == 0.0 || ay == 0.0 {
        return None;
    }
    let dot = sum(values.iter().map(|&(a, b)| (a / ax) * (b / ay)));
    let norm_a = sum(values.iter().map(|&(a, _)| (a / ax).powi(2)));
    let norm_b = sum(values.iter().map(|&(_, b)| (b / ay).powi(2)));
    Some((dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(0.0, 1.0))
}

fn log_mix(a: f64, b: f64) -> f64 {
    let hi = a.max(b);
    let lo = a.min(b);
    hi + (lo - hi).exp().ln_1p() - std::f64::consts::LN_2
}

fn divergence(
    values: &[(f64, f64)],
    method: VectorMethod,
    base: LogBase,
) -> Result<Option<f64>, OperationError> {
    let (ax, ay) = scales(values);
    if ax == 0.0 || ay == 0.0 {
        return Err(invalid(
            "information divergence requires positive mass in each distribution",
        ));
    }
    if matches!(method, VectorMethod::Kl) && values.iter().any(|&(a, b)| a > 0.0 && b == 0.0) {
        return Ok(None);
    }
    let norm_x = sum(values.iter().map(|&(a, _)| a / ax));
    let norm_y = sum(values.iter().map(|&(_, b)| b / ay));
    let (lx, ly) = (ax.ln() + norm_x.ln(), ay.ln() + norm_y.ln());
    let result = sum(values.iter().map(|&(a, b)| {
        if a == 0.0 && b == 0.0 {
            return 0.0;
        }
        let (p, q) = (a.ln() - lx, b.ln() - ly);
        let weighted = |numerator: f64, log_ratio: f64| {
            if numerator == 0.0 {
                0.0
            } else {
                numerator * log_ratio
            }
        };
        if matches!(method, VectorMethod::Kl) {
            return weighted((a / ax) / norm_x, p - q);
        }
        let midpoint = log_mix(p, q);
        (weighted((a / ax) / norm_x, p - midpoint) + weighted((b / ay) / norm_y, q - midpoint))
            / 2.0
    }));
    if !result.is_finite() {
        return Err(invalid(
            "information divergence did not produce a finite result",
        ));
    }
    let result = result.max(0.0)
        / match base {
            LogBase::Two => std::f64::consts::LN_2,
            LogBase::E => 1.0,
        };
    if !result.is_finite() {
        return Err(invalid(
            "information divergence did not produce a finite result",
        ));
    }
    Ok(Some(result))
}

/// Compute cosine or an explicit KL/JS divergence with finite JSON results.
pub fn vector_similarity(input: &VectorRequest) -> Result<VectorReport, OperationError> {
    let values = coordinates(&input.input)?;
    let (value, empty) = match input.method {
        VectorMethod::Cosine => (
            cosine(&values),
            (ScoreStatus::Undefined, Some("zero_vector")),
        ),
        method => (
            divergence(&values, method, input.log_base)?,
            (
                ScoreStatus::PositiveInfinity,
                Some("positive_mass_over_zero"),
            ),
        ),
    };
    let (status, reason) = if value.is_some() {
        (ScoreStatus::Finite, None)
    } else {
        empty
    };
    Ok(VectorReport {
        method: input.method,
        status,
        value,
        reason,
    })
}
