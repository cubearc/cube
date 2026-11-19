//! Bounded exact arithmetic, combinatorics, and empirical numeric summaries.
pub mod arithmetic;
mod catalog;
pub mod combinatorial;
pub mod letter_values;
pub mod linear;
mod rational;
pub mod representations;
pub mod statistics;

pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

use crate::errors::{ErrorCode, OperationError};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{borrow::Cow, io};

pub const MAX_DOCUMENT: usize = 12 << 20;

pub fn parse(document: &str) -> Result<Value, OperationError> {
    crate::pipeline::json::parse_with_limits(document, MAX_DOCUMENT, 16, 1_100_000)
        .map_err(|failure| (*failure.error).operation("number.analysis"))
}

pub fn invoke(operation: &str, request: Value) -> Result<Value, OperationError> {
    catalog::invoke(operation, request)
}

/// Invoke a direct operation over lossless JSON with the native host ceilings.
pub fn invoke_transport(operation: &str, request: Value) -> Result<Value, OperationError> {
    use crate::operations::{
        contracts::canonical_value,
        execution::{self, Budget, InvocationError},
    };
    if !OPERATIONS.iter().any(|row| row.id == operation) {
        return Err(invalid("unknown numeric operation").operation("number.analysis"));
    }
    let budget = Budget {
        input_bytes: MAX_DOCUMENT,
        output_bytes: 8 << 20,
        ..Budget::default()
    };
    let mut outputs = execution::invoke(
        operation,
        1,
        &serde_json::json!({"request": canonical_value(request)}),
        &serde_json::json!({}),
        budget,
    )
    .map_err(|e| match e {
        InvocationError::Operation(error) => error,
        InvocationError::Code(_) => {
            invalid("numeric execution failed").operation("number.analysis")
        }
    })?;
    Ok(outputs["report"].take())
}

pub fn command(name: &str) -> Option<&'static str> {
    OPERATIONS.iter().find_map(|row| {
        (row.command.strip_prefix("number analysis ") == Some(name)).then_some(row.id)
    })
}

fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}

fn parameter(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}

fn resource(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}

fn overflow() -> OperationError {
    resource("exact arithmetic exceeds the supported width")
}

struct Work(usize);

impl Work {
    fn charge(&mut self, visits: usize) -> Result<(), OperationError> {
        self.0 = self
            .0
            .checked_sub(visits)
            .ok_or_else(|| resource("work budget exhausted"))?;
        Ok(())
    }
}

pub(super) fn check_report(report: &impl Serialize, maximum: usize) -> Result<(), OperationError> {
    struct Budget(usize);
    impl io::Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| io::Error::other("report limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(maximum), report)
        .map_err(|_| resource("report exceeds its serialized byte budget"))
}

macro_rules! decimal {
    ($name:ident, $primitive:ty, $pattern:literal, $max:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name(pub $primitive);
        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> { stringify!($name).into() }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({"type":"string","pattern":$pattern,"maxLength":$max})
            }
        }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(&self.0)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self,D::Error> {
                let text = String::deserialize(deserializer)?;
                if text.len() > $max {
                    return Err(serde::de::Error::custom("exact decimal exceeds the supported width"));
                }
                let parsed = text.parse::<$primitive>().map_err(serde::de::Error::custom)?;
                if parsed.to_string() != text {
                    return Err(serde::de::Error::custom("expected canonical decimal digits"));
                }
                Ok(Self(parsed))
            }
        }
    };
}
decimal!(Unsigned, u128, r"^(0|[1-9][0-9]*)$", 39);
decimal!(Signed, i128, r"^(0|-?[1-9][0-9]*)$", 40);
