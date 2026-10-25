//! Shared bounded source-word indexing and explicit signed per-word extraction.
use super::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget, selection};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct Span {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar_length: usize,
    pub byte_length: usize,
}
pub(crate) struct Index<'a> {
    pub input: &'a str,
    pub words: Vec<Span>,
    pub offsets: Vec<u32>,
}
impl<'a> Index<'a> {
    pub fn new(input: &'a str, op: &'static str) -> Result<Self, OperationError> {
        check_input(input, op)?;
        let count = selection::words(input).count();
        if count > MAX_INDICES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "source exceeds 65536 words",
            )
            .operation(op));
        }
        let scalars = input.chars().count();
        let mut offsets = Vec::with_capacity(scalars);
        offsets.extend(input.char_indices().map(|(i, _)| i as u32));
        let mut words = Vec::with_capacity(count);
        for word in selection::words(input) {
            // SplitWhitespace returns subslices of input; integer subtraction
            // computes their byte position without unsafe pointer operations.
            let byte_offset = word.as_ptr() as usize - input.as_ptr() as usize;
            let scalar_index = offsets
                .binary_search(&(byte_offset as u32))
                .expect("word starts at a scalar boundary");
            words.push(Span {
                scalar_index,
                byte_offset,
                scalar_length: word.chars().count(),
                byte_length: word.len(),
            });
        }
        Ok(Self {
            input,
            words,
            offsets,
        })
    }
    pub fn scalar(&self, index: usize) -> (usize, char) {
        let offset = self.offsets[index] as usize;
        (
            offset,
            self.input[offset..].chars().next().expect("indexed scalar"),
        )
    }
}
/// Explicit ordered shared positions or one list for every source word.
#[derive(Clone, Copy)]
pub enum Positions<'a> {
    Shared(&'a [i64]),
    PerWord(&'a [Vec<i64>]),
}
#[derive(Debug, Serialize)]
pub struct Selection {
    pub position: i64,
    pub scalar_index: Option<usize>,
    pub byte_offset: Option<usize>,
    pub scalar: Option<char>,
}
#[derive(Debug, Serialize)]
pub struct WordResult {
    pub word_index: usize,
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar_length: usize,
    pub byte_length: usize,
    pub selections: Vec<Selection>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub extracted: String,
    pub words: Vec<WordResult>,
    pub omitted_count: usize,
}
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).operation("text.word.extract")
}
/// Resolve positive origin-0/1 positions and negative indexes from a word's end.
/// Negative -1 always means the final scalar; i64::MIN is handled without negation.
fn resolve(position: i64, base: u8, length: usize) -> Option<usize> {
    let index = if position < 0 {
        length.checked_sub(usize::try_from(position.unsigned_abs()).ok()?)?
    } else {
        selection::unsigned_position(usize::try_from(position).ok()?, base)?
    };
    (index < length).then_some(index)
}
/// Extract ordered signed positions per Unicode-whitespace word, retaining
/// punctuation, duplicates and original scalar/byte positions. Origin defaults
/// to 1 at interfaces; short_word is error or skip (explicit null omission records).
/// No placeholder or silent cycling. <=1 MiB input, <=65536 words/coordinates,
/// <=8 MiB serialized report, <8 MiB source index; no normalization.
pub fn extract(
    input: &str,
    positions: Positions<'_>,
    index_base: u8,
    short_word: &str,
) -> Result<Report, OperationError> {
    const OP: &str = "text.word.extract";
    if index_base > 1 || !["error", "skip"].contains(&short_word) {
        return Err(invalid(
            "index_base must be 0/1 and short_word must be error/skip",
        ));
    }
    let source = Index::new(input, OP)?;
    let all: Box<dyn Iterator<Item = &[i64]>> = match positions {
        Positions::Shared(p) => {
            if p.len() > MAX_INDICES
                || source
                    .words
                    .len()
                    .checked_mul(p.len())
                    .is_none_or(|n| n > MAX_INDICES)
            {
                return Err(
                    invalid("total requested coordinates exceed 65536").parameter("positions")
                );
            }
            Box::new(std::iter::repeat_n(p, source.words.len()))
        }
        Positions::PerWord(p) => {
            if p.len() != source.words.len() {
                return Err(invalid("per-word lists must match the source word count")
                    .parameter("per_word"));
            }
            if p.iter()
                .try_fold(0usize, |n, p| n.checked_add(p.len()))
                .is_none_or(|n| n > MAX_INDICES)
            {
                return Err(invalid("total requested coordinates exceed 65536"));
            }
            Box::new(p.iter().map(Vec::as_slice))
        }
    };
    let lists: Vec<_> = all.collect();
    if index_base == 1 && lists.iter().any(|p| p.contains(&0)) {
        return Err(invalid("zero is invalid for origin 1").parameter("positions"));
    }
    // Shared positions must validate even when an empty source has no lists.
    if let Positions::Shared(p) = positions
        && index_base == 1
        && p.contains(&0)
    {
        return Err(invalid("zero is invalid for origin 1").parameter("positions"));
    }
    let mut report = Report {
        extracted: String::new(),
        words: Vec::with_capacity(source.words.len()),
        omitted_count: 0,
    };
    let mut budget = ByteBudget::new(MAX_OUTPUT);
    for (word_index, (word, positions)) in source.words.iter().zip(lists).enumerate() {
        let mut result = WordResult {
            word_index,
            scalar_index: word.scalar_index,
            byte_offset: word.byte_offset,
            scalar_length: word.scalar_length,
            byte_length: word.byte_length,
            selections: Vec::with_capacity(positions.len()),
        };
        for &position in positions {
            if let Some(relative) = resolve(position, index_base, word.scalar_length) {
                let index = word.scalar_index + relative;
                let (offset, scalar) = source.scalar(index);
                report.extracted.push(scalar);
                result.selections.push(Selection {
                    position,
                    scalar_index: Some(index),
                    byte_offset: Some(offset),
                    scalar: Some(scalar),
                });
            } else if short_word == "skip" {
                report.omitted_count += 1;
                result.selections.push(Selection {
                    position,
                    scalar_index: None,
                    byte_offset: None,
                    scalar: None,
                });
            } else {
                return Err(OperationError::new(
                    ErrorCode::InvalidInput,
                    "requested position is outside a source word",
                )
                .operation(OP)
                .parameter("positions")
                .offset(word.byte_offset as u64));
            }
        }
        serde_json::to_writer(&mut budget, &result).map_err(|_| {
            OperationError::new(
                ErrorCode::ResourceLimit,
                "word extraction report exceeds 8 MiB",
            )
            .operation(OP)
        })?;
        report.words.push(result);
    }
    let mut budget = ByteBudget::new(MAX_OUTPUT);
    serde_json::to_writer(&mut budget, &report).map_err(|_| {
        OperationError::new(
            ErrorCode::ResourceLimit,
            "word extraction report exceeds 8 MiB",
        )
        .operation(OP)
    })?;
    Ok(report)
}
/// A bounded concrete JSON list codec for CLI signed positions. Nested lists are
/// permitted only for the explicitly per-word transport; no arbitrary values.
pub fn parse_positions(text: &str) -> Result<Vec<i64>, OperationError> {
    if text.len() > 2 * 1024 * 1024 {
        return Err(invalid("position JSON exceeds 2 MiB"));
    }
    let result: Vec<i64> = serde_json::from_str(text)
        .map_err(|_| invalid("expected a JSON list of exact signed i64 positions"))?;
    if result.len() > MAX_INDICES {
        return Err(invalid("positions exceed 65536"));
    }
    Ok(result)
}
pub fn parse_per_word(text: &str) -> Result<Vec<Vec<i64>>, OperationError> {
    if text.len() > 2 * 1024 * 1024 {
        return Err(invalid("position JSON exceeds 2 MiB"));
    }
    let result: Vec<Vec<i64>> = serde_json::from_str(text)
        .map_err(|_| invalid("expected JSON per-word lists of exact signed i64 positions"))?;
    if result.len() > MAX_INDICES
        || result
            .iter()
            .try_fold(0usize, |n, p| n.checked_add(p.len()))
            .is_none_or(|n| n > MAX_INDICES)
    {
        return Err(invalid("per-word positions exceed 65536"));
    }
    Ok(result)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.word.extract",
    category: "text",
    description: "Ordered signed positions per Unicode-whitespace word, exact original scalar/byte provenance and explicit short-word omissions; <=65536 coordinates.",
    input_type: "text",
    output_type: "text.word.extraction",
    deterministic: true,
    reversible: Some(false),
    command: "text word-extract",
    example: "cube text word-extract --positions '[1,3]' --text 'Cube Uses Binary Evidence'",
    parameters: &[
        Parameter {
            name: "positions",
            description: "Shared ordered signed-i64 list; CLI JSON text. Choose exactly one of positions/per_word.",
            required: false,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "per_word",
            description: "Explicit JSON signed-i64 list for every word; native SDK accepts nested lists, no cycling.",
            required: false,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "index_base",
            description: "Positive position origin 0 or 1; negative -1 always last.",
            required: false,
            default: Some(ParameterValue::Unsigned(1)),
            constraints: ParameterType::UnsignedChoices { choices: &[0, 1] },
        },
        Parameter {
            name: "short_word",
            description: "Fail on an absent position or retain an explicit null omission record.",
            required: false,
            default: Some(ParameterValue::Text("error")),
            constraints: ParameterType::Text {
                choices: &["error", "skip"],
            },
        },
    ],
}];
