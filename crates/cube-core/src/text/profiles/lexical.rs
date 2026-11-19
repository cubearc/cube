//! Exact Unicode word selection and stable, metered lexical ordering.
use super::{SourceSpan, check_report, invalid, parameter, resource};
use crate::errors::OperationError;
use crate::text::{MAX_INDICES, MAX_INPUT, MAX_OUTPUT, check_input, word};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

const WORK: usize = 8 << 20;

const fn one() -> u8 {
    1
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutOfRange {
    #[default]
    Error,
    Skip,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectRequest {
    pub text: String,
    #[schemars(length(max = 65536))]
    pub indices: Vec<i64>,
    #[serde(default = "one")]
    #[schemars(range(max = 1))]
    pub index_base: u8,
    #[serde(default)]
    pub out_of_range: OutOfRange,
}

#[derive(Serialize, JsonSchema)]
pub struct Selected<'a> {
    pub index: i64,
    pub word_index: usize,
    pub text: &'a str,
    #[serde(flatten)]
    pub span: SourceSpan,
}

#[derive(Serialize, JsonSchema)]
pub struct Omitted {
    pub position: usize,
    pub index: i64,
}

#[derive(Serialize, JsonSchema)]
pub struct SelectReport<'a> {
    pub selected: Vec<Selected<'a>>,
    pub omitted: Vec<Omitted>,
}

fn select(request: &SelectRequest) -> Result<SelectReport<'_>, OperationError> {
    if request.index_base > 1 {
        return Err(parameter("index_base must be zero or one"));
    }
    if request.indices.len() > MAX_INDICES {
        return Err(resource("selection exceeds 65536 indices"));
    }
    let source = word::Index::new(&request.text, "text.select_words")?;
    let mut selected = Vec::with_capacity(request.indices.len());
    let mut omitted = Vec::new();
    for (position, &index) in request.indices.iter().enumerate() {
        let Some(word_index) = word::resolve(index, request.index_base, source.words.len()) else {
            if matches!(request.out_of_range, OutOfRange::Error) {
                return Err(invalid("a requested word index is out of range").parameter("indices"));
            }
            omitted.push(Omitted { position, index });
            continue;
        };
        let word = source.words[word_index];
        let byte_end = word.byte_offset + word.byte_length;
        selected.push(Selected {
            index,
            word_index,
            text: &request.text[word.byte_offset..byte_end],
            span: SourceSpan {
                byte_start: word.byte_offset,
                byte_end,
                scalar_start: word.scalar_index,
                scalar_end: word.scalar_index + word.scalar_length,
            },
        });
    }
    let report = SelectReport { selected, omitted };
    check_report(&report, MAX_OUTPUT)?;
    Ok(report)
}

