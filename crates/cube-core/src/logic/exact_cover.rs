//! Bounded Algorithm X over reversible sparse dancing links.
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, parameter, resource},
    pipeline::json,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const MAX_WORK: usize = 1 << 20;
const MAX_OUTPUT: usize = 8 << 20;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub id: String,
    pub columns: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub max_solutions: usize,
    pub max_work: Option<usize>,
    pub max_report_bytes: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Complete,
    SolutionLimit,
    WorkLimit,
    ReportLimit,
    Cancelled,
}

impl From<Stop> for Reason {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::WorkLimit => Self::WorkLimit,
            Stop::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub solutions: Vec<Vec<String>>,
    pub complete: bool,
    pub stop_reason: Reason,
    pub work: usize,
}

fn incidence(request: &Request) -> Result<(Vec<Vec<usize>>, usize), OperationError> {
    if request.columns.len() > 256 || request.rows.len() > 4096 {
        return Err(resource("exact cover exceeds 256 columns or 4096 rows"));
    }
    let mut columns = BTreeMap::new();
    let valid_id = |text: &str| !text.is_empty() && text.len() <= 64;
    for (index, id) in request.columns.iter().enumerate() {
        if !valid_id(id) || columns.insert(id.as_str(), index + 1).is_some() {
            return Err(invalid(
                "column IDs must be unique, nonempty, and at most 64 UTF-8 bytes",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    let mut rows = Vec::with_capacity(request.rows.len());
    for row in &request.rows {
        if !valid_id(&row.id) || !seen.insert(row.id.as_str()) {
            return Err(invalid(
                "row IDs must be unique, nonempty, and at most 64 UTF-8 bytes",
            ));
        }
        if row.columns.is_empty() {
            return Err(invalid("exact cover requires nonempty rows"));
        }
        total += row.columns.len();
        if total > 65536 {
            return Err(resource("exact cover exceeds 65536 incidence nodes"));
        }
        if row.columns.len() > columns.len() {
            return Err(invalid("row has a duplicate or unknown column"));
        }
        let mut used = [false; 257];
        let mut members = Vec::with_capacity(row.columns.len());
        for id in &row.columns {
            let &column = columns
                .get(id.as_str())
                .ok_or_else(|| invalid("row refers to an unknown column"))?;
            if std::mem::replace(&mut used[column], true) {
                return Err(invalid("row has a duplicate column"));
            }
            members.push(column);
        }
        rows.push(members);
    }
    Ok((rows, total))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node {
    left: usize,
    right: usize,
    up: usize,
    down: usize,
    col: usize,
    row: usize,
}

struct Links {
    nodes: Vec<Node>,
    sizes: Vec<usize>,
}

struct Covered {
    head: usize,
    removed: Vec<usize>,
}

impl Links {
    fn new(
        columns: usize,
        rows: &[Vec<usize>],
        total: usize,
        work: &mut Work<'_>,
    ) -> Result<Self, Stop> {
        let heads = columns + 1;
        let mut nodes = Vec::with_capacity(heads + total);
        for i in 0..heads {
            work.charge(usize::from(i > 0))?;
            nodes.push(Node {
                left: (i + heads - 1) % heads,
                right: (i + 1) % heads,
                up: i,
                down: i,
                col: i,
                row: 0,
            });
        }
        let mut sizes = vec![0; heads];
        for (row, memberships) in rows.iter().enumerate() {
            work.charge(1)?;
            let first = nodes.len();
            for (position, &col) in memberships.iter().enumerate() {
                work.charge(1)?;
                let i = nodes.len();
                let up = nodes[col].up;
                nodes.push(Node {
                    left: if position == 0 {
                        first + memberships.len() - 1
                    } else {
                        i - 1
                    },
                    right: if position + 1 == memberships.len() {
                        first
                    } else {
                        i + 1
                    },
                    up,
                    down: col,
                    col,
                    row,
                });
                nodes[up].down = i;
                nodes[col].up = i;
                sizes[col] += 1;
            }
        }
        Ok(Self { nodes, sizes })
    }

    fn choose(&self, work: &mut Work<'_>) -> Result<usize, Stop> {
        let mut head = self.nodes[0].right;
        let mut cursor = self.nodes[head].right;
        while cursor != 0 {
            work.charge(1)?;
            if self.sizes[cursor] < self.sizes[head] {
                head = cursor;
            }
            cursor = self.nodes[cursor].right;
        }
        Ok(head)
    }

    fn cover(&mut self, head: usize, work: &mut Work<'_>) -> Result<Covered, Stop> {
        work.charge(1)?;
        let node = self.nodes[head];
        self.nodes[node.left].right = node.right;
        self.nodes[node.right].left = node.left;
        let mut record = Covered {
            head,
            removed: Vec::new(),
        };
        if let Err(stop) = self.unlink_rows(&mut record, work) {
            self.restore(record, work, Some(stop));
            return Err(stop);
        }
        Ok(record)
    }

    fn unlink_rows(&mut self, record: &mut Covered, work: &mut Work<'_>) -> Result<(), Stop> {
        let mut row = self.nodes[record.head].down;
        while row != record.head {
            work.charge(1)?;
            let mut cursor = self.nodes[row].right;
            while cursor != row {
                work.charge(1)?;
                let node = self.nodes[cursor];
                self.nodes[node.up].down = node.down;
                self.nodes[node.down].up = node.up;
                self.sizes[node.col] -= 1;
                record.removed.push(cursor);
                cursor = node.right;
            }
            row = self.nodes[row].down;
        }
        Ok(())
    }

    fn restore(
        &mut self,
        record: Covered,
        work: &mut Work<'_>,
        mut stopped: Option<Stop>,
    ) -> Option<Stop> {
        for index in record.removed.into_iter().rev() {
            stopped = stopped.or_else(|| work.charge(1).err());
            let node = self.nodes[index];
            self.nodes[node.up].down = index;
            self.nodes[node.down].up = index;
            self.sizes[node.col] += 1;
        }
        stopped = stopped.or_else(|| work.charge(1).err());
        let head = self.nodes[record.head];
        self.nodes[head.left].right = record.head;
        self.nodes[head.right].left = record.head;
        stopped
    }
}

struct Search<'a> {
    rows: &'a [Row],
    chosen: Vec<usize>,
    solutions: Vec<Vec<usize>>,
    limit: usize,
    report_remaining: usize,
}

impl Search<'_> {
    fn record(&mut self) -> Result<(), Reason> {
        if self.solutions.len() == self.limit {
            return Err(Reason::SolutionLimit);
        }
        let size = 3 + self
            .chosen
            .iter()
            .map(|&index| json::string_bytes(&self.rows[index].id) + 1)
            .sum::<usize>();
        self.report_remaining = self
            .report_remaining
            .checked_sub(size)
            .ok_or(Reason::ReportLimit)?;
        let mut members = self.chosen.clone();
        members.sort_unstable();
        self.solutions.push(members);
        Ok(())
    }

    fn search(&mut self, links: &mut Links, work: &mut Work<'_>) -> Result<(), Reason> {
        work.charge(1)?;
        if links.nodes[0].right == 0 {
            work.charge(self.chosen.len() * (self.chosen.len().max(1).ilog2() as usize + 1))?;
            return self.record();
        }
        let head = links.choose(work)?;
        let covered = links.cover(head, work)?;
        let result = self.try_rows(links, head, work);
        let restored = links
            .restore(covered, work, None)
            .map_or(Ok(()), |stop| Err(stop.into()));
        result.and(restored)
    }

    fn try_rows(
        &mut self,
        links: &mut Links,
        head: usize,
        work: &mut Work<'_>,
    ) -> Result<(), Reason> {
        let mut row = links.nodes[head].down;
        while row != head {
            work.charge(1)?;
            self.chosen.push(links.nodes[row].row);
            let result = self.try_row(links, row, work);
            self.chosen.pop();
            result?;
            row = links.nodes[row].down;
        }
        Ok(())
    }

    fn try_row(
        &mut self,
        links: &mut Links,
        row: usize,
        work: &mut Work<'_>,
    ) -> Result<(), Reason> {
        let mut covers = Vec::new();
        let mut cursor = links.nodes[row].right;
        let mut result = Ok(());
        while cursor != row {
            match links.cover(links.nodes[cursor].col, work) {
                Ok(cover) => covers.push(cover),
                Err(stop) => {
                    result = Err(stop.into());
                    break;
                }
            }
            cursor = links.nodes[cursor].right;
        }
        if result.is_ok() {
            result = self.search(links, work);
        }
        for cover in covers.into_iter().rev() {
            let stop = links.restore(cover, work, None);
            if let (Ok(()), Some(stop)) = (&result, stop) {
                result = Err(stop.into());
            }
        }
        result
    }
}

/// Enumerate bounded canonical row-ID exact covers.
pub fn solve(request: &Request) -> Result<Report, OperationError> {
    solve_with_cancel(request, &mut || false)
}

/// Poll the supplied cancellation check at bounded search and link updates.
pub fn solve_with_cancel(
    request: &Request,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    run(request, cancelled).map_err(|e| e.operation("logic.exact_cover"))
}

fn run(request: &Request, cancelled: &mut dyn FnMut() -> bool) -> Result<Report, OperationError> {
    let maximum = request.max_report_bytes.unwrap_or(MAX_OUTPUT);
    if request.max_solutions > 1000 || maximum > MAX_OUTPUT {
        return Err(parameter(
            "exact cover solution/report limit exceeds its hard ceiling",
        ));
    }
    if maximum < 128 {
        return Err(resource(
            "exact cover report budget is smaller than the result envelope",
        ));
    }
    let mut work = Work::new(request.max_work, MAX_WORK, cancelled)?;
    let (rows, total) = incidence(request)?;
    let mut search = Search {
        rows: &request.rows,
        chosen: Vec::new(),
        solutions: Vec::new(),
        limit: request.max_solutions,
        report_remaining: maximum - 128,
    };
    let result = Links::new(request.columns.len(), &rows, total, &mut work)
        .map_err(Reason::from)
        .and_then(|mut links| search.search(&mut links, &mut work));
    let stop_reason = result.err().unwrap_or(Reason::Complete);
    Ok(Report {
        solutions: search
            .solutions
            .into_iter()
            .map(|members| {
                members
                    .into_iter()
                    .map(|row| request.rows[row].id.clone())
                    .collect()
            })
            .collect(),
        complete: stop_reason == Reason::Complete,
        stop_reason,
        work: work.used(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_restores_a_partly_mutated_table() {
        let rows = vec![vec![1, 2, 3], vec![1, 2], vec![2, 3]];
        let mut initializing = || false;
        let mut initialize = Work::new(None, MAX_WORK, &mut initializing).unwrap();
        let baseline = Links::new(3, &rows, 7, &mut initialize).unwrap();
        for cancel_after in 1..=10 {
            let mut links = Links::new(3, &rows, 7, &mut initialize).unwrap();
            let mut polls = 0;
            let mut cancel = || {
                polls += 1;
                polls >= cancel_after
            };
            let mut work = Work::new(None, MAX_WORK, &mut cancel).unwrap();
            if let Ok(record) = links.cover(1, &mut work) {
                links.restore(record, &mut work, None);
            }
            assert_eq!(links.nodes, baseline.nodes);
            assert_eq!(links.sizes, baseline.sizes);
        }
    }
}
