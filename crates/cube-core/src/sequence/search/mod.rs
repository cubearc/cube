//! Explicit exact and approximate byte matching.
mod aho;
mod bitap;
mod exact;

pub use aho::{MultiMatch, MultiReport, MultiRequest, multi_search};
pub use bitap::{BitapMode, BitapReport, BitapRequest, EndingMatch, bitap};
pub use exact::{ExactAlgorithm, ExactReport, ExactRequest, exact_search};

use super::*;
use serde::Serialize;

const HAYSTACK: usize = 16 << 20;
const PATTERN: usize = 65536;
const RESULTS: usize = 65536;
const TRANSITIONS: usize = 1 << 26;

pub(super) fn bytes(value: &str, max: usize) -> Result<Vec<u8>, OperationError> {
    if value.len() > base64::encoded_len(max, true).unwrap() {
        return Err(resource("encoded byte buffer exceeds its bound"));
    }
    let bytes = crate::encoding::base64_decode(value)
        .map_err(|_| invalid("byte buffers require canonical padded Base64"))?;
    if bytes.len() > max {
        return Err(resource("decoded byte buffer exceeds its bound"));
    }
    Ok(bytes)
}

const fn default_results() -> usize {
    1000
}

fn matches_limit(maximum: usize) -> Result<(), OperationError> {
    if maximum > RESULTS {
        return Err(parameter("max_results exceeds 65536"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}
