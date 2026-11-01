//! Explicit numerical puzzle profiles over shared Core arithmetic and mappings.
mod catalog;
mod fixtures;
pub mod modulo;
pub mod prime;
pub mod product;
pub mod triliteral;
use super::{
    MAX_DEFINITION, MAX_INPUT, MAX_OUTPUT, MAX_TOKENS, alphabet, bound, error, homophonic,
};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes"=1048576))]
pub struct Tokens(
    #[schemars(length(max = 65536), with = "Vec<homophonic::TokenText>")] pub Vec<String>,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Encoded {
    pub tokens: Tokens,
    #[schemars(length(min = 1, max = 4096))]
    pub table: Vec<homophonic::Entry>,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decoded {
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub text: String,
    #[schemars(length(min = 1, max = 4096))]
    pub table: Vec<homophonic::Entry>,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
fn invalid(op: &'static str, name: &'static str) -> OperationError {
    error(
        op,
        ErrorCode::InvalidParameter,
        name,
        "invalid explicit numeric puzzle profile",
    )
}
fn parameters(value: &impl Serialize, op: &'static str) -> Result<(), OperationError> {
    homophonic::serialized_size(value, MAX_DEFINITION, op)
}
fn text_input(text: &str, op: &'static str) -> Result<(), OperationError> {
    bound(op, "text", text.len(), MAX_INPUT)?;
    bound(
        op,
        "text",
        text.chars().take(MAX_TOKENS + 1).count(),
        MAX_TOKENS,
    )
}
fn token_input(tokens: &[String], op: &'static str) -> Result<(), OperationError> {
    bound(op, "tokens", tokens.len(), MAX_TOKENS)?;
    homophonic::serialized_size(&tokens, MAX_INPUT, op)
}
fn hash(value: &impl Serialize) -> String {
    crate::hashing::sha256(
        &serde_json::to_vec(&crate::operations::contracts::canonical_value(
            serde_json::to_value(value).expect("parameters"),
        ))
        .expect("parameters"),
    )
}
fn complete<T: Serialize>(value: T, op: &'static str) -> Result<T, OperationError> {
    homophonic::serialized_size(&value, MAX_OUTPUT, op)?;
    Ok(value)
}
struct Table {
    entries: Vec<homophonic::Entry>,
    mapping: homophonic::Mapping,
}
impl Table {
    fn new(symbols: Vec<char>, tokens: Vec<String>) -> Result<Self, OperationError> {
        assert_eq!(
            symbols.len(),
            tokens.len(),
            "one derived token per validated symbol"
        );
        let entries: Vec<_> = symbols
            .into_iter()
            .zip(tokens)
            .map(|(symbol, token)| homophonic::Entry {
                symbol,
                tokens: vec![token],
            })
            .collect();
        let mapping = homophonic::Mapping::with_limits(
            homophonic::Definition {
                ascii_uppercase: false,
                entries: entries.clone(),
            },
            1,
            MAX_INPUT,
        )?;
        Ok(Self { entries, mapping })
    }
    fn encode(
        self,
        text: &str,
        p: &impl Serialize,
        op: &'static str,
    ) -> Result<Encoded, OperationError> {
        let tokens = self.mapping.encode(text, &[])?;
        complete(
            Encoded {
                tokens: Tokens(tokens),
                table: self.entries,
                definition_sha256: hash(p),
            },
            op,
        )
    }
    fn decode(
        self,
        tokens: &[String],
        p: &impl Serialize,
        op: &'static str,
    ) -> Result<Decoded, OperationError> {
        let text = self.mapping.decode(tokens)?;
        complete(
            Decoded {
                text,
                table: self.entries,
                definition_sha256: hash(p),
            },
            op,
        )
    }
}
