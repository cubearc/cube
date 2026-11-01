//! Explicit paired alphabets and segmented historical codebooks.
//! No language guessing, implicit normalization, dictionaries or random state.
mod catalog;
mod codebook;
mod fixtures;
mod profiles;
use super::{bound, error};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::{OPERATIONS, execute};
pub use codebook::{Book, DecodeReport, Definition, Entry, Unit};
pub use profiles::{Syllabary, great_cipher, malespin, vatsyayana};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_BYTES: usize = 1024 * 1024;
pub const MAX_ITEMS: usize = 65536;
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_UNIT_BYTES: usize = 256;
const CONFIG: &str = "cipher.nomenclator.encode";

pub(crate) fn invalid(parameter: &'static str) -> OperationError {
    error(
        CONFIG,
        ErrorCode::InvalidParameter,
        parameter,
        "invalid explicit historical profile",
    )
}
pub(crate) fn input_error(parameter: &'static str, index: usize) -> OperationError {
    error(
        CONFIG,
        ErrorCode::InvalidInput,
        parameter,
        "unknown unit/code or unmatched cancellation",
    )
    .offset(index as u64)
}
pub(crate) fn size(value: &impl Serialize) -> Result<(), OperationError> {
    super::homophonic::serialized_size(value, MAX_BYTES, CONFIG)
}
pub(crate) fn token(value: &str) -> Result<(), OperationError> {
    bound(CONFIG, "unit", value.len(), MAX_UNIT_BYTES)?;
    if value.is_empty() {
        return Err(invalid("unit"));
    }
    Ok(())
}
pub(crate) fn items(values: &[String]) -> Result<(), OperationError> {
    bound(CONFIG, "items", values.len(), MAX_ITEMS)?;
    size(&values)?;
    values.iter().try_for_each(|v| token(v))
}
/// Strict bounded JSON with duplicate-key rejection; no host capabilities.
pub fn parse<T: serde::de::DeserializeOwned>(document: &str) -> Result<T, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_BYTES, 8, 200000)
        .map_err(|e| error(CONFIG, e.error.code, "document", "invalid bounded JSON"))?;
    serde_json::from_value(value).map_err(|_| invalid("document"))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes" = MAX_BYTES))]
pub struct Tokens(
    #[schemars(length(max = 65536), with = "Vec<super::homophonic::TokenText>")] pub Vec<String>,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pairs {
    #[schemars(length(min = 1, max = 2048))]
    pub pairs: Vec<[char; 2]>,
    #[serde(default)]
    pub unmapped: super::params::Unmapped,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BookParameters {
    pub definition: Definition,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SyllabaryParameters {
    pub definition: Syllabary,
}
