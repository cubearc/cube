//! Pearson distance for aligned caller-owned counts and expected masses.
//! No distribution is fitted and no p-value or posterior probability is inferred.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedPolicy {
    /// Expected counts must sum to the observed total within relative 1e-9.
    Counts,
    /// Rescale supplied nonnegative masses to the observed total.
    NormalizeProportions,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bin {
    #[schemars(length(min = 1, max = 256))]
    pub label: String,
    #[serde(deserialize_with = "wire::integer")]
    #[schemars(range(min = 0, max = 16777216))]
    pub observed: usize,
    pub expected: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[schemars(length(max = 10000))]
    pub bins: Vec<Bin>,
    pub expected_policy: ExpectedPolicy,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Distance {
    Finite {
        value: f64,
    },
    /// Positive observation with zero expected mass gives infinite distance.
    ZeroExpectedMismatch,
    /// A positive finite mass produces a distance beyond finite f64 range.
    OutOfRange,
    /// No observations or an omitted 0/0 bin.
    Undefined,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub label: String,
    pub observed: usize,
    pub expected: f64,
    pub distance: Distance,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub method: String,
    pub expected_policy: ExpectedPolicy,
    pub model_sha256: String,
    pub observed_total: usize,
    pub expected_total: f64,
    /// Positive expected bins minus one; no fitted parameters. Undefined for
    /// incomplete evaluation, no observations or a nonfinite distance.
    pub degrees_of_freedom: Option<usize>,
    pub distance: Distance,
    pub bins: Vec<Evidence>,
    pub status: Status,
    pub scope: String,
}

fn validate(r: &Request) -> Result<(usize, f64), OperationError> {
    json_bound(r, MAX_TEXT)?;
    bound(r.bins.len(), 10000)?;
    let mut labels = BTreeSet::new();
    let mut observed = 0usize;
    let mut expected = 0.0;
    for bin in &r.bins {
        if bin.label.is_empty() || bin.label.len() > 256 || !labels.insert(&bin.label) {
            return Err(invalid(
                "frequency labels must be unique and contain 1..256 UTF-8 bytes",
            ));
        }
        if !bin.expected.is_finite()
            || (bin.expected != 0.0 && !(1e-300..=1e12).contains(&bin.expected))
        {
            return Err(invalid(
                "expected masses must be zero or finite in [1e-300,1e12]",
            ));
        }
        bound(bin.observed, MAX_WORK)?;
        observed += bin.observed;
        bound(observed, MAX_WORK)?;
        expected += bin.expected;
    }
    match r.expected_policy {
        ExpectedPolicy::Counts
            if (observed as f64 - expected).abs() > 1e-9 * (observed as f64).max(1.0) =>
        {
            return Err(invalid(
                "expected counts must match the observed total within relative 1e-9",
            ));
        }
        ExpectedPolicy::NormalizeProportions if expected == 0.0 => {
            return Err(invalid(
                "normalization requires positive total expected mass",
            ));
        }
        _ => {}
    }
    Ok((observed, expected))
}
fn contribution(observed: usize, expected: f64) -> Distance {
    if expected == 0.0 {
        return if observed == 0 {
            Distance::Undefined
        } else {
            Distance::ZeroExpectedMismatch
        };
    }
    let difference = observed as f64 - expected;
    let value = difference * difference / expected;
    if value.is_finite() {
        Distance::Finite { value }
    } else {
        Distance::OutOfRange
    }
}
fn add(left: Distance, right: Distance) -> Distance {
    match (left, right) {
        (Distance::ZeroExpectedMismatch, _) | (_, Distance::ZeroExpectedMismatch) => {
            Distance::ZeroExpectedMismatch
        }
        (Distance::OutOfRange, _) | (_, Distance::OutOfRange) => Distance::OutOfRange,
        (Distance::Undefined, other) | (other, Distance::Undefined) => other,
        (Distance::Finite { value: a }, Distance::Finite { value: b }) => {
            if (a + b).is_finite() {
                Distance::Finite { value: a + b }
            } else {
                Distance::OutOfRange
            }
        }
    }
}
/// Sum `(observed - expected)^2 / expected` in caller order. Lower is closer;
/// no logarithm is applied. Charge max(1, observed) per bin without expanding
/// counts. Limits/cancellation retain an explicitly incomplete prefix.
pub fn compare(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    let (observed_total, supplied_total) = validate(r)?;
    let mut budget = Budget::new(&r.limits, cancel)?;
    let model: Vec<_> = r.bins.iter().map(|b| (&b.label, b.expected)).collect();
    let model = serde_json::to_vec(&(r.expected_policy, model))
        .map_err(|_| invalid("invalid frequency model"))?;
    let mut bins = Vec::new();
    let mut distance = Distance::Undefined;
    let mut expected_total = 0.0;
    let mut positive_bins = 0usize;
    for bin in &r.bins {
        if bins.len() == r.limits.max_results {
            budget.stop(Stop::ResultLimit);
            break;
        }
        if !budget.take(bin.observed.max(1)) {
            break;
        }
        let expected = match r.expected_policy {
            ExpectedPolicy::Counts => bin.expected,
            ExpectedPolicy::NormalizeProportions => {
                bin.expected / supplied_total * observed_total as f64
            }
        };
        if bin.expected > 0.0 && observed_total > 0 && expected == 0.0 {
            return Err(invalid("expected mass normalization underflows f64"));
        }
        expected_total += expected;
        positive_bins += usize::from(expected > 0.0);
        let term = contribution(bin.observed, expected);
        distance = add(distance, term);
        bins.push(Evidence {
            label: bin.label.clone(),
            observed: bin.observed,
            expected,
            distance: term,
        });
    }
    if observed_total == 0 {
        distance = Distance::Undefined;
    }
    let degrees_of_freedom = if budget.status.complete
        && observed_total > 0
        && matches!(distance, Distance::Finite { .. })
    {
        positive_bins.checked_sub(1)
    } else {
        None
    };
    finish(Report {
        method: "pearson-chi-square-f64-v1".into(),
        expected_policy: r.expected_policy,
        model_sha256: crate::hashing::sha256(&model),
        observed_total, expected_total, degrees_of_freedom, distance, bins,
        status: budget.status,
        scope: "Raw frequency distance for supplied aligned bins and a fixed model; no logarithm, fitted parameters, p-value, posterior, identity or authenticity claim. Incomplete results cover only the returned prefix.".into(),
    })
}
