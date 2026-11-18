//! Deterministic contiguous spans and linear-space subsequence reconstruction.
use super::*;
use serde::Serialize;
use std::ops::Range;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PairedSpan {
    pub length: usize,
    pub left: Span,
    pub right: Span,
}

pub(super) struct Indexed {
    pub units: Vec<u32>,
    offsets: Option<Vec<usize>>,
}
impl Indexed {
    pub(super) fn with_offsets(units: Vec<u32>, offsets: Vec<usize>) -> Self {
        Self {
            units,
            offsets: Some(offsets),
        }
    }
    pub fn bytes(units: Vec<u32>) -> Self {
        Self {
            units,
            offsets: None,
        }
    }

    pub fn scalars(units: Vec<u32>, text: &str) -> Self {
        let mut offsets = Vec::with_capacity(units.len() + 1);
        offsets.extend(text.char_indices().map(|(i, _)| i));
        offsets.push(text.len());
        Self {
            units,
            offsets: Some(offsets),
        }
    }

    pub fn span(&self, start: usize, len: usize) -> Span {
        let end = start + len;
        match &self.offsets {
            Some(offsets) => Span {
                start,
                end,
                byte_start: offsets[start],
                byte_end: offsets[end],
            },
            None => Span {
                start,
                end,
                byte_start: start,
                byte_end: end,
            },
        }
    }
}

impl Pair {
    pub(super) fn indexed(&self, limit: usize) -> Result<(Indexed, Indexed), OperationError> {
        let (a, b) = self.symbols(limit)?;
        Ok(match self {
            Self::Bytes { .. } => (Indexed::bytes(a), Indexed::bytes(b)),
            Self::Scalars { left, right } => {
                (Indexed::scalars(a, left), Indexed::scalars(b, right))
            }
        })
    }
}

#[derive(Clone, Copy, Default)]
struct Block {
    left: usize,
    right: usize,
    length: usize,
}
impl Block {
    fn spans(self, a: &Indexed, b: &Indexed) -> PairedSpan {
        PairedSpan {
            length: self.length,
            left: a.span(self.left, self.length),
            right: b.span(self.right, self.length),
        }
    }
}

fn longest(left: &[u32], right: &[u32], work: &mut Work) -> Result<Block, OperationError> {
    work.charge(left.len() * right.len())?;
    let mut best = Block::default();
    if left.is_empty() || right.is_empty() {
        return Ok(best);
    }
    let mut row = vec![0; right.len() + 1];
    for (i, a) in left.iter().enumerate() {
        for (j, b) in right.iter().enumerate().rev() {
            let len = if a == b { row[j] + 1 } else { 0 };
            row[j + 1] = len;
            if len > 0 {
                let (l, r) = (i + 1 - len, j + 1 - len);
                if len > best.length || len == best.length && (l, r) < (best.left, best.right) {
                    best = Block {
                        left: l,
                        right: r,
                        length: len,
                    };
                }
            }
        }
    }
    Ok(best)
}

#[derive(Serialize, JsonSchema)]
pub struct CommonSpans {
    pub prefix: PairedSpan,
    pub suffix: PairedSpan,
    pub substring: PairedSpan,
}

/// Find exact common ends and the first witness of a longest common substring.
pub fn common_spans(input: &Pair) -> Result<CommonSpans, OperationError> {
    let (a, b) = input.indexed(MAX_UNITS)?;
    let (left, right) = (&a.units, &b.units);
    cells(left.len(), right.len(), DP_CELLS)?;
    let prefix = left.iter().zip(right).take_while(|(x, y)| x == y).count();
    let suffix = left
        .iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let tail = if suffix == 0 {
        Block::default()
    } else {
        Block {
            left: left.len() - suffix,
            right: right.len() - suffix,
            length: suffix,
        }
    };
    Ok(CommonSpans {
        prefix: Block {
            length: prefix,
            ..Block::default()
        }
        .spans(&a, &b),
        suffix: tail.spans(&a, &b),
        substring: longest(left, right, &mut Work(1 << 23))?.spans(&a, &b),
    })
}

