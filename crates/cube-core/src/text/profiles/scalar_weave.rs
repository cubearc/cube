//! Exact scalar padding and an invertible lane visitation order.

use super::{check_report, invalid, parameter, resource};
use crate::errors::OperationError;
use crate::text::{MAX_INDICES, MAX_INPUT};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const LANES: usize = 16;
const REPORT_BYTES: usize = 2 << 20;

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Left,
    #[default]
    Right,
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    EqualLengths,
    Ragged,
}

const fn space() -> char {
    ' '
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Pad {
        text: String,
        #[schemars(range(max = 65536))]
        width: usize,
        #[serde(default)]
        side: Side,
        #[serde(default = "space")]
        pad: char,
    },
    Interleave {
        #[schemars(length(min = 2, max = 16))]
        lanes: Vec<String>,
        #[serde(default)]
        mode: Mode,
    },
    Deinterleave {
        text: String,
        #[schemars(length(min = 2, max = 16))]
        lengths: Vec<usize>,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Report {
    Pad { text: String, pad_count: usize },
    Interleave { text: String, lengths: Vec<usize> },
    Deinterleave { lanes: Vec<String> },
}

struct LaneOrder {
    remaining: [usize; LANES],
    next_lanes: VecDeque<usize>,
}

impl LaneOrder {
    fn new(lengths: &[usize]) -> Result<Self, OperationError> {
        if !(2..=LANES).contains(&lengths.len()) {
            return Err(parameter("provide 2 to 16 lanes"));
        }
        if lengths
            .iter()
            .try_fold(0usize, |total, &length| total.checked_add(length))
            .is_none_or(|total| total > MAX_INDICES)
        {
            return Err(resource("lane lengths exceed 65536 scalars"));
        }
        let mut remaining = [0; LANES];
        remaining[..lengths.len()].copy_from_slice(lengths);
        let next_lanes = lengths
            .iter()
            .enumerate()
            .filter_map(|(index, &length)| (length != 0).then_some(index))
            .collect();
        Ok(Self {
            remaining,
            next_lanes,
        })
    }
}

impl Iterator for LaneOrder {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let lane = self.next_lanes.pop_front()?;
        self.remaining[lane] -= 1;
        if self.remaining[lane] != 0 {
            self.next_lanes.push_back(lane);
        }
        Some(lane)
    }
}

fn pad(text: &str, width: usize, side: Side, pad: char) -> Result<Report, OperationError> {
    if width > MAX_INDICES {
        return Err(parameter("padding width exceeds 65536 scalars"));
    }
    let count = scalar_count(text)?;
    let pad_count = width
        .checked_sub(count)
        .ok_or_else(|| invalid("source exceeds target width"))?;
    let mut output = String::with_capacity(text.len() + pad_count * pad.len_utf8());
    if matches!(side, Side::Right) {
        output.push_str(text);
    }
    output.extend(std::iter::repeat_n(pad, pad_count));
    if matches!(side, Side::Left) {
        output.push_str(text);
    }
    Ok(Report::Pad {
        text: output,
        pad_count,
    })
}

fn scalar_count(text: &str) -> Result<usize, OperationError> {
    let count = text.chars().take(MAX_INDICES + 1).count();
    if text.len() > MAX_INPUT || count > MAX_INDICES {
        return Err(resource("source exceeds 1 MiB or 65536 scalars"));
    }
    Ok(count)
}

fn interleave(lanes: &[String], mode: Mode) -> Result<Report, OperationError> {
    if !(2..=LANES).contains(&lanes.len()) {
        return Err(parameter("provide 2 to 16 lanes"));
    }
    let bytes = lanes
        .iter()
        .try_fold(0usize, |size, lane| size.checked_add(lane.len()));
    let bytes = bytes
        .filter(|&n| n <= MAX_INPUT)
        .ok_or_else(|| resource("lanes exceed 1 MiB"))?;
    let lengths = lanes
        .iter()
        .map(|lane| lane.chars().count())
        .collect::<Vec<_>>();
    let order = LaneOrder::new(&lengths)?;
    if matches!(mode, Mode::EqualLengths) && lengths.windows(2).any(|pair| pair[0] != pair[1]) {
        return Err(invalid(
            "equal-length mode requires all lanes to have the same scalar length",
        ));
    }
    let mut input = lanes.iter().map(|lane| lane.chars()).collect::<Vec<_>>();
    let mut text = String::with_capacity(bytes);
    text.extend(order.map(|lane| input[lane].next().expect("validated lane length")));
    Ok(Report::Interleave { text, lengths })
}

fn deinterleave(text: &str, lengths: &[usize]) -> Result<Report, OperationError> {
    let order = LaneOrder::new(lengths)?;
    if lengths.iter().sum::<usize>() != scalar_count(text)? {
        return Err(invalid(
            "lane lengths must sum to the exact source scalar count",
        ));
    }
    let mut lanes = lengths
        .iter()
        .map(|&len| String::with_capacity(len))
        .collect::<Vec<_>>();
    for (lane, scalar) in order.zip(text.chars()) {
        lanes[lane].push(scalar);
    }
    Ok(Report::Deinterleave { lanes })
}

/// Pad, interleave or recover exact scalar lanes using explicit dimensions.
pub fn apply(request: &Request) -> Result<Report, OperationError> {
    let report = match request {
        Request::Pad {
            text,
            width,
            side,
            pad: scalar,
        } => pad(text, *width, *side, *scalar),
        Request::Interleave { lanes, mode } => interleave(lanes, *mode),
        Request::Deinterleave { text, lengths } => deinterleave(text, lengths),
    }
    .map_err(|error| error.operation("text.scalar_weave"))?;
    check_report(&report, REPORT_BYTES).map_err(|error| error.operation("text.scalar_weave"))?;
    Ok(report)
}
