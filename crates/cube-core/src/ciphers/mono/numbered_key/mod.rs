//! Deterministic ACA Numbered Key construction over Core homophonic mappings.
mod catalog;
mod fixtures;
use super::{
    MAX_DEFINITION, MAX_INPUT, MAX_OUTPUT, MAX_SYMBOLS, MAX_TOKENS, alphabet, bound, error,
    homophonic,
};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const OP: &str = "cipher.numbered_key.encode";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Preparation {
    Exact,
    AsciiLowercaseLetters,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    First,
    RoundRobin,
}
/// Every convention is explicit. Rotation selects the extended-key position
/// numbered with origin; width is the exact zero-padded decimal token width.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=65536))]
    pub alphabet: String,
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=65536))]
    pub key: String,
    pub preparation: Preparation,
    #[schemars(range(max = 4095))]
    pub rotation: u16,
    /// Canonical unsigned decimal u128, never a floating-point JSON number.
    #[schemars(regex(pattern = "^(0|[1-9][0-9]{0,38})$"))]
    pub origin: String,
    #[schemars(range(min = 1, max = 40))]
    pub width: u8,
    pub choice: Choice,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))]
    pub text: String,
    #[schemars(length(max = 65536), with = "Vec<homophonic::TokenText>")]
    pub tokens: Vec<String>,
    #[schemars(length(min = 1, max = 4096))]
    pub table: Vec<Entry>,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
/// Positions remain in numeric order, including every repeated key occurrence.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub symbol: char,
    #[schemars(length(min = 1, max = 4096), with = "Vec<homophonic::TokenText>")]
    pub tokens: Vec<String>,
}
fn invalid(parameter: &'static str) -> OperationError {
    error(
        OP,
        ErrorCode::InvalidParameter,
        parameter,
        "invalid explicit Numbered Key convention",
    )
}
fn prepare(text: &str, policy: Preparation, key: bool) -> Result<String, OperationError> {
    let parameter = if key { "key" } else { "text" };
    bound(
        OP,
        parameter,
        text.len(),
        if key { MAX_DEFINITION } else { MAX_INPUT },
    )?;
    if !key {
        bound(OP, parameter, text.chars().count(), MAX_TOKENS)?;
    }
    match policy {
        Preparation::Exact => Ok(text.into()),
        Preparation::AsciiLowercaseLetters => {
            if !text.is_ascii() {
                return Err(error(
                    OP,
                    if key {
                        ErrorCode::InvalidParameter
                    } else {
                        ErrorCode::InvalidInput
                    },
                    parameter,
                    "ASCII letter preparation rejects non-ASCII input",
                ));
            }
            Ok(text
                .chars()
                .filter(char::is_ascii_alphabetic)
                .map(|c| c.to_ascii_lowercase())
                .collect())
        }
    }
}
struct Constructed {
    table: Vec<Entry>,
    mapping: homophonic::Mapping,
    hash: String,
}
fn positions(p: &Parameters, symbols: &[char]) -> Result<Vec<char>, OperationError> {
    if matches!(p.preparation, Preparation::AsciiLowercaseLetters)
        && !symbols.iter().all(char::is_ascii_lowercase)
    {
        return Err(invalid("alphabet"));
    }
    let key = prepare(&p.key, p.preparation, true)?;
    let mut positions: Vec<_> = key.chars().collect();
    bound(OP, "key_positions", positions.len(), MAX_SYMBOLS)?;
    let allowed: std::collections::BTreeSet<_> = symbols.iter().copied().collect();
    if positions.iter().any(|c| !allowed.contains(c)) {
        return Err(invalid("key"));
    }
    let present: std::collections::BTreeSet<_> = positions.iter().copied().collect();
    positions.extend(symbols.iter().filter(|c| !present.contains(c)).copied());
    bound(OP, "positions", positions.len(), MAX_SYMBOLS)?;
    let rotation = usize::from(p.rotation);
    if rotation >= positions.len() || !(1..=40).contains(&p.width) {
        return Err(invalid("rotation_or_width"));
    }
    positions.rotate_left(rotation);
    Ok(positions)
}
fn table(
    p: &Parameters,
    symbols: &[char],
    positions: &[char],
) -> Result<Vec<Entry>, OperationError> {
    let origin: u128 = p.origin.parse().map_err(|_| invalid("origin"))?;
    if origin.to_string() != p.origin {
        return Err(invalid("origin"));
    }
    let last = origin
        .checked_add((positions.len() - 1) as u128)
        .ok_or_else(|| invalid("origin"))?;
    if last.to_string().len() > usize::from(p.width) {
        return Err(invalid("width"));
    }
    let mut grouped: BTreeMap<char, Vec<String>> =
        symbols.iter().map(|c| (*c, Vec::new())).collect();
    for (i, c) in positions.iter().enumerate() {
        grouped.get_mut(c).expect("validated symbol").push(format!(
            "{:0width$}",
            origin + i as u128,
            width = usize::from(p.width)
        ));
    }
    let table: Vec<_> = symbols
        .iter()
        .map(|c| Entry {
            symbol: *c,
            tokens: grouped.remove(c).expect("alphabet entry"),
        })
        .collect();
    Ok(table)
}
fn mapping(table: &[Entry], first: bool) -> Result<homophonic::Mapping, OperationError> {
    homophonic::Mapping::with_token_limit(
        homophonic::Definition {
            ascii_uppercase: false,
            entries: table
                .iter()
                .map(|entry| homophonic::Entry {
                    symbol: entry.symbol,
                    tokens: if first {
                        vec![entry.tokens[0].clone()]
                    } else {
                        entry.tokens.clone()
                    },
                })
                .collect(),
        },
        MAX_SYMBOLS,
    )
}
fn construct(p: &Parameters, first: bool) -> Result<Constructed, OperationError> {
    homophonic::serialized_size(p, MAX_DEFINITION, OP)?;
    let symbols = alphabet::symbols(&p.alphabet, OP)?;
    let positions = positions(p, &symbols)?;
    let table = table(p, &symbols, &positions)?;
    let full = mapping(&table, false)?;
    let mapping = if first { mapping(&table, true)? } else { full };
    let effective =
        crate::operations::contracts::canonical_value(serde_json::to_value(p).expect("parameters"));
    Ok(Constructed {
        table,
        mapping,
        hash: crate::hashing::sha256(&serde_json::to_vec(&effective).expect("parameters")),
    })
}
fn report(c: Constructed, text: String, tokens: Vec<String>) -> Result<Report, OperationError> {
    let result = Report {
        text,
        tokens,
        table: c.table,
        definition_sha256: c.hash,
    };
    homophonic::serialized_size(&result, MAX_OUTPUT, OP)?;
    Ok(result)
}
/// Prepare text explicitly, then encode using first or per-symbol round robin.
/// Counters reset on every invocation. Never deduplicate the key's positions.
pub fn encode(text: &str, p: &Parameters) -> Result<Report, OperationError> {
    let run = || {
        let c = construct(p, matches!(p.choice, Choice::First))?;
        let text = prepare(text, p.preparation, false)?;
        let tokens = c.mapping.encode(&text, &[])?;
        report(c, text, tokens)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Decode exact padded tokens; spaces/punctuation removed by preparation cannot
/// be reconstructed. No integer coercion, alternative spelling or key guessing.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Report, OperationError> {
    let run = || {
        let c = construct(p, false)?;
        let text = c.mapping.decode(tokens)?;
        report(c, text, tokens.to_vec())
    };
    run().map_err(|e: OperationError| e.operation("cipher.numbered_key.decode"))
}
