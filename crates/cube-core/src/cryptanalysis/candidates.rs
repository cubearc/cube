//! Concrete classical key provenance and bounded exact-plaintext retention.
use super::*;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Key {
    Caesar {
        shift: i32,
    },
    Affine {
        a: i32,
        b: i32,
    },
    Vigenere {
        key: String,
    },
    Transposition {
        configuration: transposition::Configuration,
    },
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Alias {
    pub input_index: usize,
    pub key: Key,
    pub source_positions: Vec<usize>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Candidate {
    pub plaintext: String,
    pub score: scoring::Score,
    pub aliases: Vec<Alias>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Rejected {
    pub input_index: usize,
    pub reason: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub method: String,
    pub normalization: scoring::Normalization,
    pub model_sha256: String,
    pub status: Status,
    pub attempted: usize,
    pub candidates: Vec<Candidate>,
    pub rejected: Vec<Rejected>,
}
// This stores already-computed results; it never enumerates keys or dispatches an operation.
pub(crate) struct Retained {
    pub candidates: Vec<Candidate>,
    pub rejected: Vec<Rejected>,
    index: BTreeMap<String, usize>,
    bytes: usize,
    maximum: usize,
}
impl Retained {
    pub fn new(maximum: usize) -> Self {
        Self {
            candidates: vec![],
            rejected: vec![],
            index: BTreeMap::new(),
            bytes: 0,
            maximum,
        }
    }
    pub fn reject(&mut self, index: usize) {
        self.rejected.push(Rejected {
            input_index: index,
            reason: "invalid explicit cipher key/configuration".into(),
        });
    }
    pub fn accept(
        &mut self,
        text: String,
        score: scoring::Score,
        alias: Alias,
        b: &mut Budget<'_>,
    ) -> bool {
        let alias_bytes = serde_json::to_vec(&alias)
            .expect("finite typed alias")
            .len()
            + 1;
        let additional = if self.index.contains_key(&text) {
            alias_bytes
        } else {
            text.len() * 6 + alias_bytes + 256
        };
        if self.bytes + additional > 3 << 20 {
            b.stop(Stop::ResultLimit);
            return false;
        }
        if let Some(&i) = self.index.get(&text) {
            self.candidates[i].aliases.push(alias);
        } else {
            if self.candidates.len() == self.maximum {
                b.stop(Stop::ResultLimit);
                return false;
            }
            self.index.insert(text.clone(), self.candidates.len());
            self.candidates.push(Candidate {
                plaintext: text,
                score,
                aliases: vec![alias],
            });
        }
        self.bytes += additional;
        true
    }
    pub fn report(
        mut self,
        model: scoring::Prepared,
        b: Budget<'_>,
        attempted: usize,
    ) -> Result<Report, OperationError> {
        self.candidates
            .sort_by_key(|c| std::cmp::Reverse(c.score.mean_nanos));
        finish(Report {
            method: model.method,
            normalization: model.normalization,
            model_sha256: model.identity,
            status: b.status,
            attempted,
            candidates: self.candidates,
            rejected: self.rejected,
        })
    }
}
