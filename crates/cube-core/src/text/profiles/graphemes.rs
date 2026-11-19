//! Default extended grapheme clusters from UAX #29 revision 47.
use super::{SourceSpan, check_report, resource};
use crate::errors::OperationError;
use crate::text::{MAX_INDICES, MAX_OUTPUT, check_input};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use unicode_segmentation::{UNICODE_VERSION, UnicodeSegmentation};

const OP: &str = "text.graphemes";
const UAX_REVISION: u8 = 47;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Segments,
    Reverse,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub text: String,
    pub action: Action,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Report {
    Segments {
        unicode_version: (u64, u64, u64),
        uax29_revision: u8,
        segments: Vec<SourceSpan>,
    },
    Reverse {
        unicode_version: (u64, u64, u64),
        uax29_revision: u8,
        text: String,
    },
}

fn segments(text: &str) -> Result<Vec<SourceSpan>, OperationError> {
    let mut output = Vec::new();
    let mut scalar_start = 0;
    for (byte_start, cluster) in text.grapheme_indices(true) {
        if output.len() == MAX_INDICES {
            return Err(resource("grapheme listing exceeds 65536 records"));
        }
        let scalar_end = scalar_start + cluster.chars().count();
        output.push(SourceSpan {
            byte_start,
            byte_end: byte_start + cluster.len(),
            scalar_start,
            scalar_end,
        });
        scalar_start = scalar_end;
    }
    Ok(output)
}

fn reverse(text: &str) -> String {
    // A forward pass uses one cursor for RI parity and Indic/emoji context.
    // Offsets use at most four bytes per byte of the bounded UTF-8 source.
    let mut starts: Vec<u32> = text
        .grapheme_indices(true)
        .map(|(offset, _)| offset as u32)
        .collect();
    starts.push(text.len() as u32);
    let mut result = String::with_capacity(text.len());
    for bounds in starts.windows(2).rev() {
        result.push_str(&text[bounds[0] as usize..bounds[1] as usize]);
    }
    result
}

/// List half-open original byte and scalar spans or reverse the original slices.
pub fn apply(request: &Request) -> Result<Report, OperationError> {
    check_input(&request.text, OP)?;
    let report = match request.action {
        Action::Segments => Report::Segments {
            unicode_version: UNICODE_VERSION,
            uax29_revision: UAX_REVISION,
            segments: segments(&request.text).map_err(|error| error.operation(OP))?,
        },
        Action::Reverse => Report::Reverse {
            unicode_version: UNICODE_VERSION,
            uax29_revision: UAX_REVISION,
            text: reverse(&request.text),
        },
    };
    check_report(&report, MAX_OUTPUT).map_err(|error| error.operation(OP))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_unicode_grapheme_break_corpus_matches_original_boundaries() {
        let data = include_str!("../../../data/unicode-17/GraphemeBreakTest.txt");
        let mut count = 0;
        for (line, row) in data.lines().enumerate() {
            let row = row.split('#').next().unwrap().trim();
            if row.is_empty() {
                continue;
            }
            let mut text = String::new();
            let mut expected = Vec::new();
            for unit in row.split_whitespace() {
                match unit {
                    "÷" => expected.push(text.len()),
                    "×" => {}
                    hex => {
                        text.push(char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap())
                    }
                }
            }
            let report = segments(&text).unwrap();
            let observed: Vec<_> = std::iter::once(0)
                .chain(report.iter().map(|span| span.byte_end))
                .collect();
            assert_eq!(observed, expected, "test vector at line {}", line + 1);
            count += 1;
        }
        assert_eq!(count, 766);
    }
}
