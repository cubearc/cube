//! Synchronous finite-domain Life and elementary cellular automata.
use crate::{
    errors::OperationError,
    formal_work::{invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_CELLS: usize = 1 << 20;
const MAX_GENERATIONS: usize = 4096;
const MAX_WORK: usize = 16 << 20;
const MAX_REPORT: usize = 8 << 20;

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    #[default]
    FiniteDead,
    Toroidal,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub initial: Initial,
    pub generations: usize,
    #[serde(default)]
    pub boundary: Boundary,
    #[serde(default)]
    pub snapshots: Vec<usize>,
    pub max_work: Option<usize>,
    pub max_report_bytes: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Initial {
    Life {
        rows: usize,
        cols: usize,
        cells: String,
    },
    Elementary {
        rule: u8,
        cells: String,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Rule {
    Life { rule: &'static str },
    Elementary { rule: u8 },
}

#[derive(Serialize, JsonSchema)]
pub struct Snapshot {
    pub generation: usize,
    pub cells: String,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub rule: Rule,
    pub rows: usize,
    pub cols: usize,
    pub boundary: Boundary,
    pub final_grid: Snapshot,
    pub snapshots: Vec<Snapshot>,
    pub work: usize,
}

fn offset(at: usize, length: usize, direction: i8, wrapped: bool) -> Option<usize> {
    match direction {
        -1 => at.checked_sub(1).or_else(|| wrapped.then_some(length - 1)),
        0 => Some(at),
        _ if at + 1 < length => Some(at + 1),
        _ => wrapped.then_some(0),
    }
}

fn row_output(rule: u8, boundary: bool, current: &[u8], next: &mut [u8]) {
    for (i, item) in next.iter_mut().enumerate() {
        let neighbor = |direction| {
            offset(i, current.len(), direction, boundary).map_or(0, |p| current[p] - b'0')
        };
        let index = 4 * neighbor(-1) + 2 * neighbor(0) + neighbor(1);
        *item = b'0' + ((rule >> index) & 1);
    }
}

fn life_output(rows: usize, cols: usize, boundary: bool, current: &[u8], next: &mut [u8]) {
    for (index, item) in next.iter_mut().enumerate() {
        let (row, col) = (index / cols, index % cols);
        let mut count = 0;
        for dr in -1..=1 {
            let Some(r) = offset(row, rows, dr, boundary) else {
                continue;
            };
            for dc in -1..=1 {
                if (dr, dc) != (0, 0)
                    && let Some(c) = offset(col, cols, dc, boundary)
                {
                    count += current[r * cols + c] - b'0';
                }
            }
        }
        *item = if count == 3 || count == 2 && current[index] == b'1' {
            b'1'
        } else {
            b'0'
        };
    }
}

fn snapshot(generation: usize, cells: &[u8]) -> Snapshot {
    Snapshot {
        generation,
        cells: String::from_utf8(cells.to_vec()).expect("binary ASCII cells"),
    }
}

/// Evolve a declared finite grid with synchronous updates and explicit snapshots.
pub fn evolve(request: &Request) -> Result<Report, OperationError> {
    run(request).map_err(|e| e.operation("esolang.cellular_automaton"))
}

fn run(request: &Request) -> Result<Report, OperationError> {
    let (rows, cols, cells, rule) = match &request.initial {
        Initial::Life { rows, cols, cells } => (*rows, *cols, cells, Rule::Life { rule: "B3/S23" }),
        Initial::Elementary { rule, cells } => {
            (1, cells.len(), cells, Rule::Elementary { rule: *rule })
        }
    };
    if rows > MAX_CELLS
        || cols > MAX_CELLS
        || cells.len() > MAX_CELLS
        || rows.checked_mul(cols).is_none_or(|n| n > MAX_CELLS)
    {
        return Err(resource("cellular grid exceeds 1048576 cells or dimension"));
    }
    if rows * cols != cells.len() || !cells.bytes().all(|b| matches!(b, b'0' | b'1')) {
        return Err(invalid(
            "grid requires exact row-major ASCII binary cells matching its shape",
        ));
    }
    if request.generations > MAX_GENERATIONS || request.snapshots.len() > MAX_GENERATIONS + 1 {
        return Err(resource(
            "cellular request exceeds the generation or snapshot count ceiling",
        ));
    }
    if request
        .snapshots
        .iter()
        .any(|&index| index > request.generations)
    {
        return Err(parameter(
            "snapshot generation is outside the requested evolution",
        ));
    }
    let work = cells.len() * request.generations;
    let work_limit = request.max_work.unwrap_or(MAX_WORK);
    let report_limit = request.max_report_bytes.unwrap_or(MAX_REPORT);
    if work_limit > MAX_WORK || report_limit > MAX_REPORT {
        return Err(parameter(
            "requested cellular budget exceeds its hard ceiling",
        ));
    }
    if work > work_limit {
        return Err(resource("cell updates exceed the selected work budget"));
    }
    let report_bytes = 256 + (cells.len() + 36) * (request.snapshots.len() + 1);
    if report_bytes > report_limit {
        return Err(resource("retained snapshots exceed the report byte budget"));
    }
    let mut order: Vec<_> = request
        .snapshots
        .iter()
        .copied()
        .enumerate()
        .map(|(index, generation)| (generation, index))
        .collect();
    order.sort_unstable();
    let mut order = order.into_iter().peekable();
    let mut snapshots: Vec<_> = (0..request.snapshots.len()).map(|_| None).collect();
    let mut current = cells.as_bytes().to_vec();
    let mut next = vec![b'0'; cells.len()];
    let wrapped = matches!(request.boundary, Boundary::Toroidal);
    for generation in 0..=request.generations {
        while order.peek().is_some_and(|&(next, _)| next == generation) {
            let (_, index) = order.next().expect("checked generation");
            snapshots[index] = Some(snapshot(generation, &current));
        }
        if generation < request.generations {
            match rule {
                Rule::Life { .. } => life_output(rows, cols, wrapped, &current, &mut next),
                Rule::Elementary { rule } => row_output(rule, wrapped, &current, &mut next),
            }
            std::mem::swap(&mut current, &mut next);
        }
    }
    Ok(Report {
        rule,
        rows,
        cols,
        boundary: request.boundary,
        final_grid: snapshot(request.generations, &current),
        snapshots: snapshots
            .into_iter()
            .map(|item| item.expect("validated generation"))
            .collect(),
        work,
    })
}