fn lcs_row(left: &[u32], right: &[u32], reverse: bool) -> Vec<u32> {
    let mut row = vec![0; right.len() + 1];
    for i in 0..left.len() {
        let a = left[if reverse { left.len() - 1 - i } else { i }];
        let mut before = 0;
        for j in 0..right.len() {
            let b = right[if reverse { right.len() - 1 - j } else { j }];
            let previous = row[j + 1];
            row[j + 1] = if a == b {
                before + 1
            } else {
                row[j].max(previous)
            };
            before = previous;
        }
    }
    row
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case")]
pub enum Subsequence {
    Bytes { base64: String },
    Scalars { text: String },
}
#[derive(Serialize, JsonSchema)]
pub struct LcsReport {
    pub length: usize,
    pub subsequence: Subsequence,
    pub positions: Vec<PairedSpan>,
}

type Region = (Range<usize>, Range<usize>);

fn positions(a: &Indexed, b: &Indexed) -> Result<Vec<PairedSpan>, OperationError> {
    let mut result = Vec::new();
    let mut stack = vec![(0..a.units.len(), 0..b.units.len())];
    let mut work = Work(WORK);
    while let Some((left, right)) = stack.pop() {
        if left.is_empty() || right.is_empty() {
            continue;
        }
        let (l, r) = (&a.units[left.clone()], &b.units[right.clone()]);
        work.charge(l.len() * r.len())?;
        if l.len() == 1 {
            if let Some(j) = r.iter().position(|&x| x == l[0]) {
                result.push(
                    Block {
                        left: left.start,
                        right: right.start + j,
                        length: 1,
                    }
                    .spans(a, b),
                );
            }
            continue;
        }
        let middle = l.len() / 2;
        let front = lcs_row(&l[..middle], r, false);
        let back = lcs_row(&l[middle..], r, true);
        work.charge(r.len() + 1)?;
        let split = (0..=r.len())
            .max_by_key(|&j| (front[j] + back[r.len() - j], std::cmp::Reverse(j)))
            .unwrap();
        let middle = left.start + middle;
        let split = right.start + split;
        stack.push((middle..left.end, split..right.end));
        stack.push((left.start..middle, right.start..split));
    }
    // The cell budget bounds the witness to at most 2,048 positions.
    Ok(result)
}

/// Recover a deterministic longest common subsequence using Hirschberg splits.
pub fn lcs(input: &Pair) -> Result<LcsReport, OperationError> {
    let (a, b) = input.indexed(MAX_UNITS)?;
    cells(a.units.len(), b.units.len(), DP_CELLS)?;
    let positions = positions(&a, &b)?;
    let chosen = positions.iter().map(|p| a.units[p.left.start]);
    let subsequence = match input {
        Pair::Bytes { .. } => Subsequence::Bytes {
            base64: crate::encoding::base64_encode(&chosen.map(|n| n as u8).collect::<Vec<_>>()),
        },
        Pair::Scalars { .. } => Subsequence::Scalars {
            text: chosen
                .map(|n| char::from_u32(n).expect("original scalar"))
                .collect(),
        },
    };
    Ok(LcsReport {
        length: positions.len(),
        subsequence,
        positions,
    })
}

#[derive(Serialize, JsonSchema)]
pub struct RatcliffReport {
    pub similarity: f64,
    pub distance: f64,
    pub blocks: Vec<PairedSpan>,
}

fn ratcliff_blocks(a: &Indexed, b: &Indexed) -> Result<Vec<PairedSpan>, OperationError> {
    let mut work = Work(1 << 23);
    let mut stack: Vec<Region> = vec![(0..a.units.len(), 0..b.units.len())];
    let mut blocks = Vec::new();
    while let Some((l, r)) = stack.pop() {
        let mut block = longest(&a.units[l.clone()], &b.units[r.clone()], &mut work)?;
        if block.length == 0 {
            continue;
        }
        block.left += l.start;
        block.right += r.start;
        blocks.push(block.spans(a, b));
        stack.push((l.start..block.left, r.start..block.right));
        stack.push((
            block.left + block.length..l.end,
            block.right + block.length..r.end,
        ));
    }
    blocks.sort_unstable_by_key(|block| block.left.start);
    Ok(blocks)
}

/// Compare by recursively matching deterministic contiguous blocks.
pub fn ratcliff_obershelp(input: &Pair) -> Result<RatcliffReport, OperationError> {
    let (a, b) = input.indexed(16384)?;
    let blocks = ratcliff_blocks(&a, &b)?;
    let length = a.units.len() + b.units.len();
    let matched = blocks.iter().map(|b| b.length).sum::<usize>();
    let similarity = if length == 0 {
        1.0
    } else {
        2.0 * matched as f64 / length as f64
    };
    Ok(RatcliffReport {
        similarity,
        distance: 1.0 - similarity,
        blocks,
    })
}
