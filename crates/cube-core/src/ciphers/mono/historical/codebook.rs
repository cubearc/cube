use super::*;
use std::collections::{BTreeMap, BTreeSet};
/// One code may preserve multiple explicitly listed readings; encoding selects
/// the first declared code for an exact supplied unit. No concatenation/splitting.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[schemars(
        length(min = 1, max = 16),
        with = "Vec<super::super::homophonic::TokenText>"
    )]
    pub alternatives: Vec<String>,
    #[schemars(
        length(min = 1, max = 256),
        with = "Vec<super::super::homophonic::TokenText>"
    )]
    pub codes: Vec<String>,
}
/// Exact presegmented units and globally unique code ownership, <=4096 entries
/// and total codes, <=1 MiB compact JSON, <=256 UTF-8 bytes per unit/code.
/// Nulls are omitted; cancel codes erase the last surviving group, including a
/// null/unknown group, never a Unicode scalar or guessed word.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = MAX_BYTES, "x-cube-max-total-codes" = MAX_ENTRIES))]
pub struct Definition {
    #[schemars(length(min = 1, max = 4096))]
    pub entries: Vec<Entry>,
    #[serde(default)]
    #[schemars(length(max = 4096), with = "Vec<super::super::homophonic::TokenText>")]
    pub null_codes: Vec<String>,
    #[serde(default)]
    #[schemars(length(max = 4096), with = "Vec<super::super::homophonic::TokenText>")]
    pub cancel_codes: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    #[schemars(range(max = 65535))]
    pub token_index: u32,
    #[schemars(
        length(min = 1, max = 16),
        with = "Vec<super::super::homophonic::TokenText>"
    )]
    pub alternatives: Vec<String>,
}
/// Readings retain original zero-based token indices. No word spacing, accent
/// restoration or historical interpretation is inferred. Lists are in scan order.
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = MAX_BYTES))]
pub struct DecodeReport {
    #[schemars(length(max = 65536))]
    pub units: Vec<Unit>,
    #[schemars(length(max = 65536), inner(range(max = 65535)))]
    pub null_indices: Vec<u32>,
    #[schemars(length(max = 65536), inner(range(max = 65535)))]
    pub canceled_indices: Vec<u32>,
    #[schemars(length(max = 65536), inner(range(max = 65535)))]
    pub cancel_indices: Vec<u32>,
}
#[derive(Debug)]
pub struct Book {
    entries: Vec<Entry>,
    forward: BTreeMap<String, String>,
    reverse: BTreeMap<String, usize>,
    nulls: BTreeSet<String>,
    cancels: BTreeSet<String>,
}
impl Book {
    pub fn new(definition: Definition) -> Result<Self, OperationError> {
        size(&definition)?;
        bound(CONFIG, "entries", definition.entries.len(), MAX_ENTRIES)?;
        if definition.entries.is_empty() {
            return Err(invalid("entries"));
        }
        let mut owned = BTreeSet::new();
        let mut forward = BTreeMap::new();
        let mut reverse = BTreeMap::new();
        for (index, entry) in definition.entries.iter().enumerate() {
            validate_entry(entry)?;
            for code in &entry.codes {
                own(&mut owned, code)?;
                reverse.insert(code.clone(), index);
            }
            for plain in &entry.alternatives {
                forward
                    .entry(plain.clone())
                    .or_insert_with(|| entry.codes[0].clone());
            }
        }
        for code in definition.null_codes.iter().chain(&definition.cancel_codes) {
            own(&mut owned, code)?;
        }
        Ok(Self {
            entries: definition.entries,
            forward,
            reverse,
            nulls: definition.null_codes.into_iter().collect(),
            cancels: definition.cancel_codes.into_iter().collect(),
        })
    }
    /// Encode exact supplied units deterministically, selecting the first listed
    /// matching code. Never insert nulls, cancellation or implicit word breaks.
    pub fn encode(&self, units: &[String]) -> Result<Vec<String>, OperationError> {
        items(units)?;
        let mut result = Vec::new();
        let mut budget = crate::text::mapping::ByteBudget::new(MAX_BYTES);
        // Reserve serialized-array framing and charge escaped strings before growth.
        use std::io::Write;
        budget.write_all(b"[]").map_err(|_| invalid("units"))?;
        for (index, unit) in units.iter().enumerate() {
            let code = self
                .forward
                .get(unit)
                .ok_or_else(|| input_error("units", index))?;
            if index > 0 {
                budget.write_all(b",").map_err(|_| limit())?;
            }
            serde_json::to_writer(&mut budget, code).map_err(|_| limit())?;
            result.push(code.clone());
        }
        Ok(result)
    }
    /// Decode after cancellation, preserving alternatives and group positions.
    /// Unknown surviving codes fail, including blanks in a historical table.
    pub fn decode(&self, tokens: &[String]) -> Result<DecodeReport, OperationError> {
        items(tokens)?;
        let mut active = Vec::new();
        let mut report = DecodeReport::default();
        for (index, code) in tokens.iter().enumerate() {
            if self.cancels.contains(code) {
                let previous = active.pop().ok_or_else(|| input_error("tokens", index))?;
                report.canceled_indices.push(previous as u32);
                report.cancel_indices.push(index as u32);
            } else {
                active.push(index);
            }
        }
        #[derive(Serialize)]
        struct BorrowedUnit<'a> {
            token_index: u32,
            alternatives: &'a [String],
        }
        #[derive(Serialize)]
        struct BorrowedReport<'a> {
            units: Vec<BorrowedUnit<'a>>,
            null_indices: &'a [u32],
            canceled_indices: &'a [u32],
            cancel_indices: &'a [u32],
        }
        let mut units = Vec::new();
        for index in active {
            let code = &tokens[index];
            if self.nulls.contains(code) {
                report.null_indices.push(index as u32);
                continue;
            }
            let entry = self
                .reverse
                .get(code)
                .map(|i| &self.entries[*i])
                .ok_or_else(|| input_error("tokens", index))?;
            units.push(BorrowedUnit {
                token_index: index as u32,
                alternatives: &entry.alternatives,
            });
        }
        // Check exact escaped JSON bytes through borrowed table strings before
        // cloning result data. The index/reference vectors are capped by MAX_ITEMS.
        size(&BorrowedReport {
            units: units
                .iter()
                .map(|u| BorrowedUnit {
                    token_index: u.token_index,
                    alternatives: u.alternatives,
                })
                .collect(),
            null_indices: &report.null_indices,
            canceled_indices: &report.canceled_indices,
            cancel_indices: &report.cancel_indices,
        })?;
        report.units = units
            .into_iter()
            .map(|u| Unit {
                token_index: u.token_index,
                alternatives: u.alternatives.to_vec(),
            })
            .collect();
        Ok(report)
    }
}
fn limit() -> OperationError {
    error(
        CONFIG,
        ErrorCode::ResourceLimit,
        "output",
        "serialized output exceeds 1 MiB",
    )
}
fn own(owned: &mut BTreeSet<String>, code: &str) -> Result<(), OperationError> {
    token(code)?;
    if !owned.insert(code.to_owned()) {
        return Err(invalid("codes"));
    }
    bound(CONFIG, "codes", owned.len(), MAX_ENTRIES)
}
fn validate_entry(entry: &Entry) -> Result<(), OperationError> {
    if entry.alternatives.is_empty() || entry.codes.is_empty() {
        return Err(invalid("entry"));
    }
    bound(CONFIG, "alternatives", entry.alternatives.len(), 16)?;
    bound(CONFIG, "codes", entry.codes.len(), 256)?;
    let mut seen = BTreeSet::new();
    for plain in &entry.alternatives {
        token(plain)?;
        if !seen.insert(plain) {
            return Err(invalid("alternatives"));
        }
    }
    Ok(())
}
