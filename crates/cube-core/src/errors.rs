//! Stable error categories for new operations, without changing legacy variants.

use serde::Serialize;
use std::{error::Error, fmt, io};

/// Machine-readable failure category. New categories may be added compatibly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidInput,
    InvalidParameter,
    UnsupportedFormat,
    ResourceLimit,
    /// Cooperative pipeline cancellation at a bounded operation boundary.
    Cancelled,
    Io,
}

impl ErrorCode {
    /// Stable spelling used by SDK exception attributes and JSON results.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::InvalidParameter => "invalid_parameter",
            Self::UnsupportedFormat => "unsupported_format",
            Self::ResourceLimit => "resource_limit",
            Self::Cancelled => "cancelled",
            Self::Io => "io",
        }
    }
}

/// Operation failure with safe context and an optional original cause.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OperationError {
    pub code: ErrorCode,
    pub message: String,
    pub operation: Option<&'static str>,
    pub parameter: Option<&'static str>,
    pub offset: Option<u64>,
    #[serde(skip)]
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl OperationError {
    /// Construct an actionable error without including input contents.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            operation: None,
            parameter: None,
            offset: None,
            source: None,
        }
    }

    /// Attach a stable operation ID.
    pub fn operation(mut self, id: &'static str) -> Self {
        self.operation = Some(id);
        self
    }
    /// Attach the offending parameter name, never its potentially sensitive value.
    pub fn parameter(mut self, name: &'static str) -> Self {
        self.parameter = Some(name);
        self
    }
    /// Attach an absolute byte offset.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }
    /// Preserve an underlying I/O cause without embedding a path or input in the message.
    pub fn io(error: io::Error) -> Self {
        let mut result = Self::new(
            ErrorCode::Io,
            format!(
                "I/O failed ({:?}); check file access and destination",
                error.kind()
            ),
        );
        result.source = Some(Box::new(error));
        result
    }

    /// Retain a parser or transport cause without displaying its potentially sensitive text.
    pub fn caused_by(mut self, error: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(error));
        self
    }
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = self.operation {
            write!(f, "{id}: ")?;
        }
        if let Some(name) = self.parameter {
            write!(f, "parameter '{name}': ")?;
        }
        if let Some(offset) = self.offset {
            write!(f, "offset {offset}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl Error for OperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}
impl From<io::Error> for OperationError {
    fn from(error: io::Error) -> Self {
        Self::io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn safe_context_retains_original_cause() {
        let error = OperationError::io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "private filename",
        ))
        .operation("file.range.extract")
        .parameter("offset")
        .offset(17);
        assert_eq!(error.code, ErrorCode::Io);
        assert!(error.source().is_some());
        assert!(error.to_string().contains("offset 17"));
        assert!(!error.to_string().contains("private filename"));
        let error = OperationError::new(ErrorCode::InvalidInput, "invalid header");
        assert!(error.source().is_none());
        assert_eq!(error.to_string(), "invalid header");
    }
}
