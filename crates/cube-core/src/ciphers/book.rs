//! Caller-supplied book coordinates; no source storage or retrieval.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
    text::{self, selection},
};
use std::collections::BTreeMap;

pub const MAX_COORDINATES: usize = 65_536;
pub const MAX_UNITS: usize = 65_536;
const OP: &str = "cipher.book.decode";

#[derive(Clone, Copy, Debug)]
pub enum Scheme {
    WordCharacter,
    LineCharacter,
    LineWordCharacter,
}
impl Scheme {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "word_character" => Ok(Self::WordCharacter),
            "line_character" => Ok(Self::LineCharacter),
            "line_word_character" => Ok(Self::LineWordCharacter),
            _ => Err(error(
                ErrorCode::InvalidParameter,
                "unknown book coordinate scheme",
            )
            .parameter("scheme")),
        }
    }
}
fn error(code: ErrorCode, message: impl Into<String>) -> OperationError {
    OperationError::new(code, message).operation(OP)
}
fn bounded_units<'a>(
    items: impl Iterator<Item = &'a str>,
    budget: &mut usize,
) -> Result<Vec<&'a str>, OperationError> {
    let mut result = Vec::new();
    for item in items {
        *budget = budget.checked_sub(1).ok_or_else(|| {
            error(
                ErrorCode::ResourceLimit,
                "source exceeds 65536 indexed units",
            )
            .parameter("source")
        })?;
        result.push(item);
    }
    Ok(result)
}
/// Extract Unicode scalars from comma-separated unsigned decimal coordinates.
/// Pairs are word:character or line:character; triples are line:word:character.
/// Index base is 0 or 1. ASCII whitespace around fields and leading zeroes are
/// accepted, but signs, missing fields and empty records fail. Words use shared
/// Unicode whitespace splitting (punctuation retained); LF/CRLF lines retain
/// spaces, bare CR and blank lines, without a phantom final line. No normalization.
/// Source and coordinates each allow 1 MiB. Indexing allows 65536 units (lines plus
/// words for triples); at most 65536 coordinates yield at most 256 KiB UTF-8.
/// Repeated coordinates reuse scalar indexes instead of rescanning the source.
pub fn decode(
    source: &str,
    coordinates: &str,
    scheme: Scheme,
    index_base: u8,
) -> Result<String, OperationError> {
    for (name, input) in [("source", source), ("coordinates", coordinates)] {
        if input.len() > text::MAX_INPUT {
            return Err(error(ErrorCode::ResourceLimit, "text exceeds 1 MiB").parameter(name));
        }
    }
    if index_base > 1 {
        return Err(
            error(ErrorCode::InvalidParameter, "index_base must be 0 or 1").parameter("index_base"),
        );
    }
    let mut budget = MAX_UNITS;
    let units = match scheme {
        Scheme::WordCharacter => vec![bounded_units(selection::words(source), &mut budget)?],
        Scheme::LineCharacter => vec![bounded_units(selection::lines(source), &mut budget)?],
        Scheme::LineWordCharacter => {
            let lines = bounded_units(selection::lines(source), &mut budget)?;
            lines
                .into_iter()
                .map(|line| bounded_units(selection::words(line), &mut budget))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    let trim = |s: &str| s.trim_matches(|c: char| c.is_ascii_whitespace()).is_empty();
    if trim(coordinates) {
        return Ok(String::new());
    }
    let mut cache: BTreeMap<(usize, usize), Vec<char>> = BTreeMap::new();
    let mut output = String::new();
    for (record, fields) in coordinates.split(',').enumerate() {
        if record == MAX_COORDINATES {
            return Err(
                error(ErrorCode::ResourceLimit, "coordinates exceed 65536 records")
                    .parameter("coordinates"),
            );
        }
        let invalid = || {
            error(
                ErrorCode::InvalidInput,
                format!(
                    "coordinate {}: invalid, missing or out-of-range index",
                    record + 1
                ),
            )
            .parameter("coordinates")
        };
        let expected = if matches!(scheme, Scheme::LineWordCharacter) {
            3
        } else {
            2
        };
        let mut values = [0usize; 3];
        let mut count = 0;
        for field in fields.split(':') {
            if count == expected {
                return Err(invalid());
            }
            let field = field.trim_matches(|c: char| c.is_ascii_whitespace());
            if field.is_empty() || !field.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            values[count] = field
                .parse::<usize>()
                .ok()
                .and_then(|n| selection::unsigned_position(n, index_base))
                .ok_or_else(invalid)?;
            count += 1;
        }
        if count != expected {
            return Err(invalid());
        }
        let (line, unit, character) = if expected == 3 {
            (values[0], values[1], values[2])
        } else {
            (0, values[0], values[1])
        };
        let selected = units
            .get(line)
            .and_then(|row| row.get(unit))
            .ok_or_else(invalid)?;
        let scalars = cache
            .entry((line, unit))
            .or_insert_with(|| selected.chars().collect());
        output.push(*scalars.get(character).ok_or_else(invalid)?);
    }
    Ok(output)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "ciphers",
    description: "Decode caller-supplied book coordinates to exact Unicode scalars; source/coordinates <=1 MiB, <=65536 coordinates/indexed units; no source retrieval.",
    input_type: "text",
    output_type: "text",
    parameters: &[
        Parameter {
            name: "source",
            description: "Exact caller-supplied UTF-8 source, <=1 MiB; no normalization. Preserve edition/line endings and record a source hash for reproducibility.",
            required: true,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "scheme",
            description: "Comma-separated pairs/triples of decimal indexes; Unicode scalars, shared word/line tokenization.",
            required: false,
            default: Some(ParameterValue::Text("word_character")),
            constraints: ParameterType::Text {
                choices: &["word_character", "line_character", "line_word_character"],
            },
        },
        Parameter {
            name: "index_base",
            description: "Indexing base for every coordinate component.",
            required: false,
            default: Some(ParameterValue::Integer(1)),
            constraints: ParameterType::Integer { min: 0, max: 1 },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "cipher book",
    example: "cube cipher book --source source.txt --text '1:1,2:1'",
}];
