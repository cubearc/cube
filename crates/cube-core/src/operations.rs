//! Static operation metadata; direct Rust functions remain the execution API.
//!
//! IDs and type names are compatibility contracts. Add fields compatibly and do
//! not reuse an ID for different semantics. Categories are strings, so extending
//! the catalog does not require changing a central category enum.

use serde::Serialize;

/// A parameter value used only for metadata and validation, not runtime dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParameterValue<'a> {
    /// Signed integer.
    Integer(i64),
    /// Boolean flag.
    Boolean(bool),
    /// UTF-8 text.
    Text(&'a str),
    /// Exact bytes; JSON represents these as an integer array.
    Bytes(&'a [u8]),
}

/// Parameter constraints, shared by discovery and explicit preflight validation.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterType {
    /// Inclusive integer range.
    Integer { min: i64, max: i64 },
    /// Boolean flag.
    Boolean,
    /// UTF-8 text; an empty list permits any text.
    Text { choices: &'static [&'static str] },
    /// Exact bytes with a minimum length.
    Bytes { min_length: usize },
}

impl ParameterType {
    fn accepts(self, value: ParameterValue<'_>) -> bool {
        match (self, value) {
            (Self::Integer { min, max }, ParameterValue::Integer(n)) => (min..=max).contains(&n),
            (Self::Boolean, ParameterValue::Boolean(_)) => true,
            (Self::Text { choices }, ParameterValue::Text(text)) => {
                choices.is_empty() || choices.contains(&text)
            }
            (Self::Bytes { min_length }, ParameterValue::Bytes(bytes)) => bytes.len() >= min_length,
            _ => false,
        }
    }
}

/// One named operation parameter. Defaults are explicit and never applied silently.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Parameter {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
    pub default: Option<ParameterValue<'static>>,
    pub constraints: ParameterType,
}

/// Describes an implemented operation without imposing an execution hierarchy.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Descriptor {
    pub id: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Stable semantic type name: `bytes`, `text`, or a named structured result.
    pub input_type: &'static str,
    pub output_type: &'static str,
    pub parameters: &'static [Parameter],
    pub deterministic: bool,
    /// `None` means reversibility is not meaningful for this operation.
    pub reversible: Option<bool>,
    /// Existing human CLI spelling; multiple IDs may share a parameterized command.
    pub command: &'static str,
    pub example: &'static str,
}

/// A precise parameter validation failure, without including its input value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterError {
    pub operation: &'static str,
    pub parameter: String,
    pub reason: &'static str,
}

impl std::fmt::Display for ParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: parameter '{}': {}",
            self.operation, self.parameter, self.reason
        )
    }
}

impl std::error::Error for ParameterError {}

impl Descriptor {
    /// Validate supplied values without executing an operation or allocating defaults.
    /// Unknown and duplicate names, missing required values and invalid types/ranges
    /// are errors. Optional defaults remain visible in the descriptor.
    pub fn validate(&self, values: &[(&str, ParameterValue<'_>)]) -> Result<(), ParameterError> {
        let error = |name: &str, reason| ParameterError {
            operation: self.id,
            parameter: name.to_owned(),
            reason,
        };
        for (index, (name, value)) in values.iter().enumerate() {
            let parameter = self
                .parameters
                .iter()
                .find(|p| p.name == *name)
                .ok_or_else(|| error(name, "unknown parameter"))?;
            if values[..index].iter().any(|(previous, _)| previous == name) {
                return Err(error(name, "duplicate parameter"));
            }
            if !parameter.constraints.accepts(*value) {
                return Err(error(
                    name,
                    "invalid type or value; see operation constraints",
                ));
            }
        }
        for parameter in self.parameters {
            if parameter.required
                && parameter.default.is_none()
                && !values.iter().any(|(name, _)| *name == parameter.name)
            {
                return Err(error(parameter.name, "required parameter is missing"));
            }
        }
        Ok(())
    }
}

/// Collect metadata from its owning modules in legacy command order.
/// This is metadata assembly, not a dispatcher. Consumers may sort by stable ID.
pub fn catalog() -> Vec<&'static Descriptor> {
    crate::encoding::OPERATIONS
        .iter()
        .chain(crate::ciphers::OPERATIONS)
        .chain(crate::hashing::OPERATIONS)
        .chain(crate::binary::OPERATIONS)
        .chain(crate::forensics::OPERATIONS)
        .chain(crate::forensics::signatures::OPERATIONS)
        .chain(crate::forensics::inspect::OPERATIONS)
        .chain(crate::forensics::ranges::OPERATIONS)
        .collect()
}

/// Look up an exact stable ID; no aliases or heuristic matching are performed.
pub fn describe(id: &str) -> Option<&'static Descriptor> {
    catalog().into_iter().find(|descriptor| descriptor.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_is_complete_and_self_consistent() {
        let catalog = catalog();
        assert!(catalog.len() >= 14);
        let mut ids = BTreeSet::new();
        for d in catalog {
            assert!(ids.insert(d.id));
            assert!(d.id.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            }));
            assert!(!d.description.is_empty() && !d.category.is_empty() && !d.example.is_empty());
            assert!(!d.input_type.is_empty() && !d.output_type.is_empty());
            let mut names = BTreeSet::new();
            for parameter in d.parameters {
                assert!(names.insert(parameter.name));
                if let Some(default) = parameter.default {
                    assert!(parameter.constraints.accepts(default));
                }
            }
            assert_eq!(describe(d.id).unwrap().id, d.id);
        }
        assert!(describe("missing.operation").is_none());
    }

    #[test]
    fn validate_parameters_before_execution() {
        let caesar = describe("cipher.caesar.encode").unwrap();
        assert!(caesar.validate(&[]).is_err());
        for value in [i32::MIN as i64, 0, i32::MAX as i64] {
            assert!(
                caesar
                    .validate(&[("shift", ParameterValue::Integer(value))])
                    .is_ok()
            );
        }
        for value in [i32::MIN as i64 - 1, i32::MAX as i64 + 1] {
            assert!(
                caesar
                    .validate(&[("shift", ParameterValue::Integer(value))])
                    .is_err()
            );
        }
        assert!(
            caesar
                .validate(&[("shift", ParameterValue::Text("3"))])
                .is_err()
        );
        assert!(
            caesar
                .validate(&[("unknown", ParameterValue::Boolean(true))])
                .is_err()
        );
        assert!(
            caesar
                .validate(&[
                    ("shift", ParameterValue::Integer(1)),
                    ("shift", ParameterValue::Integer(2))
                ])
                .is_err()
        );
        let xor = describe("binary.xor").unwrap();
        assert!(
            xor.validate(&[("key", ParameterValue::Bytes(b""))])
                .is_err()
        );
        assert!(
            xor.validate(&[("key", ParameterValue::Bytes(b"k"))])
                .is_ok()
        );
        let e = xor.validate(&[]).unwrap_err();
        assert!(e.to_string().contains("key"));
        assert!(ParameterType::Boolean.accepts(ParameterValue::Boolean(false)));
        assert!(ParameterType::Text { choices: &["raw"] }.accepts(ParameterValue::Text("raw")));
        assert!(!ParameterType::Text { choices: &["raw"] }.accepts(ParameterValue::Text("other")));
        assert!(ParameterType::Text { choices: &[] }.accepts(ParameterValue::Text("世界")));
    }
}
