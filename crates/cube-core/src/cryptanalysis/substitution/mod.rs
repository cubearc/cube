//! Deterministic finite bijection constraints. Unobserved key completions are
//! exact permutation classes, not evidence that a particular key is correct.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
mod prepare;
mod search;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    ObservedBijectionV1,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outside {
    Preserve,
    Reject,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub cipher: char,
    pub plain: char,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    pub alias: char,
    pub symbol: char,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Segmentation {
    UnicodeWhitespace,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dictionary {
    pub segmentation: Segmentation,
    #[schemars(length(max = 4096))]
    pub words: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct SearchLimits {
    #[schemars(range(min = 0, max = 1048576))]
    #[serde(deserialize_with = "wire::integer")]
    pub max_work: usize,
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub max_results: usize,
    #[schemars(range(min = 0, max = 67108864))]
    #[serde(deserialize_with = "wire::integer")]
    pub max_memory_bytes: usize,
}
impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            max_work: 1 << 20,
            max_results: 128,
            max_memory_bytes: 64 << 20,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub profile: Profile,
    #[schemars(length(min = 26, max = 64))]
    pub alphabet: String,
    #[schemars(length(max = 65536))]
    pub ciphertext: String,
    pub outside: Outside,
    #[serde(default)]
    #[schemars(length(max = 4096))]
    pub cribs: Vec<Pair>,
    #[serde(default)]
    #[schemars(length(max = 256))]
    pub aliases: Vec<Alias>,
    #[serde(default)]
    pub dictionary: Option<Dictionary>,
    #[serde(default)]
    pub model: Option<scoring::Model>,
    #[serde(default)]
    pub limits: SearchLimits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DictionaryClass {
    pub word: String,
    pub input_indices: Vec<usize>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Constraint {
    pub token: String,
    pub original_byte_offsets: Vec<usize>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AliasUse {
    pub alias: char,
    pub symbol: char,
    pub occurrences: usize,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Candidate {
    pub plaintext: String,
    /// Targets in request alphabet order; one canonical complete key in this class.
    pub canonical_key: String,
    pub mapping: Vec<Pair>,
    pub free_cipher: String,
    pub free_plain: String,
    /// Exact number of full keys represented, including this canonical key.
    pub key_count: String,
    /// Dictionary class index for each report constraint, in constraint order.
    pub word_choices: Vec<usize>,
    pub score: Option<scoring::Score>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub method: Profile,
    pub request_sha256: String,
    pub model_sha256: Option<String>,
    pub status: Status,
    pub contradiction: bool,
    pub plaintext_unique: Option<bool>,
    pub key_unique: Option<bool>,
    pub workspace_bytes_upper_bound: usize,
    pub aliases: Vec<AliasUse>,
    pub dictionary_classes: Vec<DictionaryClass>,
    pub constraints: Vec<Constraint>,
    pub candidates: Vec<Candidate>,
    /// Candidate indices by descending optional model score; stable search order on ties.
    pub ranking: Vec<usize>,
}
/// Solve exact caller-owned constraints. Search order is deterministic; limits
/// return a marked prefix. Contradiction requires a proof, never a timeout.
/// Dictionary words are exact whitespace-delimited tokens, including punctuation.
pub fn solve(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    bound(r.limits.max_work, 1 << 20)?;
    bound(r.limits.max_memory_bytes, 64 << 20)?;
    let limits = Limits {
        max_work: r.limits.max_work,
        max_results: r.limits.max_results,
    };
    let mut budget = Budget::new(&limits, cancel)?;
    let model = r.model.as_ref().map(scoring::Prepared::new).transpose()?;
    let mut p = prepare::Prepared::new(r, &mut budget)?;
    let metadata = serde_json::to_value((&p.classes, &p.constraints, &p.alias_uses))
        .map_err(|_| invalid("invalid constraints"))?;
    json_bound(&metadata, (1 << 20) - 8192)?;
    json_bound(
        &crate::operations::contracts::canonical_value(metadata),
        (1 << 20) - 8192,
    )?;
    let mut candidates = Vec::new();
    if budget.status.complete && !p.conflict {
        search::run(r, &p, model.as_ref(), &mut budget, &mut candidates)?;
    }
    let contradiction = p.conflict || (budget.status.complete && candidates.is_empty());
    let plaintext_unique = if candidates.len() > 1 {
        Some(false)
    } else if budget.status.complete {
        Some(candidates.len() == 1)
    } else {
        None
    };
    let key_unique =
        if candidates.len() > 1 || candidates.first().is_some_and(|c| c.key_count != "1") {
            Some(false)
        } else if budget.status.complete {
            Some(candidates.len() == 1)
        } else {
            None
        };
    let mut ranking: Vec<_> = (0..candidates.len()).collect();
    ranking.sort_by_key(|&i| {
        std::cmp::Reverse(candidates[i].score.as_ref().and_then(|s| s.mean_nanos))
    });
    finish(Report {
        method: r.profile,
        request_sha256: crate::hashing::sha256(
            &serde_json::to_vec(r).map_err(|_| invalid("invalid request"))?,
        ),
        model_sha256: model.map(|m| m.identity),
        status: budget.status,
        contradiction,
        plaintext_unique,
        key_unique,
        workspace_bytes_upper_bound: p.memory,
        aliases: std::mem::take(&mut p.alias_uses),
        dictionary_classes: std::mem::take(&mut p.classes),
        constraints: std::mem::take(&mut p.constraints),
        candidates,
        ranking,
    })
}
