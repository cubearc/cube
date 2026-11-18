//! Global and local exact integer alignment with affine-gap traceback.
use super::{spans::Indexed, *};
use serde::Serialize;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Global,
    Local,
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Gap {
    Linear {
        #[serde(default = "negative_one")]
        penalty: i64,
    },
    Affine {
        open: i64,
        extend: i64,
    },
}
impl Default for Gap {
    fn default() -> Self {
        Self::Linear { penalty: -1 }
    }
}
const fn two() -> i64 {
    2
}
const fn negative_one() -> i64 {
    -1
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignRequest {
    pub operands: Pair,
    pub mode: Mode,
    #[serde(default = "two", rename = "match")]
    pub match_score: i64,
    #[serde(default = "negative_one")]
    pub mismatch: i64,
    #[serde(default)]
    pub gap: Gap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EditKind {
    Equal,
    Substitute,
    Delete,
    Insert,
}

#[derive(Serialize, JsonSchema)]
pub struct Run {
    pub kind: EditKind,
    pub left: Span,
    pub right: Span,
}

#[derive(Serialize, JsonSchema)]
pub struct AlignReport {
    pub score: i64,
    pub left: Span,
    pub right: Span,
    pub runs: Vec<Run>,
}

const MATCH: usize = 0;
const DELETE: usize = 1;
const INSERT: usize = 2;
const STOP: u8 = 3;
const UNREACHABLE: u8 = 4;

#[derive(Clone, Copy)]
struct Cell {
    scores: [i64; 3],
    sources: [u8; 3],
}
impl Cell {
    const UNSET: Self = Self {
        scores: [0; 3],
        sources: [UNREACHABLE; 3],
    };
    fn set(&mut self, state: usize, score: i64, source: u8) {
        self.scores[state] = score;
        self.sources[state] = source;
    }
    fn get(&self, state: usize) -> Option<i64> {
        (self.sources[state] != UNREACHABLE).then_some(self.scores[state])
    }
    fn best(&self) -> (i64, usize) {
        [MATCH, DELETE, INSERT]
            .into_iter()
            .filter_map(|state| self.get(state).map(|score| (score, state)))
            .max_by_key(|&(score, state)| (score, std::cmp::Reverse(state)))
            .expect("at least one reachable alignment state")
    }
}

fn add_score(score: i64, weight: i64) -> Result<i64, OperationError> {
    score
        .checked_add(weight)
        .ok_or_else(|| parameter("alignment score exceeds signed 64-bit range"))
}

#[derive(Clone, Copy)]
struct Scores {
    match_score: i64,
    mismatch: i64,
    open: i64,
    extend: i64,
    local: bool,
}
impl Scores {
    fn checked(request: &AlignRequest) -> Result<Self, OperationError> {
        let (open, extend) = match request.gap {
            Gap::Linear { penalty } => (penalty, penalty),
            Gap::Affine { open, extend } => (open, extend),
        };
        let local = matches!(request.mode, Mode::Local);
        if ![request.match_score, request.mismatch]
            .iter()
            .all(|&x| (-1_000_000..=1_000_000).contains(&x))
            || open > 0
            || extend > 0
            || local && (request.match_score <= 0 || request.mismatch >= 0)
        {
            return Err(parameter("invalid match, mismatch or gap scoring profile"));
        }
        Ok(Self {
            match_score: request.match_score,
            mismatch: request.mismatch,
            open,
            extend,
            local,
        })
    }
    fn gap(self, previous: Cell, state: usize) -> Result<(i64, u8), OperationError> {
        let opposite = if state == DELETE { INSERT } else { DELETE };
        let mut best = None;
        for (source, weight) in [
            (state, self.extend),
            (MATCH, self.open),
            (opposite, self.open),
        ] {
            if let Some(score) = previous.get(source) {
                let score = add_score(score, weight)?;
                if best.is_none_or(|(previous, _)| score > previous) {
                    best = Some((score, source as u8));
                }
            }
        }
        Ok(best.expect("a gap always has a reachable predecessor"))
    }
}

struct Matrix {
    width: usize,
    cells: Vec<Cell>,
    endpoint: (usize, usize, usize),
    score: i64,
}

fn fill(left: &[u32], right: &[u32], scores: Scores) -> Result<Matrix, OperationError> {
    let width = right.len() + 1;
    let mut grid = vec![Cell::UNSET; cells(left.len() + 1, width, 1 << 20)?];
    grid[0].set(MATCH, 0, STOP);
    let mut optimum = 0;
    let mut endpoint = (0, 0, MATCH);
    for i in 0..=left.len() {
        for j in 0..=right.len() {
            if i == 0 && j == 0 {
                continue;
            }
            let k = i * width + j;
            let mut cell = Cell::UNSET;
            if i > 0 && j > 0 {
                let (score, state) = grid[k - width - 1].best();
                let weight = if left[i - 1] == right[j - 1] {
                    scores.match_score
                } else {
                    scores.mismatch
                };
                cell.set(MATCH, add_score(score, weight)?, state as u8);
            }
            if i > 0 {
                let (score, source) = scores.gap(grid[k - width], DELETE)?;
                cell.set(DELETE, score, source);
            }
            if j > 0 {
                let (score, source) = scores.gap(grid[k - 1], INSERT)?;
                cell.set(INSERT, score, source);
            }
            if scores.local && cell.get(MATCH).is_none_or(|score| score <= 0) {
                cell.set(MATCH, 0, STOP);
            }
            let (value, state) = cell.best();
            if scores.local && value > optimum {
                (optimum, endpoint) = (value, (i, j, state));
            }
            grid[k] = cell;
        }
    }
    if !scores.local {
        let (value, state) = grid[grid.len() - 1].best();
        (optimum, endpoint) = (value, (left.len(), right.len(), state));
    }
    Ok(Matrix {
        width,
        cells: grid,
        endpoint,
        score: optimum,
    })
}

fn trace(grid: &Matrix, left: &Indexed, right: &Indexed, local: bool) -> (Span, Span, Vec<Run>) {
    let (end_left, end_right, mut state) = grid.endpoint;
    let (mut i, mut j) = (end_left, end_right);
    let mut spans: Vec<(EditKind, usize, usize, usize, usize)> = Vec::new();
    while i > 0 || j > 0 {
        let cell = grid.cells[i * grid.width + j];
        if local && cell.scores[state] <= 0 {
            break;
        }
        let (old_i, old_j) = (i, j);
        let kind = match state {
            MATCH => {
                i -= 1;
                j -= 1;
                if left.units[i] == right.units[j] {
                    EditKind::Equal
                } else {
                    EditKind::Substitute
                }
            }
            DELETE => {
                i -= 1;
                EditKind::Delete
            }
            _ => {
                j -= 1;
                EditKind::Insert
            }
        };
        if let Some(last) = spans.last_mut().filter(|last| last.0 == kind) {
            last.1 = i;
            last.3 = j;
        } else {
            spans.push((kind, i, old_i, j, old_j));
        }
        let source = cell.sources[state];
        if source == STOP {
            break;
        }
        state = usize::from(source);
    }
    let runs = spans
        .into_iter()
        .rev()
        .map(|(kind, a, b, c, d)| Run {
            kind,
            left: left.span(a, b - a),
            right: right.span(c, d - c),
        })
        .collect();
    (
        left.span(i, end_left - i),
        right.span(j, end_right - j),
        runs,
    )
}

/// Align exact input units with auditable source spans and deterministic ties.
pub fn align(request: &AlignRequest) -> Result<AlignReport, OperationError> {
    let scores = Scores::checked(request)?;
    let (left, right) = request.operands.indexed(16384)?;
    let grid = fill(&left.units, &right.units, scores)?;
    let (left, right, runs) = trace(&grid, &left, &right, scores.local);
    Ok(AlignReport {
        score: grid.score,
        left,
        right,
        runs,
    })
}
