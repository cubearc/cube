//! Exact set and multiset comparisons of caller-owned terms.
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Terms {
    Tokens {
        left: Vec<String>,
        right: Vec<String>,
    },
    Qgrams {
        left: String,
        right: String,
        #[serde(default = "one")]
        q: usize,
    },
}
const fn one() -> usize {
    1
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SetMethod {
    Jaccard,
    Dice,
    Overlap,
    Tversky,
    QgramL1,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rational {
    pub numerator: u32,
    #[schemars(range(min = 1))]
    pub denominator: u32,
}
impl Default for Rational {
    fn default() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetRequest {
    pub input: Terms,
    pub method: SetMethod,
    #[serde(default)]
    pub alpha: Option<Rational>,
    #[serde(default)]
    pub beta: Option<Rational>,
}

#[derive(Serialize, JsonSchema)]
pub struct Fraction {
    /// Decimal digits preserve the exact value on 64-bit and JavaScript hosts.
    pub numerator: String,
    pub denominator: String,
}
#[derive(Serialize, JsonSchema)]
pub struct SetReport {
    pub method: SetMethod,
    pub intersection: usize,
    pub union: usize,
    pub left_only: usize,
    pub right_only: usize,
    pub fraction: Option<Fraction>,
    pub similarity: Option<f64>,
    pub jaccard_distance: Option<f64>,
    pub l1: Option<usize>,
}

pub(super) const DISTINCT: usize = 10_000;
type Counts<'a> = BTreeMap<&'a str, usize>;

fn count<'a>(
    items: impl Iterator<Item = &'a str>,
    work: &mut Work,
) -> Result<Counts<'a>, OperationError> {
    let mut terms = Counts::new();
    for term in items {
        // Two inclusive tree paths and node probes conservatively charge the
        // compared bytes. This bounds repeated long equal Unicode keys too.
        work.charge(1 + term.len() + 24)?;
        *terms.entry(term).or_default() += 1;
        if terms.len() > DISTINCT {
            return Err(resource("term inventory exceeds 10000 distinct keys"));
        }
    }
    Ok(terms)
}

fn token_counts<'a>(tokens: &'a [String], work: &mut Work) -> Result<Counts<'a>, OperationError> {
    if tokens.len() > MAX_UNITS
        || tokens.iter().any(|token| token.len() > 64)
        || tokens.iter().map(String::len).sum::<usize>() > TEXT_BYTES
    {
        return Err(resource(
            "explicit term count or byte length exceeds its bound",
        ));
    }
    count(tokens.iter().map(String::as_str), work)
}

fn qgrams<'a>(text: &'a str, q: usize, work: &mut Work) -> Result<Counts<'a>, OperationError> {
    let len = scalar_count(text, MAX_UNITS)?;
    let mut offsets = Vec::with_capacity(len + 1);
    offsets.extend(text.char_indices().map(|(i, _)| i));
    offsets.push(text.len());
    work.charge(text.len() + len)?;
    count(offsets.windows(q + 1).map(|w| &text[w[0]..w[q]]), work)
}

struct Totals {
    intersection: usize,
    left_only: usize,
    right_only: usize,
    l1: usize,
}

fn totals(
    left: &Counts<'_>,
    right: &Counts<'_>,
    work: &mut Work,
) -> Result<Totals, OperationError> {
    let mut totals = Totals {
        intersection: 0,
        left_only: 0,
        right_only: 0,
        l1: 0,
    };
    work.charge(left.len() + right.len())?;
    for (&key, &qty) in left {
        if let Some(&other) = right.get(key) {
            totals.intersection += 1;
            totals.l1 += qty.abs_diff(other);
        } else {
            totals.left_only += 1;
            totals.l1 += qty;
        }
    }
    for (&key, &qty) in right {
        if !left.contains_key(key) {
            totals.right_only += 1;
            totals.l1 += qty;
        }
    }
    if totals.intersection + totals.left_only + totals.right_only > DISTINCT {
        return Err(resource("combined inventory exceeds 10000 distinct keys"));
    }
    Ok(totals)
}

fn ratio(method: SetMethod, t: &Totals, alpha: Rational, beta: Rational) -> Option<(u128, u128)> {
    let (i, l, r) = (
        t.intersection as u128,
        t.left_only as u128,
        t.right_only as u128,
    );
    if matches!(method, SetMethod::QgramL1) {
        return None;
    }
    if i == 0 {
        return Some((u128::from(l + r == 0), 1));
    }
    Some(match method {
        SetMethod::Jaccard => (i, i + l + r),
        SetMethod::Dice => (2 * i, 2 * i + l + r),
        SetMethod::Overlap => (i, i + l.min(r)),
        _ => {
            let (a, b) = (u128::from(alpha.denominator), u128::from(beta.denominator));
            let numerator = i * a * b;
            (
                numerator,
                numerator
                    + l * u128::from(alpha.numerator) * b
                    + r * u128::from(beta.numerator) * a,
            )
        }
    })
}

/// Compare token sets or count overlapping q-gram differences.
pub fn set_similarity(input: &SetRequest) -> Result<SetReport, OperationError> {
    if !matches!(input.method, SetMethod::Tversky)
        && (input.alpha.is_some() || input.beta.is_some())
    {
        return Err(parameter("alpha and beta apply only to Tversky"));
    }
    let (alpha, beta) = (
        input.alpha.unwrap_or_default(),
        input.beta.unwrap_or_default(),
    );
    if alpha.denominator == 0
        || beta.denominator == 0
        || alpha.numerator == 0 && beta.numerator == 0
    {
        return Err(parameter(
            "Tversky requires positive denominators and at least one nonzero weight",
        ));
    }
    let mut work = Work(WORK);
    let (left, right) = match &input.input {
        Terms::Tokens { left, right } => (
            token_counts(left, &mut work)?,
            token_counts(right, &mut work)?,
        ),
        Terms::Qgrams { left, right, q } => {
            if !(1..=16).contains(q) {
                return Err(parameter("q must be in 1..=16"));
            }
            (qgrams(left, *q, &mut work)?, qgrams(right, *q, &mut work)?)
        }
    };
    let t = totals(&left, &right, &mut work)?;
    let exact = ratio(input.method, &t, alpha, beta);
    let similarity = exact.map(|(n, d)| n as f64 / d as f64);
    Ok(SetReport {
        method: input.method,
        intersection: t.intersection,
        union: t.intersection + t.left_only + t.right_only,
        left_only: t.left_only,
        right_only: t.right_only,
        l1: matches!(input.method, SetMethod::QgramL1).then_some(t.l1),
        fraction: exact.map(|(n, d)| Fraction {
            numerator: n.to_string(),
            denominator: d.to_string(),
        }),
        similarity,
        jaccard_distance: similarity
            .filter(|_| matches!(input.method, SetMethod::Jaccard))
            .map(|v| 1.0 - v),
    })
}
