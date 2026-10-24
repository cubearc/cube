//! Explicit Unicode text operations and shared limits for new text interfaces.
use crate::errors::{ErrorCode, OperationError};
/// Maximum UTF-8 input bytes for v0.5 text/symbol/cipher operations (1 MiB).
pub const MAX_INPUT: usize = 1024 * 1024;
/// Maximum UTF-8 output bytes, enforced while producing expanding results (8 MiB).
pub const MAX_OUTPUT: usize = 8 * 1024 * 1024;
pub(crate) fn check_input(input: &str, operation: &'static str) -> Result<(), OperationError> {
    if input.len() > MAX_INPUT {
        Err(
            OperationError::new(ErrorCode::ResourceLimit, "text input exceeds 1 MiB")
                .operation(operation),
        )
    } else {
        Ok(())
    }
}
