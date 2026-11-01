//! Finite, explainable compatibility rules. Thresholds describe this profile's
//! policy, not universal language facts or calibrated probabilities.
use super::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    AsciiClassicalEvidenceV1,
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    NonuniformCoincidenceV1,
    PeriodicColumnsV1,
}
fn rules() -> Vec<Rule> {
    vec![Rule::NonuniformCoincidenceV1, Rule::PeriodicColumnsV1]
}
fn periods() -> Vec<usize> {
    (2..=16).collect()
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub profile: Profile,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
    #[serde(default = "rules")]
    #[schemars(length(max = 2))]
    pub rules: Vec<Rule>,
    #[serde(default = "periods", deserialize_with = "wire::indices")]
    #[schemars(length(max = 32))]
    pub periods: Vec<usize>,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Observations {
    pub input_bytes: usize,
    pub ascii_letters: usize,
    pub distinct_letters: usize,
    pub letter_fraction: f64,
    pub maximum_letter_fraction: f64,
    pub ic: Option<f64>,
    pub control_bytes: usize,
    pub non_ascii_bytes: usize,
    pub transport_like: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Suggestion {
    pub rule: Rule,
    pub compatible_families: Vec<String>,
    pub periods: Vec<usize>,
    pub assumptions: Vec<String>,
    pub alternatives: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ColumnEvidence {
    pub observation: period::Period,
    pub samples_per_column: usize,
    pub minimum_distinct_letters: usize,
    pub minimum_squared_coefficient_of_variation: f64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub profile: Profile,
    pub view: String,
    pub status: Status,
    /// None if the input scan was not completed within the budget.
    pub observations: Option<Observations>,
    pub eligibility_failures: Vec<String>,
    pub columns: Vec<ColumnEvidence>,
    pub suggestions: Vec<Suggestion>,
}
fn source(r: &Request) -> Result<&[u8], OperationError> {
    match (&r.text, &r.bytes) {
        (Some(t), None) => Ok(t.as_bytes()),
        (None, Some(b)) => Ok(b),
        _ => Err(invalid(
            "supply exactly one of text or bytes; bytes are never automatically decoded",
        )),
    }
}
fn observe(bytes: &[u8]) -> (Vec<u8>, Observations) {
    let letters: Vec<_> = bytes
        .iter()
        .filter(|b| b.is_ascii_alphabetic())
        .map(|b| b.to_ascii_uppercase())
        .collect();
    let counts = period::counts(&letters);
    let letter_fraction = letters.len() as f64 / bytes.len().max(1) as f64;
    let compact: Vec<_> = bytes
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    let hex = !compact.is_empty() && compact.iter().all(u8::is_ascii_hexdigit);
    let encoded = compact.len() >= 64
        && !bytes.iter().any(u8::is_ascii_whitespace)
        && compact
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || b"+/=_-".contains(b))
        && compact
            .iter()
            .any(|b| b.is_ascii_digit() || b"+/=_-".contains(b));
    let observations = Observations {
        input_bytes: bytes.len(),
        ascii_letters: letters.len(),
        distinct_letters: counts.iter().filter(|&&c| c > 0).count(),
        letter_fraction,
        maximum_letter_fraction: *counts.iter().max().expect("26 bins") as f64
            / letters.len().max(1) as f64,
        ic: period::ic(&counts),
        control_bytes: bytes
            .iter()
            .filter(|b| b.is_ascii_control() && !b.is_ascii_whitespace())
            .count(),
        non_ascii_bytes: bytes.iter().filter(|b| !b.is_ascii()).count(),
        transport_like: hex || encoded,
    };
    (letters, observations)
}
fn eligibility(o: &Observations) -> Vec<String> {
    let mut reasons = Vec::new();
    for (failed, reason) in [
        (o.ascii_letters < 64, "fewer_than_64_ascii_letters"),
        (o.distinct_letters < 8, "fewer_than_8_distinct_letters"),
        (o.letter_fraction < 0.75, "letter_fraction_below_0_75"),
        (
            o.maximum_letter_fraction >= 0.25,
            "dominant_letter_at_least_0_25",
        ),
        (o.control_bytes > 0, "binary_control_bytes"),
        (o.non_ascii_bytes > 0, "outside_ascii_profile"),
        (o.transport_like, "transport_like_or_hex"),
    ] {
        if failed {
            reasons.push(reason.into());
        }
    }
    reasons
}
fn nonuniform(o: &Observations) -> Option<Suggestion> {
    o.ic.filter(|ic|(0.052..=0.095).contains(ic)).map(|_|Suggestion {
        rule:Rule::NonuniformCoincidenceV1,
        compatible_families:vec!["plaintext".into(),"monoalphabetic_substitution".into(),"transposition".into()],periods:Vec::new(),
        assumptions:vec!["ASCII letter view has at least 64 letters and 8 distinct letters; IC in [0.052,0.095].".into(),"Nonuniform natural-language-like distribution; thresholds are fixed policy, not universal language properties.".into()],
        alternatives:vec!["IC is invariant under a bijective substitution or rearrangement; it cannot distinguish these families from plaintext.".into(),"Structured or periodic data can have the same counts; short or other-language samples may fall outside this interval.".into()],
    })
}
fn periodic(o: &Observations, columns: &[ColumnEvidence]) -> Option<Suggestion> {
    let global = o.ic?;
    let periods: Vec<_> = columns
        .iter()
        .filter(|p| {
            p.minimum_distinct_letters >= 8
                && p.minimum_squared_coefficient_of_variation >= 0.1
                && p.observation
                    .mean_column_ic
                    .is_some_and(|ic| (0.052..=0.095).contains(&ic) && ic - global >= 0.012)
        })
        .map(|p| p.observation.period)
        .collect();
    (!periods.is_empty()).then(||Suggestion {
        rule:Rule::PeriodicColumnsV1,compatible_families:vec!["repeating_key_polyalphabetic".into()],periods,
        assumptions:vec!["At least 24 ASCII letters, 8 distinct letters and squared coefficient of variation >=0.1 over occupied bins per column; mean column IC in [0.052,0.095] and at least 0.012 above whole-text IC.".into(),"The caller's periods split the ASCII-letter stream; punctuation does not advance the period.".into()],
        alternatives:vec!["Periodic plaintext, interleaved streams and constructed data may produce the same signal.".into(),"Uniform/random data or insufficient column evidence does not establish any cipher identity.".into()],
    })
}
/// Suggest compatible classical families from explicit facts and finite rules.
/// Empty suggestions mean insufficient evidence, never proof of encryption.
/// Caller rule order is preserved; duplicate rules/periods are rejected.
pub fn suggest(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    let bytes = source(r)?;
    bound(bytes.len(), MAX_TEXT)?;
    bound(r.rules.len(), 2)?;
    bound(r.periods.len(), 32)?;
    if r.rules.iter().collect::<BTreeSet<_>>().len() != r.rules.len()
        || r.periods.iter().collect::<BTreeSet<_>>().len() != r.periods.len()
        || r.periods.iter().any(|p| !(2..=64).contains(p))
    {
        return Err(invalid(
            "unique supported rules and periods in 2..64 required",
        ));
    }
    let mut budget = Budget::new(&r.limits, cancel)?;
    let mut report = Report {
        profile: r.profile,
        view: "ascii_letters_uppercase_byte_view_v1".into(),
        status: budget.status.clone(),
        observations: None,
        eligibility_failures: Vec::new(),
        columns: Vec::new(),
        suggestions: Vec::new(),
    };
    if !budget.take(bytes.len().max(1)) {
        report.status = budget.status;
        return finish(report);
    }
    let (letters, observations) = observe(bytes);
    report.eligibility_failures = eligibility(&observations);
    if report.eligibility_failures.is_empty() {
        for &rule in &r.rules {
            if !budget.take(1) {
                break;
            }
            let suggestion = match rule {
                Rule::NonuniformCoincidenceV1 => nonuniform(&observations),
                Rule::PeriodicColumnsV1 => {
                    for &p in &r.periods {
                        if letters.len() / p < 24 {
                            continue;
                        }
                        if !budget.take(letters.len() + p * 27) {
                            break;
                        }
                        report.columns.push(column_evidence(&letters, p));
                    }
                    if !budget.status.complete {
                        break;
                    }
                    periodic(&observations, &report.columns)
                }
            };
            if let Some(s) = suggestion {
                if report.suggestions.len() == r.limits.max_results {
                    budget.stop(Stop::ResultLimit);
                    break;
                }
                report.suggestions.push(s);
            }
        }
    }
    report.observations = Some(observations);
    report.status = budget.status;
    finish(report)
}

fn column_evidence(bytes: &[u8], p: usize) -> ColumnEvidence {
    let columns = period::column_counts(bytes, p);
    let minimum_distinct_letters = columns
        .iter()
        .map(|c| c.iter().filter(|&&n| n > 0).count())
        .min()
        .unwrap_or(0);
    let minimum_squared_coefficient_of_variation = columns
        .iter()
        .map(|c| {
            let occupied = c.iter().filter(|&&n| n > 0).count() as f64;
            let sum = c.iter().sum::<u64>() as f64;
            let squares = c.iter().map(|&n| (n as f64).powi(2)).sum::<f64>();
            occupied * squares / (sum * sum) - 1.0
        })
        .fold(f64::INFINITY, f64::min);
    ColumnEvidence {
        observation: period::summarize_columns(&columns, p),
        samples_per_column: bytes.len() / p,
        minimum_distinct_letters,
        minimum_squared_coefficient_of_variation,
    }
}
