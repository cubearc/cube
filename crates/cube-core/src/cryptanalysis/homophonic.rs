//! Exact caller-token counts; optional grouping remains a separate hypothesis.
use super::*;
use crate::ciphers::mono::homophonic as h;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub tokens: Vec<String>,
    #[serde(default)]
    pub groups: Vec<h::Entry>,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Count {
    pub token: String,
    pub count: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GroupCount {
    pub symbol: char,
    pub count: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Edge {
    pub left: String,
    pub right: String,
    pub count: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Coincidence {
    pub numerator: u64,
    pub denominator: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub status: Status,
    pub processed: usize,
    pub observed: Vec<Count>,
    pub adjacency: Vec<Edge>,
    pub token_ic: Option<Coincidence>,
    pub hypotheses: Vec<GroupCount>,
    pub grouped_ic: Option<Coincidence>,
    pub unmapped: usize,
    pub group_definition_sha256: String,
}
fn coincidence(counts: impl Iterator<Item = u64>) -> Option<Coincidence> {
    let (mut n, mut numerator) = (0, 0);
    for c in counts {
        n += c;
        numerator += c * c.saturating_sub(1);
    }
    (n > 1).then(|| Coincidence {
        numerator,
        denominator: n * (n - 1),
    })
}
/// Mapping validation is the same as the existing homophonic cipher, including
/// unique ownership. Unmapped tokens stay observed and are counted separately.
pub fn inspect(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    json_bound(&r.groups, 64 << 10)?;
    bound(r.tokens.len(), 65536)?;
    json_bound(&r.tokens, MAX_TEXT)?;
    let distinct: BTreeSet<_> = r.tokens.iter().collect();
    bound(distinct.len(), 4096)?;
    if distinct.iter().any(|s| s.is_empty() || s.len() > 256) {
        return Err(invalid(
            "tokens must be nonempty and at most 256 UTF-8 bytes",
        ));
    }
    if !r.groups.is_empty() {
        h::Mapping::new(h::Definition {
            ascii_uppercase: false,
            entries: r.groups.clone(),
        })?;
    }
    let owners: BTreeMap<_, _> = r
        .groups
        .iter()
        .flat_map(|e| e.tokens.iter().map(move |t| (t.as_str(), e.symbol)))
        .collect();
    let mut b = Budget::new(&r.limits, cancel)?;
    let mut counts = BTreeMap::new();
    let mut pairs = BTreeMap::new();
    let mut groups = BTreeMap::new();
    let mut processed = 0;
    let mut unmapped = 0;
    for (i, token) in r.tokens.iter().enumerate() {
        let new_token = !counts.contains_key(token.as_str());
        let new_pair = i > 0 && !pairs.contains_key(&(r.tokens[i - 1].as_str(), token.as_str()));
        if (new_token && counts.len() == r.limits.max_results)
            || (new_pair && pairs.len() == r.limits.max_results)
        {
            b.stop(Stop::ResultLimit);
            break;
        }
        if !b.take(2 + usize::from(i > 0)) {
            break;
        }
        *counts.entry(token.as_str()).or_insert(0u64) += 1;
        if i > 0 {
            *pairs
                .entry((r.tokens[i - 1].as_str(), token.as_str()))
                .or_insert(0u64) += 1;
        }
        if let Some(&group) = owners.get(token.as_str()) {
            *groups.entry(group).or_insert(0u64) += 1;
        } else {
            unmapped += 1;
        }
        processed += 1;
    }
    let token_ic = coincidence(counts.values().copied());
    let grouped_ic = coincidence(groups.values().copied());
    let observed = counts
        .into_iter()
        .map(|(token, count)| Count {
            token: token.into(),
            count,
        })
        .collect();
    let adjacency = pairs
        .into_iter()
        .map(|((left, right), count)| Edge {
            left: left.into(),
            right: right.into(),
            count,
        })
        .collect();
    // Caller group order, including declared but unobserved groups, is provenance.
    let hypotheses = r
        .groups
        .iter()
        .map(|g| GroupCount {
            symbol: g.symbol,
            count: groups.get(&g.symbol).copied().unwrap_or(0),
        })
        .collect();
    finish(Report {
        status: b.status,
        processed,
        observed,
        adjacency,
        token_ic,
        hypotheses,
        grouped_ic,
        unmapped,
        group_definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&r.groups).expect("typed groups"),
        ),
    })
}
