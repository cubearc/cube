//! Bounded classical-cipher evidence. Scores are heuristics, never probabilities
//! of correctness. Models, keys and configurations are always caller-owned.
pub mod candidates;
mod catalog;
pub mod classical;
pub mod crib;
mod fixtures;
pub mod frequency;
pub mod homophonic;
pub mod period;
pub mod scoring;
pub mod transposition;
pub mod vigenere;
use crate::ciphers::transposition::wire;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::sync::atomic::{AtomicBool, Ordering};

pub const MAX_TEXT: usize = 1 << 20;
pub const MAX_WORK: usize = 16 << 20;
pub const MAX_REPORT: usize = 4 << 20;
pub const MAX_REQUEST: usize = 4 << 20;

/// A caller can lower, but cannot raise, these ceilings.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    #[schemars(range(min = 0, max = 16777216))]
    #[serde(deserialize_with = "wire::integer")]
    pub max_work: usize,
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub max_results: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_results: 4096,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    Complete,
    WorkLimit,
    ResultLimit,
    Cancelled,
    Pruned,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Status {
    pub complete: bool,
    pub stop: Stop,
    pub work: usize,
}
pub(crate) struct Budget<'a> {
    pub status: Status,
    max: usize,
    cancel: Option<&'a AtomicBool>,
}
impl<'a> Budget<'a> {
    pub fn new(l: &Limits, cancel: Option<&'a AtomicBool>) -> Result<Self, OperationError> {
        bound(l.max_work, MAX_WORK)?;
        bound(l.max_results, 4096)?;
        if l.max_results == 0 {
            return Err(invalid("max_results must be positive"));
        }
        Ok(Self {
            status: Status {
                complete: true,
                stop: Stop::Complete,
                work: 0,
            },
            max: l.max_work,
            cancel,
        })
    }
    pub fn take(&mut self, n: usize) -> bool {
        if self.cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            self.stop(Stop::Cancelled);
            return false;
        }
        if n > self.max - self.status.work {
            self.stop(Stop::WorkLimit);
            return false;
        }
        self.status.work += n;
        true
    }
    pub fn stop(&mut self, reason: Stop) {
        self.status.complete = false;
        self.status.stop = reason;
    }
}
pub(crate) fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
pub(crate) fn bound(n: usize, max: usize) -> Result<(), OperationError> {
    if n > max {
        Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "declared cryptanalysis ceiling exceeded",
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn json_bound<T: Serialize>(value: &T, max: usize) -> Result<(), OperationError> {
    let mut writer = crate::text::mapping::ByteBudget::new(max);
    serde_json::to_writer(&mut writer, value).map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "serialized cryptanalysis value exceeds byte ceiling",
        )
    })
}
pub(crate) fn finish<T: Serialize>(report: T) -> Result<T, OperationError> {
    json_bound(&report, MAX_REPORT)?;
    let value = serde_json::to_value(&report).map_err(|_| invalid("invalid structured report"))?;
    json_bound(
        &crate::operations::contracts::canonical_value(value),
        MAX_REPORT,
    )?;
    Ok(report)
}
/// Bounded strict JSON transport. No duplicate keys, unknown fields or references.
pub fn parse_request<T: DeserializeOwned>(document: &str) -> Result<T, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_REQUEST, 16, 400000)
        .map_err(|e| OperationError::new(e.error.code, "invalid bounded cryptanalysis JSON"))?;
    serde_json::from_value(value).map_err(|_| invalid("use the exact typed cryptanalysis request"))
}
pub(crate) fn text_bound(text: &str, symbols: usize) -> Result<(), OperationError> {
    bound(text.len(), MAX_TEXT)?;
    bound(text.chars().count(), symbols)
}
/// ASCII letter view, with byte offsets in the unmodified original text.
pub(crate) fn letters(text: &str) -> (String, Vec<usize>) {
    text.char_indices()
        .filter(|(_, c)| c.is_ascii_alphabetic())
        .map(|(i, c)| (c.to_ascii_uppercase(), i))
        .unzip()
}
