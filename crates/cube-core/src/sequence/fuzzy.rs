//! Scalar Jaro profiles and directional soft token matching.
use super::*;
use serde::Serialize;

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JaroMethod {
    #[default]
    Jaro,
    JaroWinkler,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JaroRequest {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub method: JaroMethod,
    #[serde(default = "scale")]
    #[schemars(range(min = 0.0, max = 0.25))]
    pub scale: f64,
    #[serde(default = "threshold")]
    #[schemars(range(min = 0.0, max = 1.0))]
    pub threshold: f64,
}

const fn scale() -> f64 {
    0.1
}
const fn threshold() -> f64 {
    0.7
}

#[derive(Serialize, JsonSchema)]
pub struct JaroReport {
    pub method: JaroMethod,
    pub similarity: f64,
    pub distance: f64,
    pub scale: f64,
    pub threshold: f64,
}

fn jaro_value(left: &[u32], right: &[u32], work: &mut Work) -> Result<f64, OperationError> {
    if left.is_empty() || right.is_empty() {
        return Ok(f64::from(u8::from(left.is_empty() && right.is_empty())));
    }
    let radius = (left.len().max(right.len()) / 2).saturating_sub(1);
    let window =
        |i: usize| i.saturating_sub(radius).min(right.len())..(i + radius + 1).min(right.len());
    work.charge(left.len() + right.len())?;
    work.charge((0..left.len()).map(|i| window(i).len()).sum())?;
    let mut matched = vec![false; right.len()];
    let mut ordered = Vec::new();
    for (i, &value) in left.iter().enumerate() {
        if let Some(j) = window(i).find(|&j| !matched[j] && right[j] == value) {
            matched[j] = true;
            ordered.push(value);
        }
    }
    if ordered.is_empty() {
        return Ok(0.0);
    }
    let observed = right
        .iter()
        .zip(matched)
        .filter_map(|(&value, used)| used.then_some(value));
    let changed = ordered
        .iter()
        .copied()
        .zip(observed)
        .filter(|(a, b)| a != b)
        .count();
    let count = ordered.len() as f64;
    Ok((count / left.len() as f64
        + count / right.len() as f64
        + (count - changed as f64 / 2.0) / count)
        / 3.0)
}

fn boost(left: &[u32], right: &[u32], value: f64, scale: f64, threshold: f64) -> f64 {
    if value <= threshold {
        return value;
    }
    let prefix = left
        .iter()
        .zip(right)
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();
    value + (1.0 - value) * prefix as f64 * scale
}

/// Score an exact scalar pair with a pinned matching and optional prefix rule.
pub fn jaro(input: &JaroRequest) -> Result<JaroReport, OperationError> {
    if !(0.0..=0.25).contains(&input.scale) || !(0.0..=1.0).contains(&input.threshold) {
        return Err(parameter(
            "Jaro–Winkler scale or threshold is outside its allowed interval",
        ));
    }
    let left = scalars(&input.left, MAX_UNITS)?;
    let right = scalars(&input.right, MAX_UNITS)?;
    let mut score = jaro_value(&left, &right, &mut Work(WORK))?;
    if matches!(input.method, JaroMethod::JaroWinkler) {
        score = boost(&left, &right, score, input.scale, input.threshold);
    }
    Ok(JaroReport {
        method: input.method,
        similarity: score,
        distance: 1.0 - score,
        scale: input.scale,
        threshold: input.threshold,
    })
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenBase {
    Jaro,
    JaroWinkler,
    NormalizedLevenshtein,
}
#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    LeftToRight,
    Symmetric,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MongeRequest {
    #[schemars(length(max = 256))]
    pub left: Vec<String>,
    #[schemars(length(max = 256))]
    pub right: Vec<String>,
    pub base: TokenBase,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Serialize, JsonSchema)]
pub struct TokenMatch {
    pub index: Option<usize>,
    pub similarity: f64,
}
#[derive(Serialize, JsonSchema)]
pub struct MongeReport {
    pub base: TokenBase,
    pub direction: Direction,
    pub similarity: f64,
    pub matches: Vec<TokenMatch>,
    pub reverse_matches: Option<Vec<TokenMatch>>,
}

fn term_score(
    a: &[u32],
    b: &[u32],
    base: TokenBase,
    work: &mut Work,
) -> Result<f64, OperationError> {
    match base {
        TokenBase::NormalizedLevenshtein => {
            work.charge(a.len() * b.len() + a.len() + b.len() + 1)?;
            let max = a.len().max(b.len());
            Ok(if max == 0 {
                1.0
            } else {
                1.0 - edit::levenshtein(a, b) as f64 / max as f64
            })
        }
        TokenBase::Jaro => jaro_value(a, b, work),
        TokenBase::JaroWinkler => Ok(boost(a, b, jaro_value(a, b, work)?, scale(), threshold())),
    }
}

fn directed(
    left: &[Vec<u32>],
    right: &[Vec<u32>],
    base: TokenBase,
    work: &mut Work,
) -> Result<(f64, Vec<TokenMatch>), OperationError> {
    let mut total = 0.0;
    let mut matches = Vec::with_capacity(left.len());
    for a in left {
        let mut best = TokenMatch {
            index: None,
            similarity: 0.0,
        };
        for (j, b) in right.iter().enumerate() {
            let score = term_score(a, b, base, work)?;
            if best.index.is_none() || score > best.similarity {
                best = TokenMatch {
                    index: Some(j),
                    similarity: score,
                };
            }
        }
        total += best.similarity;
        matches.push(best);
    }
    let score = if left.is_empty() {
        f64::from(u8::from(right.is_empty()))
    } else {
        total / left.len() as f64
    };
    Ok((score, matches))
}

/// Score token arrays with deterministic first-occurrence best matches.
pub fn monge_elkan(input: &MongeRequest) -> Result<MongeReport, OperationError> {
    let (left, right) = (&input.left, &input.right);
    if left.len().max(right.len()) > 256
        || left.iter().chain(right).map(String::len).sum::<usize>() > 65536
    {
        return Err(resource("Monge–Elkan exceeds token or total byte bounds"));
    }
    let pair_count = left.len()
        * right.len()
        * if matches!(input.direction, Direction::Symmetric) {
            2
        } else {
            1
        };
    if pair_count > 65536 {
        return Err(resource("Monge–Elkan exceeds the token-pair budget"));
    }
    let tokenize = |terms: &[String]| {
        terms
            .iter()
            .map(|s| scalars(s, 64))
            .collect::<Result<Vec<_>, _>>()
    };
    let (left, right) = (tokenize(left)?, tokenize(right)?);
    let mut work = Work(1 << 23);
    let (mut similarity, matches) = directed(&left, &right, input.base, &mut work)?;
    let reverse_matches = if matches!(input.direction, Direction::Symmetric) {
        let (reverse, choices) = directed(&right, &left, input.base, &mut work)?;
        similarity = (similarity + reverse) / 2.0;
        Some(choices)
    } else {
        None
    };
    Ok(MongeReport {
        base: input.base,
        direction: input.direction,
        similarity,
        matches,
        reverse_matches,
    })
}
