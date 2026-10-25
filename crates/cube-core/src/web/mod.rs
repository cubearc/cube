//! Offline artifact observations. No URL/path resolution, browser, network or execution.
pub mod har;
pub mod html;
use crate::errors::{ErrorCode, OperationError};
pub const MAX_HTML_BYTES: usize = 1024 * 1024;
pub const MAX_FIELD: usize = 65536;
pub const MAX_RECORDS: usize = 256;
pub const MAX_OUTPUT_TEXT: usize = 128 * 1024;
fn invalid(op: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message).operation(op)
}
fn limit(op: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation(op)
}
fn unsupported(op: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message).operation(op)
}
/// Classification only; references remain inert strings and are never resolved.
pub fn reference_kind(value: &str) -> &'static str {
    let v = value.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if v.starts_with('#') {
        "fragment"
    } else if v.get(..5).is_some_and(|s| s.eq_ignore_ascii_case("data:")) {
        "data"
    } else if v.starts_with("//")
        || v.get(..7)
            .is_some_and(|s| s.eq_ignore_ascii_case("http://"))
        || v.get(..8)
            .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
    {
        "remote"
    } else if v.split_once(':').is_some_and(|(s, _)| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+.-".contains(&b))
    }) {
        "other_scheme"
    } else {
        "relative"
    }
}
fn observations(value: &str) -> Vec<&'static str> {
    let mut result = Vec::new();
    if value.chars().any(char::is_control) {
        result.push("contains_control_characters")
    }
    if value.split('/').any(|s| s == "..") {
        result.push("contains_parent_path_segment")
    }
    if reference_kind(value) == "other_scheme" {
        result.push("non_http_scheme")
    }
    result
}
