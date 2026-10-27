//! Explicit disjoint scalar classes with original byte/scalar provenance.
use super::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget};
use crate::errors::{ErrorCode, OperationError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// One exact set with a unique nonempty name and distinct nonempty output symbol.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub name: String,
    pub characters: String,
    pub symbol: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub ascii_fold: bool,
    pub classes: Vec<Class>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Position {
    pub scalar_index: usize,
    pub byte_offset: usize,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Record {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub class: String,
    pub emitted: String,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Preserved {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar: char,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Report {
    pub mapped_text: String,
    pub records: Vec<Record>,
    pub preserved: Vec<Preserved>,
    pub filtered: Vec<Position>,
    pub filtered_count: usize,
}
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).operation("text.class.map")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "class mapping exceeds 65536 positions or 8 MiB output/report",
    )
    .operation("text.class.map")
}
impl Definition {
    /// ASCII AEIOU→0, BCDFGHJKLMNPQRSTVWXZ→1. If Y is present in the
    /// input, callers must choose vowel, consonant or unmapped explicitly.
    pub fn preset(y: &str) -> Result<Self, OperationError> {
        let mut vowels = "AEIOU".to_owned();
        let mut consonants = "BCDFGHJKLMNPQRSTVWXZ".to_owned();
        match y {
            "unmapped" => {}
            "vowel" => vowels.push('Y'),
            "consonant" => consonants.push('Y'),
            _ => return Err(invalid("y must be vowel, consonant or unmapped").parameter("y")),
        }
        Ok(Self {
            ascii_fold: true,
            classes: vec![
                Class {
                    name: "vowel".into(),
                    characters: vowels,
                    symbol: "0".into(),
                },
                Class {
                    name: "consonant".into(),
                    characters: consonants,
                    symbol: "1".into(),
                },
            ],
        })
    }
    /// Strict bounded JSON Definition. Custom classes use an explicit folding flag.
    pub fn parse(text: &str) -> Result<Self, OperationError> {
        if text.len() > 65536 {
            return Err(invalid("class definitions exceed 64 KiB"));
        }
        serde_json::from_str(text).map_err(|_| invalid("invalid class definition JSON"))
    }
    fn validate(&self) -> Result<BTreeMap<char, usize>, OperationError> {
        if self.classes.is_empty() || self.classes.len() > 4096 {
            return Err(invalid("expected 1..4096 nonempty classes"));
        }
        let mut budget = ByteBudget::new(65536);
        serde_json::to_writer(&mut budget, self)
            .map_err(|_| invalid("class definitions exceed 64 KiB"))?;
        let mut names = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        let mut map = BTreeMap::new();
        for (i, class) in self.classes.iter().enumerate() {
            if class.name.is_empty()
                || class.symbol.is_empty()
                || class.characters.is_empty()
                || !names.insert(&class.name)
                || !outputs.insert(&class.symbol)
            {
                return Err(invalid(
                    "classes need unique names, distinct nonempty outputs and nonempty sets",
                ));
            }
            for c in class.characters.chars() {
                if map.len() == 4096 {
                    return Err(invalid("class definitions exceed 4096 scalars"));
                }
                let c = if self.ascii_fold {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                if map.insert(c, i).is_some() {
                    return Err(invalid("class sets overlap or duplicate after folding"));
                }
            }
        }
        let mut previous: Option<&String> = None;
        for output in outputs {
            if previous.is_some_and(|p| output.starts_with(p.as_str())) {
                return Err(invalid(
                    "output symbols must be prefix-free to avoid ambiguous concatenation",
                ));
            }
            previous = Some(output);
        }
        Ok(map)
    }
    /// Return mapped symbols separately from preserved scalars and filtered
    /// positions. `unmapped`: error/preserve/filter. Every original scalar counts;
    /// indices and byte offsets are zero based. No normalization/class guessing.
    /// <=1 MiB input, <=65536 positions, <=8 MiB serialized report/output.
    pub fn map(&self, text: &str, unmapped: &str) -> Result<Report, OperationError> {
        const OP: &str = "text.class.map";
        check_input(text, OP)?;
        if !["error", "preserve", "filter"].contains(&unmapped) {
            return Err(invalid("unmapped must be error, preserve or filter").parameter("unmapped"));
        }
        let map = self.validate()?;
        if text.chars().count() > MAX_INDICES {
            return Err(limit());
        }
        let mut report = Report {
            mapped_text: String::new(),
            records: vec![],
            preserved: vec![],
            filtered: vec![],
            filtered_count: 0,
        };
        // Charge each retained record and mapped symbol before retaining the next.
        let mut budget = ByteBudget::new(MAX_OUTPUT);
        for (scalar_index, (byte_offset, c)) in text.char_indices().enumerate() {
            let key = if self.ascii_fold {
                c.to_ascii_uppercase()
            } else {
                c
            };
            if let Some(i) = map.get(&key) {
                let class = &self.classes[*i];
                let record = Record {
                    scalar_index,
                    byte_offset,
                    class: class.name.clone(),
                    emitted: class.symbol.clone(),
                };
                serde_json::to_writer(&mut budget, &record).map_err(|_| limit())?;
                serde_json::to_writer(&mut budget, &class.symbol).map_err(|_| limit())?;
                report.mapped_text.push_str(&class.symbol);
                report.records.push(record);
            } else {
                match unmapped {
                    "preserve" => {
                        let record = Preserved {
                            scalar_index,
                            byte_offset,
                            scalar: c,
                        };
                        serde_json::to_writer(&mut budget, &record).map_err(|_| limit())?;
                        serde_json::to_writer(&mut budget, &c).map_err(|_| limit())?;
                        report.preserved.push(record);
                        report.mapped_text.push(c);
                    }
                    "filter" => report.filtered.push(Position {
                        scalar_index,
                        byte_offset,
                    }),
                    _ => {
                        return Err(OperationError::new(
                            ErrorCode::InvalidInput,
                            "unmapped scalar; choose explicit preserve/filter policy if needed",
                        )
                        .operation(OP)
                        .offset(byte_offset as u64));
                    }
                }
            }
        }
        report.filtered_count = report.filtered.len();
        let mut budget = ByteBudget::new(MAX_OUTPUT);
        serde_json::to_writer(&mut budget, &report).map_err(|_| limit())?;
        Ok(report)
    }
}
/// Default preset convenience function; Y requires an explicit choice only when
/// present. Custom definitions instead call Definition::map and define Y exactly.
pub fn map(text: &str, y: Option<&str>, unmapped: &str) -> Result<Report, OperationError> {
    check_input(text, "text.class.map")?;
    if y.is_none() && text.bytes().any(|b| b == b'y' || b == b'Y') {
        return Err(
            invalid("Y requires explicit vowel/consonant/unmapped handling").parameter("y"),
        );
    }
    Definition::preset(y.unwrap_or("unmapped"))?.map(text, unmapped)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "y",
        description: "Required for Y in preset input; conflicts with custom definitions.",
        required: false,
        default: None,
        constraints: ParameterType::Text {
            choices: &["vowel", "consonant", "unmapped"],
        },
    },
    Parameter {
        name: "definition",
        description: "Optional strict JSON class definition with explicit ASCII folding.",
        required: false,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "unmapped",
        description: "Unmapped scalar handling.",
        required: false,
        default: Some(ParameterValue::Text("error")),
        constraints: ParameterType::Text {
            choices: &["error", "preserve", "filter"],
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.class.map",
    category: "text",
    description: "Map disjoint scalar sets with source positions and explicit preserved/filtered records; <=65536 positions/8 MiB report.",
    input_type: "text",
    output_type: "text.class.report",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: Some(false),
    command: "text class",
    example: "cube text class --text AEIOU",
}];
