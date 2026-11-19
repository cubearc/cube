//! Exact integer Match Rating Approach reports for ASCII-letter names.
use super::{check_report, phonetic::ascii_name};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const OP: &str = "text.match_rating";
const PROFILE: &str = "apache-commons-codec-1.15/cube-ascii-v1";

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Encode { name: String },
    Compare { left: String, right: String },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Report {
    Encode {
        profile: &'static str,
        key: String,
    },
    Compare {
        profile: &'static str,
        left_key: String,
        right_key: String,
        unmatched_count: Option<usize>,
        rating: Option<usize>,
        minimum: Option<usize>,
        matches: bool,
    },
}

fn key(name: &str) -> String {
    let mut condensed = Vec::<u8>::with_capacity(name.len());
    for (position, byte) in name.bytes().enumerate() {
        let upper = byte.to_ascii_uppercase();
        if position != 0 && (b"AEIOU".contains(&upper) || condensed.last() == Some(&upper)) {
            continue;
        }
        condensed.push(upper);
    }
    if condensed.len() > 6 {
        let tail = condensed.len() - 3;
        condensed.copy_within(tail.., 3);
        condensed.truncate(6);
    }
    String::from_utf8(condensed).expect("ASCII key")
}

fn minimum(total: usize) -> usize {
    match total {
        0..=4 => 5,
        5..=7 => 4,
        8..=11 => 3,
        _ => 2,
    }
}

fn unmatched(left: &str, right: &str) -> usize {
    let (mut left, mut right) = (left.as_bytes().to_vec(), right.as_bytes().to_vec());
    let (left_end, right_end) = (left.len() - 1, right.len() - 1);
    for i in 0..left.len().min(right.len()) {
        if left[i] == right[i] {
            left[i] = 0;
            right[i] = 0;
        }
        if left[left_end - i] == right[right_end - i] {
            left[left_end - i] = 0;
            right[right_end - i] = 0;
        }
    }
    let remaining = |key: Vec<u8>| key.into_iter().filter(|byte| *byte != 0).count();
    remaining(left).max(remaining(right))
}

fn compare(left: &str, right: &str) -> Report {
    let (left_key, right_key) = (key(left), key(right));
    let metrics = if left_key.is_empty()
        || right_key.is_empty()
        || left_key.len().abs_diff(right_key.len()) >= 3
    {
        None
    } else {
        let unmatched_count = unmatched(&left_key, &right_key);
        let rating = 6 - unmatched_count;
        let minimum = minimum(left_key.len() + right_key.len());
        Some((unmatched_count, rating, minimum))
    };
    let matches = metrics.is_some_and(|(_, rating, minimum)| rating >= minimum);
    Report::Compare {
        profile: PROFILE,
        left_key,
        right_key,
        unmatched_count: metrics.map(|(count, _, _)| count),
        rating: metrics.map(|(_, rating, _)| rating),
        minimum: metrics.map(|(_, _, minimum)| minimum),
        matches,
    }
}

fn execute(request: &Request) -> Result<Report, OperationError> {
    let report = match request {
        Request::Encode { name } => {
            ascii_name(name)?;
            Report::Encode {
                profile: PROFILE,
                key: key(name),
            }
        }
        Request::Compare { left, right } => {
            ascii_name(left)?;
            ascii_name(right)?;
            compare(left, right)
        }
    };
    check_report(&report, 4096)?;
    Ok(report)
}

/// Derive an ASCII key or compare keys with the MRA length gate and rating table.
pub fn apply(request: &Request) -> Result<Report, OperationError> {
    execute(request).map_err(|error| error.operation(OP))
}
