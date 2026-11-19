//! Typed direct numeric reports backed by the same Core entry as native callers.
use super::*;

pub(super) fn supports(operation: &str) -> bool {
    crate::numbers::analysis::OPERATIONS
        .iter()
        .any(|row| row.id == operation)
}

pub(super) fn execute(
    operation: &str,
    mut inputs: Value,
    _: Value,
    _: super::super::Budget,
    _: Option<super::super::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    crate::numbers::analysis::invoke(operation, inputs["request"].take())
        .map(|report| json!({"report": report}))
}