/// Copy selected Unicode-whitespace words with coordinates in the original text.
pub fn select_words(request: &SelectRequest) -> Result<SelectReport<'_>, OperationError> {
    select(request).map_err(|error| error.operation("text.select_words"))
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    #[default]
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Collation {
    #[default]
    Scalar,
    AsciiLetters,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case", deny_unknown_fields)]
pub enum SortRequest {
    Scalars {
        text: String,
        #[serde(default)]
        order: Order,
        #[serde(default)]
        collation: Collation,
    },
    Tokens {
        #[schemars(length(max = 65536))]
        tokens: Vec<String>,
        #[serde(default)]
        order: Order,
        #[serde(default)]
        collation: Collation,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case")]
pub enum SortReport<'a> {
    Scalars {
        text: String,
        original_indices: Vec<usize>,
    },
    Tokens {
        tokens: Vec<&'a str>,
        original_indices: Vec<usize>,
    },
}

struct Meter(usize);

impl Meter {
    fn charge(&mut self, steps: usize) -> Result<(), OperationError> {
        self.0 = self
            .0
            .checked_sub(steps)
            .ok_or_else(|| resource("lexical comparison exceeds its work limit"))?;
        Ok(())
    }
}

fn merge_order(
    count: usize,
    order: Order,
    mut compare: impl FnMut(usize, usize) -> Result<Ordering, OperationError>,
) -> Result<Vec<usize>, OperationError> {
    let mut current: Vec<_> = (0..count).collect();
    let mut next = Vec::with_capacity(count);
    let mut width = 1;
    while width < count {
        next.clear();
        for chunk in current.chunks(width * 2) {
            let (left, right) = chunk.split_at(chunk.len().min(width));
            let (mut l, mut r) = (0, 0);
            while l < left.len() && r < right.len() {
                let relation = compare(left[l], right[r])?;
                let take_right = match order {
                    Order::Ascending => relation == Ordering::Greater,
                    Order::Descending => relation == Ordering::Less,
                };
                if take_right {
                    next.push(right[r]);
                    r += 1;
                } else {
                    next.push(left[l]);
                    l += 1;
                }
            }
            next.extend_from_slice(&left[l..]);
            next.extend_from_slice(&right[r..]);
        }
        std::mem::swap(&mut current, &mut next);
        width *= 2;
    }
    Ok(current)
}

fn scalar_sort(
    text: &str,
    order: Order,
    collation: Collation,
) -> Result<SortReport<'_>, OperationError> {
    check_input(text, "text.sort")?;
    let mut data = Vec::with_capacity(text.len().min(MAX_INDICES));
    for c in text.chars() {
        if data.len() == MAX_INDICES {
            return Err(resource("source exceeds 65536 scalars"));
        }
        if matches!(collation, Collation::AsciiLetters) && !c.is_ascii_alphabetic() {
            return Err(invalid("ascii_letters requires ASCII letters"));
        }
        data.push(c);
    }
    let key = |value: char| match collation {
        Collation::Scalar => value,
        Collation::AsciiLetters => value.to_ascii_lowercase(),
    };
    let mut budget = Meter(WORK);
    let original_indices = merge_order(data.len(), order, |a, b| {
        budget.charge(1)?;
        Ok(key(data[a]).cmp(&key(data[b])))
    })?;
    let text = original_indices.iter().map(|&i| data[i]).collect();
    Ok(SortReport::Scalars {
        text,
        original_indices,
    })
}

fn compare_tokens(
    a: &str,
    b: &str,
    collation: Collation,
    budget: &mut Meter,
) -> Result<Ordering, OperationError> {
    budget.charge(1)?;
    for (a, b) in a.bytes().zip(b.bytes()) {
        budget.charge(1)?;
        let ordering = match collation {
            Collation::Scalar => a.cmp(&b),
            Collation::AsciiLetters => a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()),
        };
        if !ordering.is_eq() {
            return Ok(ordering);
        }
    }
    Ok(a.len().cmp(&b.len()))
}

fn token_sort(
    tokens: &[String],
    order: Order,
    collation: Collation,
) -> Result<SortReport<'_>, OperationError> {
    if tokens.len() > MAX_INDICES
        || tokens
            .iter()
            .try_fold(0usize, |total, token| total.checked_add(token.len()))
            .is_none_or(|n| n > MAX_INPUT)
    {
        return Err(resource("token array exceeds 1 MiB or 65536 tokens"));
    }
    if matches!(collation, Collation::AsciiLetters)
        && tokens
            .iter()
            .any(|token| !token.bytes().all(|b| b.is_ascii_alphabetic()))
    {
        return Err(invalid("ascii_letters requires ASCII-letter tokens"));
    }
    let mut budget = Meter(WORK);
    // For valid UTF-8, byte order gives the exact scalar lexical order.
    let original_indices = merge_order(tokens.len(), order, |a, b| {
        compare_tokens(&tokens[a], &tokens[b], collation, &mut budget)
    })?;
    let tokens = original_indices
        .iter()
        .map(|&i| tokens[i].as_str())
        .collect();
    Ok(SortReport::Tokens {
        tokens,
        original_indices,
    })
}

/// Sort exact scalars or explicit tokens and return the stable source permutation.
pub fn sort(request: &SortRequest) -> Result<SortReport<'_>, OperationError> {
    let report = match request {
        SortRequest::Scalars {
            text,
            order,
            collation,
        } => scalar_sort(text, *order, *collation),
        SortRequest::Tokens {
            tokens,
            order,
            collation,
        } => token_sort(tokens, *order, *collation),
    }
    .map_err(|error| error.operation("text.sort"))?;
    check_report(&report, MAX_OUTPUT).map_err(|error| error.operation("text.sort"))?;
    Ok(report)
}
