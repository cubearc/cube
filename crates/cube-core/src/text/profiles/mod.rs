//! Bounded text transformations with explicit scalar, grapheme and lexical profiles.
pub mod beider_morse;
pub mod case;
mod catalog;
pub mod daitch_mokotoff;
pub mod graphemes;
pub mod lexical;
pub mod match_rating;
pub mod phonetic;
pub mod regex;
pub mod scalar_weave;

pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;

fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}

fn parameter(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}

fn resource(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}

fn check_report(report: &impl Serialize, limit: usize) -> Result<(), OperationError> {
    let mut writer = super::mapping::ByteBudget::new(limit);
    serde_json::to_writer(&mut writer, report)
        .map_err(|_| resource("report exceeds its byte limit"))
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SourceSpan {
    pub byte_start: usize,
    pub byte_end: usize,
    pub scalar_start: usize,
    pub scalar_end: usize,
}

/// Maximum exact JSON document size, including string escapes and signed coordinates.
pub const MAX_DOCUMENT: usize = 9 << 20;

/// Parse a bounded profile request with duplicate-field and nesting checks.
pub fn parse(document: &str) -> Result<serde_json::Value, OperationError> {
    crate::pipeline::json::parse_with_limits(document, MAX_DOCUMENT, 12, 262_144)
        .map_err(|error| (*error.error).operation("text.profile"))
}

/// Execute a profile on its typed JSON document.
pub fn invoke(
    operation: &str,
    value: serde_json::Value,
) -> Result<serde_json::Value, OperationError> {
    catalog::invoke(operation, value)
}

/// Resolve an explicit CLI command from the profile descriptors.
pub fn command(name: &str) -> Option<&'static str> {
    OPERATIONS
        .iter()
        .find_map(|op| (op.command.strip_prefix("text profile ") == Some(name)).then_some(op.id))
}
