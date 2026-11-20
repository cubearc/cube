//! Bounded work and cooperative cancellation for deterministic search.
use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    WorkLimit,
    Cancelled,
}

impl Stop {
    pub(crate) fn error(self) -> OperationError {
        match self {
            Self::WorkLimit => resource("work budget exhausted"),
            Self::Cancelled => OperationError::new(ErrorCode::Cancelled, "operation cancelled"),
        }
    }
}

pub(crate) struct Work<'a> {
    limit: usize,
    used: usize,
    cancelled: &'a mut dyn FnMut() -> bool,
}

impl<'a> Work<'a> {
    pub(crate) fn new(
        requested: Option<usize>,
        hard: usize,
        cancelled: &'a mut dyn FnMut() -> bool,
    ) -> Result<Self, OperationError> {
        let limit = requested.unwrap_or(hard);
        if limit > hard {
            return Err(parameter("requested work exceeds its hard ceiling"));
        }
        Ok(Self {
            limit,
            used: 0,
            cancelled,
        })
    }

    pub(crate) fn charge(&mut self, units: usize) -> Result<(), Stop> {
        if (self.cancelled)() {
            return Err(Stop::Cancelled);
        }
        self.used = self
            .used
            .checked_add(units)
            .filter(|&v| v <= self.limit)
            .ok_or(Stop::WorkLimit)?;
        Ok(())
    }

    pub(crate) fn used(&self) -> usize {
        self.used
    }
}

pub(crate) fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}

pub(crate) fn parameter(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}

pub(crate) fn resource(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}

/// Check compact JSON size without constructing an unbounded serialized copy.
pub(crate) fn check_bytes(value: &impl Serialize, maximum: usize) -> Result<(), OperationError> {
    struct Remaining(usize);
    impl std::io::Write for Remaining {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| std::io::Error::other("byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Remaining(maximum), value)
        .map_err(|_| resource("serialized document exceeds its byte budget"))
}
