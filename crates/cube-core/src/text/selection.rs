//! Word/line scalar selection. Existing positional extraction remains independent.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType},
    text,
};
use serde::Serialize;
/// Unicode data used by Rust's White_Space tokenization; no language segmentation.
pub const WORD_UNICODE_VERSION: (u8, u8, u8) = std::char::UNICODE_VERSION;
pub(crate) fn unsigned_position(value: usize, index_base: u8) -> Option<usize> {
    value.checked_sub(usize::from(index_base))
}
pub(crate) fn words(input: &str) -> std::str::SplitWhitespace<'_> {
    input.split_whitespace()
}
pub(crate) fn lines(input: &str) -> std::str::Lines<'_> {
    input.lines()
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Selector {
    WordFirst,
    WordLast,
    LineFirst,
    LineLast,
}
impl Selector {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "word_first" => Ok(Self::WordFirst),
            "word_last" => Ok(Self::WordLast),
            "line_first" => Ok(Self::LineFirst),
            "line_last" => Ok(Self::LineLast),
            _ => Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "selector must be word_first, word_last, line_first or line_last",
            )
            .operation("text.select")
            .parameter("selector")),
        }
    }
}
/// Select the first/last Unicode scalar in each nonempty word/line. Words split
/// on Unicode whitespace and retain punctuation. Lines follow LF/CRLF semantics:
/// no line-ending scalars, no extra final empty line, and bare CR remains data.
/// Empty lines contribute no scalar; leading/trailing line spaces remain data.
/// `line_first` is an acrostic without automatic trimming or normalization.
pub fn select(input: &str, selector: Selector) -> Result<String, OperationError> {
    const OP: &str = "text.select";
    text::check_input(input, OP)?;
    match selector {
        Selector::WordFirst => {
            text::collect_bounded(words(input).filter_map(|w| w.chars().next()), OP)
        }
        Selector::WordLast => {
            text::collect_bounded(words(input).filter_map(|w| w.chars().next_back()), OP)
        }
        Selector::LineFirst => {
            text::collect_bounded(lines(input).filter_map(|w| w.chars().next()), OP)
        }
        Selector::LineLast => {
            text::collect_bounded(lines(input).filter_map(|w| w.chars().next_back()), OP)
        }
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.select",
    category: "text",
    description: "First/last Unicode scalar per word or nonempty LF/CRLF line; punctuation/line spaces retained; no normalization; <=1 MiB.",
    input_type: "text",
    output_type: "text",
    parameters: &[Parameter {
        name: "selector",
        description: "Words split on Unicode whitespace; line_first is an untrimmed acrostic. Unicode scalars, not grapheme clusters.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["word_first", "word_last", "line_first", "line_last"],
        },
    }],
    deterministic: true,
    reversible: Some(false),
    command: "text select",
    example: "cube text select --selector word_first --text 'Cube Uses Binary Evidence'",
}];
