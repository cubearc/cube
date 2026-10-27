//! Deterministic inverse coordinates over exactly the existing book tokenization.
mod catalog;
mod fixtures;
#[cfg(test)]
mod tests;
use super::{MAX_COORDINATES, MAX_UNITS, Scheme, index_units};
use crate::{
    ciphers::{
        poly::{bound, invalid, json_bound},
        transposition::wire,
    },
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const OP: &str = "cipher.book.encode";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateScheme {
    WordCharacter,
    LineCharacter,
    LineWordCharacter,
}
impl CoordinateScheme {
    fn legacy(self) -> Scheme {
        match self {
            Self::WordCharacter => Scheme::WordCharacter,
            Self::LineCharacter => Scheme::LineCharacter,
            Self::LineWordCharacter => Scheme::LineWordCharacter,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchPolicy {
    First,
    RoundRobin,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EncodeOptions {
    pub scheme: CoordinateScheme,
    #[schemars(range(min = 0, max = 1))]
    #[serde(deserialize_with = "wire::integer")]
    pub index_base: u8,
    pub matching: MatchPolicy,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EncodeReport {
    #[schemars(length(max=2097152),extend("x-cube-max-utf8-bytes"=2097152))]
    pub coordinates: String,
    #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub source_sha256: String,
    #[schemars(range(min = 0, max = 65536))]
    pub coordinate_count: u32,
    #[schemars(range(min = 0, max = 65536))]
    pub indexed_units: u32,
}
/// Encode exact scalars with first-occurrence or independent per-scalar cycles.
/// Source and plaintext each <=1 MiB, <=65536 coordinates and indexed units.
/// Existing Unicode whitespace/LF/CRLF semantics are shared with `book::decode`.
/// The source hash covers exact UTF-8 bytes, including line endings. There is no
/// normalization or source retrieval. Report <=4 MiB; no guessed edition or key.
pub fn encode(source: &str, text: &str, p: &EncodeOptions) -> Result<EncodeReport, OperationError> {
    for (field, value) in [("source", source), ("text", text)] {
        bound(OP, field, value.len(), 1048576)?;
    }
    if p.index_base > 1 {
        return Err(invalid(OP, "index_base", "index base must be 0 or 1"));
    }
    let n = text.chars().take(MAX_COORDINATES + 1).count();
    bound(OP, "text", n, MAX_COORDINATES)?;
    let units = index_units(source, p.scheme.legacy()).map_err(|e| e.operation(OP))?;
    let indexed_units = if matches!(p.scheme, CoordinateScheme::LineWordCharacter) {
        units.len() + units.iter().map(Vec::len).sum::<usize>()
    } else {
        units[0].len()
    };
    debug_assert!(indexed_units <= MAX_UNITS);
    let base = u32::from(p.index_base);
    let mut positions: BTreeMap<char, Vec<[u32; 3]>> = BTreeMap::new();
    for (line, row) in units.iter().enumerate() {
        for (unit, word) in row.iter().enumerate() {
            for (character, c) in word.chars().enumerate() {
                positions.entry(c).or_default().push([
                    line as u32 + base,
                    unit as u32 + base,
                    character as u32 + base,
                ]);
            }
        }
    }
    let mut cycles: BTreeMap<char, usize> = BTreeMap::new();
    let mut coordinates = String::new();
    for (offset, c) in text.char_indices() {
        let choices = positions.get(&c).ok_or_else(|| {
            OperationError::new(
                ErrorCode::InvalidInput,
                "plaintext scalar is absent from the indexed source",
            )
            .operation(OP)
            .parameter("text")
            .offset(offset as u64)
        })?;
        let next = cycles.entry(c).or_default();
        let coordinate = choices[match p.matching {
            MatchPolicy::First => 0,
            MatchPolicy::RoundRobin => *next % choices.len(),
        }];
        *next += 1;
        if !coordinates.is_empty() {
            coordinates.push(',');
        }
        let [line, unit, character] = coordinate;
        use std::fmt::Write;
        if matches!(p.scheme, CoordinateScheme::LineWordCharacter) {
            write!(&mut coordinates, "{line}:{unit}:{character}").expect("string write");
        } else {
            write!(&mut coordinates, "{unit}:{character}").expect("string write");
        }
    }
    let report = EncodeReport {
        coordinates,
        source_sha256: crate::hashing::sha256(source.as_bytes()),
        coordinate_count: n as u32,
        indexed_units: indexed_units as u32,
    };
    json_bound(&report, 4194304, OP)?;
    Ok(report)
}
