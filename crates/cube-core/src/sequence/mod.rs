//! Exact comparisons over caller-provided sequences without text preparation.
mod alignment;
mod catalog;
pub mod diff;
mod dtw;
mod edit;
mod fuzzy;
mod hamming;
pub mod inventory;
pub mod search;
mod spans;
pub mod tfidf;
pub mod vector;

pub use alignment::{AlignReport, AlignRequest, EditKind, Gap, Mode, Run, align};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub use dtw::{Cost, DtwReport, DtwRequest, dtw};
pub use edit::{EditMethod, EditReport, EditRequest, edit_distance};
pub use fuzzy::{
    Direction, JaroMethod, JaroReport, JaroRequest, MongeReport, MongeRequest, TokenBase,
    TokenMatch, jaro, monge_elkan,
};
pub use hamming::{HammingReport, HammingRequest, hamming};
pub use spans::{
    CommonSpans, LcsReport, PairedSpan, RatcliffReport, Span, Subsequence, common_spans, lcs,
    ratcliff_obershelp,
};

use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Deserialize;

const TEXT_BYTES: usize = 1 << 20;
const MAX_UNITS: usize = 65_536;
const DP_CELLS: usize = 1 << 22;
const WORK: usize = 1 << 24;

/// A pair of exact byte sequences (canonical padded Base64) or scalar strings.
#[derive(Clone, Deserialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pair {
    Bytes { left: String, right: String },
    Scalars { left: String, right: String },
}

impl Pair {
    fn symbols(&self, limit: usize) -> Result<(Vec<u32>, Vec<u32>), OperationError> {
        match self {
            Self::Bytes { left, right } => {
                let bytes = |wire: &str| {
                    if wire.len() > base64::encoded_len(limit, true).unwrap() {
                        return Err(resource("byte sequence exceeds the unit limit"));
                    }
                    let bytes = crate::encoding::base64_decode(wire)
                        .map_err(|_| invalid("byte sequences require canonical padded Base64"))?;
                    if bytes.len() > limit {
                        return Err(resource("byte sequence exceeds the unit limit"));
                    }
                    Ok(bytes.into_iter().map(u32::from).collect())
                };
                Ok((bytes(left)?, bytes(right)?))
            }
            Self::Scalars { left, right } => Ok((scalars(left, limit)?, scalars(right, limit)?)),
        }
    }
}

fn scalar_count(text: &str, limit: usize) -> Result<usize, OperationError> {
    if text.len() > TEXT_BYTES {
        return Err(resource("UTF-8 sequence exceeds the byte limit"));
    }
    let count = text.chars().count();
    if count > limit {
        return Err(resource("UTF-8 sequence exceeds the scalar limit"));
    }
    Ok(count)
}

fn scalars(text: &str, limit: usize) -> Result<Vec<u32>, OperationError> {
    let mut result = Vec::with_capacity(scalar_count(text, limit)?);
    result.extend(text.chars().map(u32::from));
    Ok(result)
}

fn invalid(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}

fn parameter(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}

fn resource(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}

fn cells(rows: usize, cols: usize, maximum: usize) -> Result<usize, OperationError> {
    rows.checked_mul(cols)
        .filter(|&size| size <= maximum)
        .ok_or_else(|| resource("sequence matrix exceeds the cell budget"))
}

struct Work(usize);

impl Work {
    fn charge(&mut self, steps: usize) -> Result<(), OperationError> {
        self.0 = self
            .0
            .checked_sub(steps)
            .ok_or_else(|| resource("sequence computation exceeds the work budget"))?;
        Ok(())
    }
}

/// Maximum JSON transport bytes before operation-specific shape and bounds checks.
pub const MAX_DOCUMENT: usize = 46 << 20;

/// Run a typed sequence analysis from its canonical JSON request.
pub fn invoke(
    operation: &str,
    request: serde_json::Value,
) -> Result<serde_json::Value, OperationError> {
    catalog::invoke(operation, request)
}

/// Resolve the exact name of a sequence analysis from the CLI subcommand vocabulary.
pub fn command(name: &str) -> Option<&'static str> {
    OPERATIONS.iter().find_map(|op| {
        (op.id
            .strip_prefix("sequence.")
            .is_some_and(|suffix| suffix.replace('_', "-") == name))
        .then_some(op.id)
    })
}

/// Parse exact inputs with duplicate-key, transport byte, depth and node limits.
pub fn parse(document: &str) -> Result<serde_json::Value, OperationError> {
    crate::pipeline::json::parse_with_limits(document, MAX_DOCUMENT, 12, 262_144)
        .map_err(|error| (*error.error).operation("sequence"))
}

struct SerializedSize(usize);
impl std::io::Write for SerializedSize {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| std::io::Error::other("sequence report byte limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serialized_bound(
    value: &impl serde::Serialize,
    size: usize,
    message: &'static str,
) -> Result<(), OperationError> {
    serde_json::to_writer(SerializedSize(size), value).map_err(|_| resource(message))
}
