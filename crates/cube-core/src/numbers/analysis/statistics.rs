//! Explicit finite sample summaries with reproducible reduction order.
use super::{Unsigned, Work, check_report, invalid, parameter, resource};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Estimator {
    Shannon,
    Renyi,
    MillerMadow,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum EntropyRequest {
    Shannon { counts: Vec<u64> },
    Renyi { counts: Vec<u64>, alpha: f64 },
    MillerMadow { counts: Vec<u64> },
}
#[derive(Serialize, JsonSchema)]
pub struct EntropyReport {
    pub method: Estimator,
    pub alpha: Option<f64>,
    pub samples: Unsigned,
    pub nonzero_bins: usize,
    pub bits: f64,
}

#[derive(Default)]
struct Sum {
    value: f64,
    error: f64,
}
impl Sum {
    fn add(&mut self, value: f64) {
        let adjusted = value - self.error;
        let combined = self.value + adjusted;
        self.error = (combined - self.value) - adjusted;
        self.value = combined;
    }
}
fn sum(values: impl Iterator<Item = f64>) -> f64 {
    let mut total = Sum::default();
    for value in values {
        total.add(value);
    }
    total.value
}
fn finite(value: f64) -> Result<f64, OperationError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid(
            "numeric calculation requires finite inputs and intermediates",
        ))
    }
}

