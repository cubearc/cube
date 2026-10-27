use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub id: String,
    #[serde(serialize_with = "weight", deserialize_with = "read_weight")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub weight: u64,
    #[serde(default)]
    pub value: Option<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FiniteModel {
    pub outcomes: Vec<Outcome>,
    pub event_a: Vec<String>,
    #[serde(default)]
    pub event_b: Option<Vec<String>>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OutcomeProbability {
    pub id: String,
    pub probability: Fraction,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct FiniteReport {
    pub assumptions: FiniteModel,
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub total_mass: u128,
    pub outcomes: Vec<OutcomeProbability>,
    pub probability_a: Fraction,
    pub probability_b: Option<Fraction>,
    pub intersection: Option<Fraction>,
    pub conditional_a_given_b: Option<Fraction>,
    pub posterior: Option<Vec<OutcomeProbability>>,
    pub expected_value: Option<SignedFraction>,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
fn event(
    ids: &[String],
    lookup: &BTreeMap<&str, usize>,
    b: &mut Budget,
) -> Result<BTreeSet<usize>, OperationError> {
    if ids.len() > MAX_SUPPORT {
        return Err(resource(b.op, "event", "event exceeds 4096 IDs"));
    }
    let mut set = BTreeSet::new();
    for id in ids {
        b.charge(1 + id.len())?;
        let Some(&i) = lookup.get(id.as_str()) else {
            return Err(invalid(b.op, "event", "unknown outcome ID"));
        };
        if !set.insert(i) {
            return Err(invalid(b.op, "event", "duplicate outcome ID in event"));
        }
    }
    Ok(set)
}
/// Exact normalized outcomes, intersection and conditioning on a supplied joint
/// model. Numeric values must be present for every outcome or none. Posterior is
/// the distribution of these same outcomes conditioned on B, with zeros retained.
pub fn finite_analyze(model: &FiniteModel, limits: Limits) -> Result<FiniteReport, OperationError> {
    const OP: &str = "probability.finite.analyze";
    if model.outcomes.is_empty() || model.outcomes.len() > MAX_SUPPORT {
        return Err(resource(
            OP,
            "outcomes",
            "require 1..4096 supplied outcomes",
        ));
    }
    if model
        .outcomes
        .iter()
        .any(|o| o.id.is_empty() || o.id.len() > 64)
    {
        return Err(invalid(OP, "id", "IDs require 1..64 UTF-8 bytes"));
    }
    if model.event_a.len() > MAX_SUPPORT
        || model
            .event_b
            .as_ref()
            .is_some_and(|v| v.len() > MAX_SUPPORT)
    {
        return Err(resource(OP, "event", "events exceed 4096 IDs"));
    }
    if model
        .event_a
        .iter()
        .chain(model.event_b.iter().flatten())
        .any(|s| s.len() > 64)
    {
        return Err(invalid(OP, "event", "event ID exceeds 64 bytes"));
    }
    let input = 128
        + model
            .outcomes
            .iter()
            .map(|o| crate::pipeline::json::string_bytes(&o.id) + 128)
            .sum::<usize>()
        + model
            .event_a
            .iter()
            .chain(model.event_b.iter().flatten())
            .map(|s| crate::pipeline::json::string_bytes(s) + 1)
            .sum::<usize>();
    if input > MAX_INPUT {
        return Err(resource(OP, "model", "compact model bound exceeds 1 MiB"));
    }
    let report = 4096 + input + model.outcomes.len() * 1400;
    let mut b = Budget::new(
        OP,
        limits,
        super::MAX_INPUT * 4 + model.outcomes.len() * 512 + report * 2,
        report,
    )?;
    let mut lookup = BTreeMap::new();
    let mut total = 0;
    let mut pos = 0;
    let mut neg = 0;
    let numeric = model.outcomes[0].value.is_some();
    for (i, o) in model.outcomes.iter().enumerate() {
        b.charge(1 + o.id.len())?;
        if lookup.insert(o.id.as_str(), i).is_some() {
            return Err(invalid(OP, "id", "duplicate outcome ID"));
        }
        if o.value.is_some() != numeric {
            return Err(invalid(
                OP,
                "value",
                "provide numeric values for all outcomes or none",
            ));
        }
        total = add(total, o.weight.into(), OP)?;
        if let Some(v) = o.value {
            let product = mul(u128::from(v.unsigned_abs()), o.weight.into(), OP)?;
            if v < 0 {
                neg = add(neg, product, OP)?;
            } else {
                pos = add(pos, product, OP)?;
            }
        }
    }
    if total == 0 {
        return Err(invalid(
            OP,
            "weights",
            "at least one weight must be positive",
        ));
    }
    let a = event(&model.event_a, &lookup, &mut b)?;
    let c = model
        .event_b
        .as_ref()
        .map(|ids| event(ids, &lookup, &mut b))
        .transpose()?;
    let (mut ma, mut mb, mut mab) = (0, 0, 0);
    let mut outcomes = Vec::new();
    for (i, o) in model.outcomes.iter().enumerate() {
        b.charge(1)?;
        let w = u128::from(o.weight);
        if a.contains(&i) {
            ma = add(ma, w, OP)?;
        }
        if c.as_ref().is_some_and(|c| c.contains(&i)) {
            mb = add(mb, w, OP)?;
            if a.contains(&i) {
                mab = add(mab, w, OP)?;
            }
        }
        outcomes.push(OutcomeProbability {
            id: o.id.clone(),
            probability: fraction(w, total),
        });
    }
    if c.is_some() && mb == 0 {
        return Err(invalid(OP, "event_b", "cannot condition on zero mass"));
    }
    let posterior = if let Some(c) = c {
        let mut v = Vec::new();
        for (i, o) in model.outcomes.iter().enumerate() {
            b.charge(1)?;
            v.push(OutcomeProbability {
                id: o.id.clone(),
                probability: fraction(if c.contains(&i) { o.weight.into() } else { 0 }, mb),
            });
        }
        Some(v)
    } else {
        None
    };
    let expected_value = if numeric {
        let f = fraction(pos.abs_diff(neg), total);
        Some(SignedFraction {
            negative: neg > pos,
            numerator: f.numerator,
            denominator: f.denominator,
        })
    } else {
        None
    };
    Ok(FiniteReport {
        assumptions: model.clone(),
        total_mass: total,
        outcomes,
        probability_a: fraction(ma, total),
        probability_b: model.event_b.as_ref().map(|_| fraction(mb, total)),
        intersection: model.event_b.as_ref().map(|_| fraction(mab, total)),
        conditional_a_given_b: model.event_b.as_ref().map(|_| fraction(mab, mb)),
        posterior,
        expected_value,
        work: b.work,
        memory_bytes_upper_bound: b.memory,
        report_bytes_upper_bound: b.report,
    })
}
