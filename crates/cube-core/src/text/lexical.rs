//! Preparation shared by the two caller-dictionary consumers.
use super::{multiset::Profile, statistics};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Entry {
    pub text: String,
    pub aliases: Vec<usize>,
}
pub(crate) fn error(
    op: &'static str,
    code: ErrorCode,
    name: &'static str,
    msg: &str,
) -> OperationError {
    OperationError::new(code, msg).operation(op).parameter(name)
}
pub(crate) fn charge(
    work: &mut usize,
    n: usize,
    max: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    *work = work.checked_add(n).filter(|v| *v <= max).ok_or_else(|| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "max_work",
            "preparation/index work exceeds budget",
        )
    })?;
    Ok(())
}
pub(crate) fn prepare(
    s: &str,
    p: Profile,
    work: &mut usize,
    max: usize,
    op: &'static str,
) -> Result<Vec<char>, OperationError> {
    charge(work, s.len(), max, op)?;
    let (v, n) = statistics::prepare(s, p.normalization, p.case, p.filter, max - *work, op)?;
    charge(work, n + v.len(), max, op)?;
    Ok(v)
}
pub(crate) fn entries(
    words: &[String],
    p: Profile,
    max_scalars: usize,
    work: &mut usize,
    max: usize,
    op: &'static str,
) -> Result<Vec<Entry>, OperationError> {
    let mut lookup = BTreeMap::new();
    let mut result: Vec<Entry> = Vec::new();
    for (i, s) in words.iter().enumerate() {
        let v = prepare(s, p, work, max, op)?;
        if v.is_empty() {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "dictionary",
                "dictionary entry is empty after preparation",
            ));
        }
        if v.len() > max_scalars {
            return Err(error(
                op,
                ErrorCode::ResourceLimit,
                "dictionary",
                "prepared word exceeds scalar limit",
            ));
        }
        charge(
            work,
            v.len() * (1 + words.len().max(1).ilog2() as usize),
            max,
            op,
        )?;
        let text: String = v.into_iter().collect();
        if let Some(&j) = lookup.get(&text) {
            let j: usize = j;
            result[j].aliases.push(i);
        } else {
            lookup.insert(text.clone(), result.len());
            result.push(Entry {
                text,
                aliases: vec![i],
            });
        }
    }
    Ok(result)
}
/// Decode a bounded JSON string array without guessing a dictionary format.
pub fn parse_dictionary(
    document: &str,
    max_bytes: usize,
    max_entries: usize,
) -> Result<Vec<String>, OperationError> {
    const OP: &str = "text.dictionary.parse";
    let v = crate::pipeline::json::parse_with_limits(
        document,
        max_bytes,
        2,
        max_entries.saturating_add(1),
    )
    .map_err(|e| e.error.operation(OP))?;
    serde_json::from_value(v).map_err(|e| {
        error(
            OP,
            ErrorCode::InvalidInput,
            "dictionary",
            "expected JSON array of UTF-8 strings",
        )
        .caused_by(e)
    })
}
