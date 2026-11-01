//! Reversible exact-scalar permutations, including explicit keyed alphabets.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
const CONFIG: &str = "cipher.substitution.encode";
/// Immutable validated bijection. No case folding, normalization or aliases.
#[derive(Debug)]
pub struct Mapping {
    forward: BTreeMap<char, char>,
    reverse: BTreeMap<char, char>,
}
pub(super) fn symbols(value: &str, op: &'static str) -> Result<Vec<char>, OperationError> {
    bound(op, "alphabet", value.len(), MAX_DEFINITION)?;
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for c in value.chars() {
        if !seen.insert(c) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "alphabet",
                "alphabet scalars must be unique",
            ));
        }
        bound(op, "alphabet", result.len() + 1, MAX_SYMBOLS)?;
        result.push(c);
    }
    if result.is_empty() {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "alphabet",
            "alphabet must contain at least one scalar",
        ));
    }
    Ok(result)
}
/// Deduplicate a keyword in first-occurrence order, then append unused base scalars.
/// Every key scalar must belong to the explicit base alphabet; empty keys are valid.
pub fn keyword_alphabet(base: &str, keyword: &str) -> Result<String, OperationError> {
    bound(
        CONFIG,
        "keyword",
        base.len().saturating_add(keyword.len()),
        MAX_DEFINITION,
    )?;
    let alphabet = symbols(base, CONFIG)?;
    let allowed: BTreeSet<_> = alphabet.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut output = String::new();
    for (offset, c) in keyword.char_indices() {
        if !allowed.contains(&c) {
            return Err(error(
                CONFIG,
                ErrorCode::InvalidParameter,
                "keyword",
                "keyword scalar is absent from the base alphabet",
            )
            .offset(offset as u64));
        }
        if seen.insert(c) {
            output.push(c);
        }
    }
    for c in alphabet {
        if seen.insert(c) {
            output.push(c);
        }
    }
    Ok(output)
}
impl Mapping {
    /// Validate 1..4096 unique scalars per alphabet, equal sizes, <=64 KiB combined.
    pub fn new(source: &str, target: &str) -> Result<Self, OperationError> {
        bound(
            CONFIG,
            "alphabet",
            source.len().saturating_add(target.len()),
            MAX_DEFINITION,
        )?;
        let source = symbols(source, CONFIG)?;
        let target = symbols(target, CONFIG)?;
        if source.len() != target.len() {
            return Err(error(
                CONFIG,
                ErrorCode::InvalidParameter,
                "target",
                "source and target alphabets must have equal scalar counts",
            ));
        }
        Ok(Self {
            forward: source.iter().copied().zip(target.iter().copied()).collect(),
            reverse: target.into_iter().zip(source).collect(),
        })
    }
    /// Construct a permutation from an explicit base and keyword; no hidden alphabet.
    pub fn keyword(base: &str, keyword: &str) -> Result<Self, OperationError> {
        Self::new(base, &keyword_alphabet(base, keyword)?)
    }
    /// Encode <=1 MiB text into <=4 MiB text. Unsupported policy defaults belong to
    /// adapters; direct callers explicitly choose reject or preserve.
    pub fn encode(&self, text: &str, unmapped: &str) -> Result<String, OperationError> {
        self.transform(text, unmapped, false)
    }
    /// Invert the exact mapping. Preserve is an inverse only when preserved symbols
    /// do not collide with the opposite alphabet; reject guarantees the bijection.
    pub fn decode(&self, text: &str, unmapped: &str) -> Result<String, OperationError> {
        self.transform(text, unmapped, true)
    }
    fn transform(
        &self,
        text: &str,
        unmapped: &str,
        decode: bool,
    ) -> Result<String, OperationError> {
        let op = if decode {
            "cipher.substitution.decode"
        } else {
            CONFIG
        };
        bound(op, "text", text.len(), MAX_INPUT)?;
        let preserve = policy(op, unmapped)?;
        let map = if decode { &self.reverse } else { &self.forward };
        let mut output = String::new();
        for (offset, c) in text.char_indices() {
            let mapped = match map.get(&c) {
                Some(v) => *v,
                None if preserve => c,
                None => {
                    return Err(error(
                        op,
                        ErrorCode::InvalidInput,
                        "text",
                        "scalar is absent from the selected alphabet",
                    )
                    .offset(offset as u64));
                }
            };
            bound(op, "output", output.len() + mapped.len_utf8(), MAX_OUTPUT)?;
            output.push(mapped);
        }
        Ok(output)
    }
}

/// Select exactly one explicit target or keyword; interfaces share this rule.
pub fn configured(
    source: &str,
    target: Option<&str>,
    keyword: Option<&str>,
) -> Result<Mapping, OperationError> {
    match (target, keyword) {
        (Some(target), None) => Mapping::new(source, target),
        (None, Some(key)) => Mapping::keyword(source, key),
        _ => Err(error(
            CONFIG,
            ErrorCode::InvalidParameter,
            "target",
            "choose exactly one target alphabet or keyword",
        )),
    }
}