/// Estimate entropy from explicitly supplied counts.
pub fn entropy(request: &EntropyRequest) -> Result<EntropyReport, OperationError> {
    let (counts, method, alpha) = match request {
        EntropyRequest::Shannon { counts } => (counts, Estimator::Shannon, None),
        EntropyRequest::Renyi { counts, alpha } => (counts, Estimator::Renyi, Some(*alpha)),
        EntropyRequest::MillerMadow { counts } => (counts, Estimator::MillerMadow, None),
    };
    if counts.len() > 10_000 {
        return Err(resource("entropy exceeds 10000 counts"));
    }
    let total = counts.iter().map(|&n| u128::from(n)).sum();
    if total == 0 {
        return Err(invalid("entropy requires at least one positive count"));
    }
    let positive = || counts.iter().copied().filter(|&n| n != 0);
    let probabilities = || positive().map(|n| n as f64 / total as f64);
    let shannon = || sum(probabilities().map(|p| -p * p.log2()));
    let nonzero_bins = positive().count();
    let bits = match alpha {
        Some(a) if !(0.5..=16.).contains(&a) || a != 1. && (a - 1.).abs() < 1e-6 => {
            return Err(parameter(
                "Rényi alpha must be in 0.5..16 and separated from 1 by at least 1e-6, or exactly 1",
            ));
        }
        Some(a) if a != 1. && (a - 1.).abs() <= 0.125 => {
            // Rewrite the sum as 1 + Σ p·expm1((a-1)·ln(p)) near alpha=1.
            // This preserves the small log-sum before division by 1-alpha.
            let correction = sum(probabilities().map(|p| p * ((a - 1.) * p.ln()).exp_m1()));
            correction.ln_1p() / ((1. - a) * std::f64::consts::LN_2)
        }
        Some(a) if a != 1. => {
            let logs: Vec<_> = probabilities().map(|p| a * p.ln()).collect();
            let high = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            (high + sum(logs.into_iter().map(|x| (x - high).exp())).ln())
                / ((1. - a) * std::f64::consts::LN_2)
        }
        _ if matches!(method, Estimator::MillerMadow) => {
            shannon() + (nonzero_bins - 1) as f64 / (2. * total as f64 * std::f64::consts::LN_2)
        }
        _ => shannon(),
    };
    Ok(EntropyReport {
        method,
        alpha,
        samples: Unsigned(total),
        nonzero_bins,
        bits: finite(bits)?.max(0.),
    })
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Normalize {
    #[default]
    Raw,
    PopulationCovariance,
    Pearson,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum CorrelationRequest {
    Lag {
        x: Vec<f64>,
        #[serde(default)]
        y: Option<Vec<f64>>,
        lags: Vec<i32>,
        #[serde(default)]
        normalize: Normalize,
    },
    Pearson {
        x: Vec<f64>,
        y: Vec<f64>,
    },
    Spearman {
        x: Vec<f64>,
        y: Vec<f64>,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct Lag {
    pub lag: i32,
    pub overlap: usize,
    pub value: Option<f64>,
}
#[derive(Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CorrelationReport {
    Lags { lags: Vec<Lag> },
    Scalar { value: Option<f64>, samples: usize },
}

fn validate_vector(x: &[f64], maximum: usize) -> Result<(), OperationError> {
    if x.len() > maximum {
        return Err(resource("sample count exceeds its limit"));
    }
    if !x.iter().all(|n| n.is_finite()) {
        return Err(invalid("samples must be finite"));
    }
    Ok(())
}
fn scale(values: &[f64]) -> f64 {
    let maximum = values.iter().map(|v| v.abs()).fold(0., f64::max);
    if maximum == 0. { 1. } else { maximum }
}
struct Moments {
    cross: f64,
    xx: f64,
    yy: f64,
}
fn centered(x: &[f64], y: &[f64], xs: f64, ys: f64) -> Moments {
    let (xm, ym) = (
        sum(x.iter().map(|n| n / xs)) / x.len() as f64,
        sum(y.iter().map(|n| n / ys)) / y.len() as f64,
    );
    let (mut cross, mut xx, mut yy) = (Sum::default(), Sum::default(), Sum::default());
    for (&x, &y) in x.iter().zip(y) {
        let (dx, dy) = (x / xs - xm, y / ys - ym);
        cross.add(dx * dy);
        xx.add(dx * dx);
        yy.add(dy * dy);
    }
    Moments {
        cross: cross.value,
        xx: xx.value,
        yy: yy.value,
    }
}
fn normalized(
    x: &[f64],
    y: &[f64],
    mode: Normalize,
    work: &mut Work,
) -> Result<Option<f64>, OperationError> {
    if x.is_empty() {
        return Err(invalid("requested lag has no overlapping samples"));
    }
    let passes = if matches!(mode, Normalize::Raw) { 3 } else { 5 };
    work.charge(x.len() * passes)?;
    let (xs, ys) = (scale(x), scale(y));
    let restore = |c| (c * xs.min(ys)) * xs.max(ys);
    if matches!(mode, Normalize::Raw) {
        return Ok(Some(finite(restore(sum(x
            .iter()
            .zip(y)
            .map(|(&a, &b)| (a / xs) * (b / ys)))))?));
    }
    let m = centered(x, y, xs, ys);
    match mode {
        Normalize::Raw => unreachable!(),
        Normalize::PopulationCovariance | Normalize::Pearson if m.xx == 0. || m.yy == 0. => {
            Ok(None)
        }
        Normalize::PopulationCovariance => Ok(Some(finite(restore(m.cross / x.len() as f64))?)),
        Normalize::Pearson => {
            let value = finite(m.cross / (m.xx.sqrt() * m.yy.sqrt()))?;
            if value.abs() > 1. + 1e-12 {
                return Err(invalid("Pearson calculation exceeded its numerical range"));
            }
            Ok(Some(value.clamp(-1., 1.)))
        }
    }
}
fn overlap<'a>(x: &'a [f64], y: &'a [f64], lag: i32) -> Result<LagSlices<'a>, OperationError> {
    let offset = lag.unsigned_abs() as usize;
    let (xx, yy) = (
        if lag < 0 { x.get(offset..) } else { Some(x) },
        if lag > 0 { y.get(offset..) } else { Some(y) },
    );
    let (Some(xx), Some(yy)) = (xx, yy) else {
        return Err(invalid("requested lag has no overlapping samples"));
    };
    let n = xx.len().min(yy.len());
    Ok(LagSlices {
        left: &xx[..n],
        right: &yy[..n],
    })
}
struct LagSlices<'a> {
    left: &'a [f64],
    right: &'a [f64],
}
fn ranks(values: &[f64], work: &mut Work) -> Result<Vec<f64>, OperationError> {
    let mut indexes: Vec<usize> = (0..values.len()).collect();
    let mut comparisons = 0;
    indexes.sort_unstable_by(|&a, &b| {
        comparisons += 1;
        values[a].total_cmp(&values[b]).then_with(|| a.cmp(&b))
    });
    work.charge(comparisons + values.len() * 2)?;
    let mut output = vec![0.; values.len()];
    let mut start = 0;
    while start < indexes.len() {
        let mut end = start + 1;
        while end < indexes.len() && values[indexes[end]] == values[indexes[start]] {
            end += 1;
        }
        let rank = (start + 1 + end) as f64 / 2.;
        for &index in &indexes[start..end] {
            output[index] = rank;
        }
        start = end;
    }
    Ok(output)
}

/// Compute explicit overlap moments or scalar Pearson and tie-ranked Spearman.
pub fn correlation(request: &CorrelationRequest) -> Result<CorrelationReport, OperationError> {
    let (x, y) = match request {
        CorrelationRequest::Lag { x, y, .. } => (x.as_slice(), y.as_deref().unwrap_or(x)),
        CorrelationRequest::Pearson { x, y } | CorrelationRequest::Spearman { x, y } => {
            (x.as_slice(), y.as_slice())
        }
    };
    validate_vector(x, 65536)?;
    validate_vector(y, 65536)?;
    let mut work = Work(16 << 20);
    if let CorrelationRequest::Lag {
        lags, normalize, ..
    } = request
    {
        if lags.len() > 256 {
            return Err(resource("at most 256 lags are supported"));
        }
        let lags = lags
            .iter()
            .map(|&lag| {
                let slice = overlap(x, y, lag)?;
                let overlap = slice.left.len();
                let value = normalized(slice.left, slice.right, *normalize, &mut work)?;
                Ok(Lag {
                    lag,
                    overlap,
                    value,
                })
            })
            .collect::<Result<Vec<_>, OperationError>>()?;
        return Ok(CorrelationReport::Lags { lags });
    }
    if x.len() != y.len() || x.len() < 2 {
        return Err(invalid(
            "scalar correlation requires matching sample counts of at least two",
        ));
    }
    let value = if matches!(request, CorrelationRequest::Spearman { .. }) {
        normalized(
            &ranks(x, &mut work)?,
            &ranks(y, &mut work)?,
            Normalize::Pearson,
            &mut work,
        )?
    } else {
        normalized(x, y, Normalize::Pearson, &mut work)?
    };
    Ok(CorrelationReport::Scalar {
        value,
        samples: x.len(),
    })
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tie {
    #[default]
    Error,
    Drop,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "input", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunsRequest {
    Binary {
        values: Vec<u8>,
    },
    Threshold {
        values: Vec<f64>,
        threshold: f64,
        #[serde(default)]
        tie: Tie,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct RunsReport {
    pub runs: usize,
    pub n0: usize,
    pub n1: usize,
    pub dropped: usize,
    pub expected: Option<f64>,
    pub variance: Option<f64>,
    pub z: Option<f64>,
    pub large_sample: bool,
    pub undefined_reason: Option<&'static str>,
}
#[derive(Default)]
struct Counts {
    runs: usize,
    counts: [usize; 2],
    dropped: usize,
    last: Option<u8>,
}
impl Counts {
    fn push(&mut self, value: u8) {
        self.counts[usize::from(value)] += 1;
        if self.last != Some(value) {
            self.runs += 1;
            self.last = Some(value);
        }
    }
}
/// Wald–Wolfowitz two-category evidence using the NIST mean and variance.
pub fn runs(request: &RunsRequest) -> Result<RunsReport, OperationError> {
    let mut state = Counts::default();
    match request {
        RunsRequest::Binary { values } => {
            if values.len() > 1 << 20 {
                return Err(resource("runs input exceeds 1048576 categories"));
            }
            for &n in values {
                if n > 1 {
                    return Err(invalid("binary categories are exact integers zero or one"));
                }
                state.push(n);
            }
        }
        RunsRequest::Threshold {
            values,
            threshold,
            tie,
        } => {
            finite(*threshold)?;
            validate_vector(values, 1 << 20)?;
            for &n in values {
                if n == *threshold {
                    if matches!(tie, Tie::Error) {
                        return Err(invalid("value equals the explicit threshold"));
                    }
                    state.dropped += 1;
                } else {
                    state.push(u8::from(n > *threshold));
                }
            }
        }
    }
    let [n0, n1] = state.counts;
    let total = (n0 + n1) as f64;
    let product = 2. * n0 as f64 * n1 as f64;
    let expected = (total > 0.).then(|| 1. + product / total);
    let variance = if total == 0. {
        None
    } else if product == 0. {
        Some(0.)
    } else {
        Some(product * (product - total) / (total * total * (total - 1.)))
    };
    let z = variance
        .filter(|&v| v > 0.)
        .map(|v| (state.runs as f64 - expected.expect("nonempty categories")) / v.sqrt());
    let undefined_reason = if total == 0. {
        Some("empty")
    } else if n0 == 0 || n1 == 0 {
        Some("single_category")
    } else if z.is_none() {
        Some("zero_variance")
    } else {
        None
    };
    Ok(RunsReport {
        runs: state.runs,
        n0,
        n1,
        dropped: state.dropped,
        expected,
        variance,
        z,
        large_sample: n0 > 10 && n1 > 10,
        undefined_reason,
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenfordRequest {
    pub values: Vec<String>,
}
#[derive(Serialize, JsonSchema)]
pub struct BenfordReport {
    pub counts: [usize; 9],
    pub included: usize,
    pub zeros: usize,
    pub probabilities: [f64; 9],
}

/// Count exact first significant decimal digits and return Benford weights.
pub fn benford(request: &BenfordRequest) -> Result<BenfordReport, OperationError> {
    if request.values.len() > 65536 {
        return Err(resource("first-digit input exceeds 65536 values"));
    }
    let mut remaining = 1 << 20;
    let mut counts = [0; 9];
    let mut zeros = 0;
    for value in &request.values {
        if value.len() > 4096 || value.len() > remaining {
            return Err(resource(
                "first-digit input exceeds its per-value or total byte limit",
            ));
        }
        remaining -= value.len();
        let (_, digits, _) = crate::numbers::signed_digits(value);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid(
                "first-digit inputs require signed ASCII decimal integers",
            ));
        }
        if let Some(digit) = digits.bytes().find(|&b| b != b'0') {
            counts[usize::from(digit - b'1')] += 1;
        } else {
            zeros += 1;
        }
    }
    // Rounded from log10(1 + 1/d) for integral d = 1 through 9.
    let probabilities = [
        std::f64::consts::LOG10_2,
        0.17609125905568124,
        0.12493873660829995,
        0.09691001300805642,
        0.07918124604762483,
        0.0669467896306132,
        0.057991946977686754,
        0.05115252244738129,
        0.04575749056067512,
    ];
    Ok(BenfordReport {
        counts,
        included: request.values.len() - zeros,
        zeros,
        probabilities,
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MarkovRequest {
    pub states: Vec<String>,
    #[serde(default)]
    pub sequence: Option<Vec<String>>,
    #[serde(default)]
    pub sequences: Option<Vec<Vec<String>>>,
}
#[derive(Serialize, JsonSchema)]
pub struct Probability {
    pub numerator: u32,
    pub denominator: u32,
}
#[derive(Serialize, JsonSchema)]
pub struct TransitionRow {
    pub total: u32,
    pub counts: Vec<u32>,
    pub probabilities: Option<Vec<Probability>>,
}
#[derive(Serialize, JsonSchema)]
pub struct MarkovReport {
    pub states: Vec<String>,
    pub sequence_length: usize,
    pub sequence_count: usize,
    pub rows: Vec<TransitionRow>,
}
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Count transitions along explicit sequence boundaries and normalize nonempty rows.
pub fn markov(request: &MarkovRequest) -> Result<MarkovReport, OperationError> {
    let sequences = match (&request.sequence, &request.sequences) {
        (Some(single), None) => std::slice::from_ref(single),
        (None, Some(many)) => many,
        _ => return Err(parameter("provide exactly one of sequence or sequences")),
    };
    if request.states.len() > 256 || sequences.len() > 65536 {
        return Err(resource("Markov dimensions exceed their bounds"));
    }
    check_report(&(&request.states, sequences), 1 << 20)?;
    let ids: BTreeMap<_, _> = request
        .states
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();
    if ids.len() != request.states.len() {
        return Err(invalid("state IDs must be distinct"));
    }
    let size = ids.len();
    let mut rows = vec![vec![0_u32; size]; size];
    let mut sequence_length = 0;
    for sequence in sequences {
        sequence_length += sequence.len();
        if sequence_length > 65536 {
            return Err(resource("Markov sequences exceed 65536 state observations"));
        }
        let mut previous = None::<usize>;
        for name in sequence {
            let &state = ids
                .get(name.as_str())
                .ok_or_else(|| invalid("sequence contains an undeclared state"))?;
            if let Some(from) = previous {
                rows[from][state] += 1;
            }
            previous = Some(state);
        }
    }
    let rows = rows
        .into_iter()
        .map(|counts| {
            let total = counts.iter().sum();
            let probabilities = (total != 0).then(|| {
                counts
                    .iter()
                    .map(|&n| {
                        let d = gcd(total, n);
                        Probability {
                            numerator: n / d,
                            denominator: total / d,
                        }
                    })
                    .collect()
            });
            TransitionRow {
                total,
                counts,
                probabilities,
            }
        })
        .collect();
    Ok(MarkovReport {
        states: request.states.clone(),
        sequence_length,
        sequence_count: sequences.len(),
        rows,
    })
}
