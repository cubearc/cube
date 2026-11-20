//! Portable direct adapter to the owning formal operation implementations.
use super::*;

pub(super) fn supports(id: &str) -> bool {
    operations::formal::OPERATIONS
        .iter()
        .any(|row| row.id == id)
}
pub(super) fn execute(
    operation: &str,
    mut inputs: Value,
    _: Value,
    _: super::super::Budget,
    _: Option<super::super::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    operations::formal::invoke_with_cancel(operation, inputs["request"].take(), &mut || false)
        .map(|report| json!({"report":report}))
}
