//! Fixed-point natural-log n-gram scores and exact whitespace-token dictionary scores.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Normalization {
    Exact,
    AsciiLetters,
}
impl Normalization {
    pub(crate) fn apply(self, text: &str) -> String {
        match self {
            Self::Exact => text.into(),
            Self::AsciiLetters => letters(text).0,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Weight {
    pub text: String,
    pub weight: f64,
}
/// Counts use one aggregate unseen bucket. Probabilities supply its mass directly.
/// Dictionary tokens use Rust Unicode whitespace; punctuation is retained.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Model {
    Counts {
        n: usize,
        entries: Vec<Weight>,
        smoothing: f64,
        normalization: Normalization,
    },
    Probabilities {
        n: usize,
        entries: Vec<Weight>,
        unseen_probability: f64,
        normalization: Normalization,
    },
    Dictionary {
        entries: Vec<Weight>,
        unseen_weight: f64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[schemars(length(max = 4096))]
    pub candidates: Vec<String>,
    pub model: Model,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Score {
    pub input_index: usize,
    pub samples: usize,
    /// Sum of per-sample values rounded to 1e-9; None means no scored samples.
    pub total_nanos: Option<i64>,
    /// Arithmetic mean in the same units, truncated toward zero.
    pub mean_nanos: Option<i64>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub method: String,
    pub model_sha256: String,
    pub status: Status,
    pub scores: Vec<Score>,
    /// Input indices sorted by descending mean score, then caller order; empty last.
    pub ranking: Vec<usize>,
}
pub(crate) struct Prepared {
    pub identity: String,
    pub method: String,
    n: usize,
    normalization: Normalization,
    weights: BTreeMap<String, i64>,
    unseen: i64,
}
fn positive(x: f64) -> bool {
    x.is_finite() && (1e-300..=1e12).contains(&x)
}
fn nanos(x: f64) -> i64 {
    (x * 1e9).round() as i64
}
impl Prepared {
    pub fn new(model: &Model) -> Result<Self, OperationError> {
        json_bound(model, MAX_TEXT)?;
        let (n, entries, normalization, unseen, denominator, logarithmic) = match model {
            Model::Counts {
                n,
                entries,
                smoothing,
                normalization,
            } => {
                if !positive(*smoothing)
                    || entries
                        .iter()
                        .any(|e| !e.weight.is_finite() || !(0.0..=1e12).contains(&e.weight))
                {
                    return Err(invalid(
                        "finite nonnegative counts and positive smoothing required",
                    ));
                }
                (
                    *n,
                    entries,
                    *normalization,
                    *smoothing,
                    entries.iter().map(|e| e.weight).sum::<f64>()
                        + smoothing * (entries.len() + 1) as f64,
                    true,
                )
            }
            Model::Probabilities {
                n,
                entries,
                unseen_probability,
                normalization,
            } => {
                let total = entries.iter().map(|e| e.weight).sum::<f64>() + unseen_probability;
                if !positive(*unseen_probability)
                    || entries.iter().any(|e| !positive(e.weight))
                    || (total - 1.0).abs() > 1e-9
                {
                    return Err(invalid(
                        "probabilities including unseen bucket must sum to one within 1e-9",
                    ));
                }
                (*n, entries, *normalization, *unseen_probability, 1.0, true)
            }
            Model::Dictionary {
                entries,
                unseen_weight,
            } => {
                if !unseen_weight.is_finite()
                    || unseen_weight.abs() > 100.0
                    || entries
                        .iter()
                        .any(|e| !e.weight.is_finite() || e.weight.abs() > 100.0)
                {
                    return Err(invalid("dictionary weights must be finite in [-100,100]"));
                }
                (0, entries, Normalization::Exact, *unseen_weight, 1.0, false)
            }
        };
        bound(entries.len(), 10000)?;
        if logarithmic && !(1..=8).contains(&n) {
            return Err(invalid("ngram width must be 1..8"));
        }
        let mut weights = BTreeMap::new();
        for e in entries {
            let valid = if logarithmic {
                e.text.chars().count() == n && normalization.apply(&e.text) == e.text
            } else {
                !e.text.is_empty() && !e.text.chars().any(char::is_whitespace)
            };
            if !valid {
                return Err(invalid(
                    "model entries must be canonical ngrams or single nonempty dictionary tokens",
                ));
            }
            let value = match model {
                Model::Counts { smoothing, .. } => (e.weight + smoothing).ln() - denominator.ln(),
                _ if logarithmic => e.weight.ln(),
                _ => e.weight,
            };
            if weights.insert(e.text.clone(), nanos(value)).is_some() {
                return Err(invalid("duplicate model entry"));
            }
        }
        let bytes = serde_json::to_vec(model).map_err(|_| invalid("invalid model"))?;
        Ok(Self {
            identity: crate::hashing::sha256(&bytes),
            method: if logarithmic {
                "ngram-log-nanos-v1"
            } else {
                "dictionary-weight-nanos-v1"
            }
            .into(),
            n,
            normalization,
            weights,
            unseen: nanos(if logarithmic {
                unseen.ln() - denominator.ln()
            } else {
                unseen
            }),
        })
    }
    pub fn score(&self, text: &str, index: usize, budget: &mut Budget<'_>) -> Option<Score> {
        // Charge every source scalar, including discarded punctuation and empty attempts.
        if !budget.take(text.chars().count().max(1)) {
            return None;
        }
        let normalized = self.normalization.apply(text);
        let mut count = 0usize;
        let mut total = 0i64;
        let mut add = |sample: &str| {
            count += 1;
            total += self.weights.get(sample).copied().unwrap_or(self.unseen);
        };
        if self.n == 0 {
            for sample in normalized.split_whitespace() {
                add(sample);
            }
        } else {
            let chars: Vec<char> = normalized.chars().collect();
            for window in chars.windows(self.n) {
                add(&window.iter().collect::<String>());
            }
        }
        Some(Score {
            input_index: index,
            samples: count,
            total_nanos: (count > 0).then_some(total),
            mean_nanos: (count > 0).then(|| total / count as i64),
        })
    }
}
/// Score ordered candidates against the explicit model. Cancellation/work limits
/// return a marked prefix; invalid models reject before evaluating candidates.
pub fn score(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    bound(r.candidates.len(), 4096)?;
    let mut bytes = 0usize;
    for text in &r.candidates {
        text_bound(text, MAX_TEXT)?;
        bytes = bytes.saturating_add(text.len());
    }
    bound(bytes, MAX_TEXT)?;
    let p = Prepared::new(&r.model)?;
    let mut budget = Budget::new(&r.limits, cancel)?;
    let mut scores = Vec::new();
    for (i, text) in r.candidates.iter().enumerate() {
        if scores.len() == r.limits.max_results {
            budget.stop(Stop::ResultLimit);
            break;
        }
        let Some(s) = p.score(text, i, &mut budget) else {
            break;
        };
        scores.push(s);
    }
    let mut ranking: Vec<_> = (0..scores.len()).collect();
    ranking.sort_by_key(|&i| std::cmp::Reverse(scores[i].mean_nanos));
    finish(Report {
        method: p.method,
        model_sha256: p.identity,
        status: budget.status,
        scores,
        ranking,
    })
}
