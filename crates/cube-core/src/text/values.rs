//! Exact word/scalar numeric evidence, without implicit alphabets or normalization.
use super::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget, word::Index};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Serialize)]
pub struct Length {
    pub word_index: usize,
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub byte_length: usize,
    pub scalar_length: usize,
}
#[derive(Debug, Serialize)]
pub struct Codepoint {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar: char,
    pub value: u32,
    pub presentation: String,
}
#[derive(Debug, Serialize)]
pub struct CharacterValue {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar: char,
    pub value: Option<u64>,
}
#[derive(Debug, Serialize)]
pub struct WordValues {
    pub word_index: usize,
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar_length: usize,
    pub values: Vec<CharacterValue>,
    pub sum: Option<u64>,
    pub omitted_count: usize,
}
fn error(op: &'static str, code: ErrorCode, message: &str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
fn check_report<T: Serialize>(value: &T, op: &'static str) -> Result<(), OperationError> {
    let mut budget = ByteBudget::new(MAX_OUTPUT);
    serde_json::to_writer(&mut budget, value).map_err(|_| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "numeric evidence report exceeds 8 MiB",
        )
    })
}
/// Unicode-scalar word lengths with original boundaries; Unicode whitespace
/// splits words, punctuation remains data. <=1 MiB, <=65536 words, <=8 MiB report.
pub fn word_lengths(input: &str) -> Result<Vec<Length>, OperationError> {
    const OP: &str = "text.word.lengths";
    let source = Index::new(input, OP)?;
    let result: Vec<_> = source
        .words
        .iter()
        .enumerate()
        .map(|(word_index, w)| Length {
            word_index,
            scalar_index: w.scalar_index,
            byte_offset: w.byte_offset,
            byte_length: w.byte_length,
            scalar_length: w.scalar_length,
        })
        .collect();
    check_report(&result, OP)?;
    Ok(result)
}
/// Exact u32 scalar values, including whitespace. Presentation is decimal, hex
/// (uppercase, no prefix) or unicode (U+XXXX); numeric value never changes.
/// <=1 MiB input, <=65536 original scalar records, <=8 MiB serialized report.
pub fn codepoints(input: &str, format: &str) -> Result<Vec<Codepoint>, OperationError> {
    const OP: &str = "text.codepoints";
    check_input(input, OP)?;
    if !["decimal", "hex", "unicode"].contains(&format) {
        return Err(error(
            OP,
            ErrorCode::InvalidParameter,
            "format must be decimal, hex or unicode",
        )
        .parameter("format"));
    }
    if input.chars().count() > MAX_INDICES {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "code points exceed 65536 records",
        ));
    }
    let result: Vec<_> = input
        .char_indices()
        .enumerate()
        .map(|(scalar_index, (byte_offset, scalar))| {
            let value = scalar as u32;
            let presentation = match format {
                "hex" => format!("{value:X}"),
                "unicode" => format!("U+{value:04X}"),
                _ => value.to_string(),
            };
            Codepoint {
                scalar_index,
                byte_offset,
                scalar,
                value,
                presentation,
            }
        })
        .collect();
    check_report(&result, OP)?;
    Ok(result)
}
/// Bounded CLI JSON array of [scalar, exact unsigned integer] pairs.
pub fn parse_table(json: &str) -> Result<Vec<(char, u64)>, OperationError> {
    const OP: &str = "text.word.values";
    if json.len() > 65536 {
        return Err(
            error(OP, ErrorCode::ResourceLimit, "value table exceeds 64 KiB").parameter("table"),
        );
    }
    let result: Vec<(char, u64)> = serde_json::from_str(json).map_err(|_| {
        error(
            OP,
            ErrorCode::InvalidParameter,
            "expected exact scalar/u64 JSON pairs",
        )
        .parameter("table")
    })?;
    if result.len() > 4096 {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "value table exceeds 4096 entries",
        )
        .parameter("table"));
    }
    Ok(result)
}
/// Per-word character values default to exact Unicode scalar code points.
/// An optional <=4096-entry/64 KiB table matches exact scalars; unmapped error
/// (default at interfaces) or skip retains a null value and omission count.
/// Optional checked u64 sums retain every character record and obey max_sum.
/// <=1 MiB source, <=65536 total values including omissions, <=8 MiB report.
pub fn word_values(
    input: &str,
    table: Option<&[(char, u64)]>,
    unmapped: &str,
    sums: bool,
    max_sum: u64,
) -> Result<Vec<WordValues>, OperationError> {
    const OP: &str = "text.word.values";
    if !["error", "skip"].contains(&unmapped) {
        return Err(error(
            OP,
            ErrorCode::InvalidParameter,
            "unmapped must be error or skip",
        )
        .parameter("unmapped"));
    }
    if !sums && max_sum != u64::MAX {
        return Err(
            error(OP, ErrorCode::InvalidParameter, "max_sum requires sums").parameter("max_sum"),
        );
    }
    let map = if let Some(table) = table {
        if table.len() > 4096 {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "value table exceeds 4096 entries",
            )
            .parameter("table"));
        }
        let mut budget = ByteBudget::new(65536);
        serde_json::to_writer(&mut budget, table).map_err(|_| {
            error(OP, ErrorCode::ResourceLimit, "value table exceeds 64 KiB").parameter("table")
        })?;
        let mut map = BTreeMap::new();
        for &(scalar, value) in table {
            if map.insert(scalar, value).is_some() {
                return Err(error(
                    OP,
                    ErrorCode::InvalidParameter,
                    "duplicate exact scalar in value table",
                )
                .parameter("table"));
            }
        }
        Some(map)
    } else {
        None
    };
    let source = Index::new(input, OP)?;
    if source.words.iter().map(|w| w.scalar_length).sum::<usize>() > MAX_INDICES {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "word values exceed 65536 total scalar records",
        ));
    }
    let mut result = Vec::with_capacity(source.words.len());
    let mut budget = ByteBudget::new(MAX_OUTPUT);
    for (word_index, word) in source.words.iter().enumerate() {
        let mut values = Vec::with_capacity(word.scalar_length);
        let mut sum = 0u64;
        let mut omitted_count = 0;
        for scalar_index in word.scalar_index..word.scalar_index + word.scalar_length {
            let (byte_offset, scalar) = source.scalar(scalar_index);
            let value = map
                .as_ref()
                .map_or(Some(u64::from(scalar as u32)), |m| m.get(&scalar).copied());
            if let Some(value) = value {
                if sums {
                    sum = sum
                        .checked_add(value)
                        .filter(|v| *v <= max_sum)
                        .ok_or_else(|| {
                            error(
                                OP,
                                ErrorCode::ResourceLimit,
                                "word sum overflows u64 or exceeds max_sum",
                            )
                            .parameter("max_sum")
                            .offset(byte_offset as u64)
                        })?;
                }
            } else if unmapped == "skip" {
                omitted_count += 1;
            } else {
                return Err(error(
                    OP,
                    ErrorCode::InvalidInput,
                    "unmapped scalar in explicit value table",
                )
                .parameter("table")
                .offset(byte_offset as u64));
            }
            values.push(CharacterValue {
                scalar_index,
                byte_offset,
                scalar,
                value,
            });
        }
        let record = WordValues {
            word_index,
            scalar_index: word.scalar_index,
            byte_offset: word.byte_offset,
            scalar_length: word.scalar_length,
            values,
            sum: sums.then_some(sum),
            omitted_count,
        };
        serde_json::to_writer(&mut budget, &record).map_err(|_| {
            error(
                OP,
                ErrorCode::ResourceLimit,
                "word value report exceeds 8 MiB",
            )
        })?;
        result.push(record);
    }
    check_report(&result, OP)?;
    Ok(result)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "text.word.lengths",
        category: "text",
        description: "Exact Unicode-scalar lengths of whitespace-delimited words with original boundaries; punctuation retained, <=65536 records.",
        input_type: "text",
        output_type: "text.word.lengths",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "text word-lengths",
        example: "cube text word-lengths --text 'A é'",
    },
    Descriptor {
        id: "text.codepoints",
        category: "text",
        description: "Exact u32 scalar code points with original offsets and decimal/hex/U+ presentation; <=65536 records, no normalization.",
        input_type: "text",
        output_type: "text.codepoints",
        parameters: &[Parameter {
            name: "format",
            description: "Presentation only; numeric fields remain exact integers.",
            required: false,
            default: Some(ParameterValue::Text("decimal")),
            constraints: ParameterType::Text {
                choices: &["decimal", "hex", "unicode"],
            },
        }],
        deterministic: true,
        reversible: Some(false),
        command: "text codepoints",
        example: "cube text codepoints --format unicode --text 'A é'",
    },
    Descriptor {
        id: "text.word.values",
        category: "text",
        description: "Ordered per-word scalar/u64 values with original positions, explicit optional tables/omissions and checked sums; <=65536 total values.",
        input_type: "text",
        output_type: "text.word.values",
        deterministic: true,
        reversible: Some(false),
        command: "text word-values",
        example: "cube text word-values --sums --text 'A é'",
        parameters: &[
            Parameter {
                name: "table",
                description: "Optional JSON [scalar,u64] pairs; <=4096 unique scalars/64 KiB, exact matching.",
                required: false,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
            Parameter {
                name: "unmapped",
                description: "Explicit custom-table policy; skip retains null values and counts.",
                required: false,
                default: Some(ParameterValue::Text("error")),
                constraints: ParameterType::Text {
                    choices: &["error", "skip"],
                },
            },
            Parameter {
                name: "sums",
                description: "Retain per-character values and add an exact checked u64 word sum.",
                required: false,
                default: Some(ParameterValue::Boolean(false)),
                constraints: ParameterType::Boolean,
            },
            Parameter {
                name: "max_sum",
                description: "Inclusive sum bound; non-default requires sums.",
                required: false,
                default: Some(ParameterValue::Unsigned(u64::MAX)),
                constraints: ParameterType::Unsigned {
                    min: 0,
                    max: u64::MAX,
                },
            },
        ],
    },
];
