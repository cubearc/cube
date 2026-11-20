//! Deterministic CNF search with explicit completeness and cooperative polling.
pub mod cdcl;
pub use crate::formal_work::Stop;

use crate::{
    errors::OperationError,
    formal_work::{Work, invalid, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_WORK: usize = 1 << 20;
const MAX_VARIABLES: usize = 256;
type Lit = i16;
type Clause = Vec<Lit>;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Formula {
    #[schemars(length(max = 256))]
    pub variables: Vec<u32>,
    #[schemars(length(max = 4096))]
    pub clauses: Vec<Vec<i32>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub formula: Formula,
    #[schemars(range(max = 1048576))]
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct Binding {
    pub variable: u32,
    pub value: bool,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Outcome {
    Sat { assignment: Vec<Binding> },
    Unsat,
    Incomplete { reason: Stop },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub result: Outcome,
    pub work: usize,
}

struct Prepared {
    variables: BTreeMap<u32, usize>,
    clauses: Vec<Clause>,
}

enum CheckError {
    Stop(Stop),
    Operation(OperationError),
}
impl From<Stop> for CheckError {
    fn from(stop: Stop) -> Self {
        Self::Stop(stop)
    }
}
impl From<OperationError> for CheckError {
    fn from(error: OperationError) -> Self {
        Self::Operation(error)
    }
}

fn variable(lit: Lit) -> usize {
    usize::from(lit.unsigned_abs() - 1)
}

impl Prepared {
    fn new(formula: &Formula, work: &mut Work<'_>) -> Result<Self, CheckError> {
        if formula.variables.len() > MAX_VARIABLES || formula.clauses.len() > 4096 {
            return Err(resource("CNF exceeds 256 variables or 4096 clauses").into());
        }
        let mut variables = BTreeMap::new();
        for (index, &id) in formula.variables.iter().enumerate() {
            work.charge(1)?;
            if id == 0 || id > i32::MAX as u32 || variables.insert(id, index).is_some() {
                return Err(
                    invalid("CNF requires distinct positive signed-32 variable IDs").into(),
                );
            }
        }
        let mut clauses = Vec::with_capacity(formula.clauses.len());
        let mut literals = 0;
        for clause in &formula.clauses {
            work.charge(1)?;
            literals += clause.len();
            if literals > 65536 {
                return Err(resource("CNF exceeds 65536 literals").into());
            }
            if let Some(active) = translate(&variables, clause, work)? {
                clauses.push(active);
            }
        }
        Ok(Self { variables, clauses })
    }
}

fn translate(
    variables: &BTreeMap<u32, usize>,
    literals: &[i32],
    work: &mut Work<'_>,
) -> Result<Option<Clause>, CheckError> {
    let mut signs = [0_i16; MAX_VARIABLES];
    let mut tautology = false;
    let mut clause = Vec::with_capacity(literals.len().min(variables.len()));
    for &literal in literals {
        work.charge(1)?;
        let &index = variables
            .get(&literal.unsigned_abs())
            .ok_or_else(|| invalid("CNF literal uses zero or an undeclared variable"))?;
        let sign = literal.signum() as Lit;
        if signs[index] == 0 {
            signs[index] = sign;
            clause.push((index as Lit + 1) * sign);
        } else {
            tautology |= signs[index] != sign;
        }
    }
    Ok((!tautology).then_some(clause))
}

struct Trail {
    values: Vec<i16>,
    levels: Vec<usize>,
    reasons: Vec<Option<usize>>,
    assigned: Vec<Lit>,
}

impl Trail {
    fn new(variables: usize) -> Self {
        Self {
            values: vec![0; variables],
            levels: vec![0; variables],
            reasons: vec![None; variables],
            assigned: Vec::with_capacity(variables),
        }
    }
    fn assign(
        &mut self,
        lit: Lit,
        level: usize,
        reason: Option<usize>,
        work: &mut Work<'_>,
    ) -> Result<(), Stop> {
        work.charge(1)?;
        let v = variable(lit);
        debug_assert_eq!(self.values[v], 0);
        self.values[v] = lit.signum();
        self.levels[v] = level;
        self.reasons[v] = reason;
        self.assigned.push(lit);
        Ok(())
    }

    fn rollback(&mut self, mark: usize, work: &mut Work<'_>) -> Result<(), Stop> {
        while self.assigned.len() > mark {
            work.charge(1)?;
            let v = variable(self.assigned.pop().expect("known length"));
            self.values[v] = 0;
            self.levels[v] = 0;
            self.reasons[v] = None;
        }
        Ok(())
    }

    fn first_unassigned(&self, work: &mut Work<'_>) -> Result<Lit, Stop> {
        for (i, &value) in self.values.iter().enumerate() {
            work.charge(1)?;
            if value == 0 {
                return Ok(i as Lit + 1);
            }
        }
        unreachable!("an undecided clause contains an unassigned variable")
    }

    fn assignment(&self, ids: &[u32], work: &mut Work<'_>) -> Result<Outcome, Stop> {
        work.charge(ids.len())?;
        Ok(Outcome::Sat {
            assignment: ids
                .iter()
                .zip(&self.values)
                .map(|(&variable, &value)| Binding {
                    variable,
                    value: value > 0,
                })
                .collect(),
        })
    }
}

enum Propagation {
    Conflict(usize),
    Satisfied,
    Undecided,
}

enum ClauseState {
    True,
    Empty,
    Unit(Lit),
    Open,
}

fn scan(clause: &[Lit], state: &Trail, work: &mut Work<'_>) -> Result<ClauseState, Stop> {
    let mut unit = None;
    let mut open = false;
    for &lit in clause {
        work.charge(1)?;
        match state.values[variable(lit)] * lit.signum() {
            1 => return Ok(ClauseState::True),
            0 if unit.is_none() => unit = Some(lit),
            0 => open = true,
            _ => (),
        }
    }
    Ok(match (unit, open) {
        (None, _) => ClauseState::Empty,
        (Some(value), false) => ClauseState::Unit(value),
        (Some(_), true) => ClauseState::Open,
    })
}

fn propagate(
    clauses: &[Clause],
    state: &mut Trail,
    level: usize,
    work: &mut Work<'_>,
) -> Result<Propagation, Stop> {
    loop {
        let mut changed = false;
        let mut open = false;
        for (i, clause) in clauses.iter().enumerate() {
            work.charge(1)?;
            match scan(clause, state, work)? {
                ClauseState::True => (),
                ClauseState::Empty => return Ok(Propagation::Conflict(i)),
                ClauseState::Open => open = true,
                ClauseState::Unit(lit) => {
                    state.assign(lit, level, Some(i), work)?;
                    changed = true;
                }
            }
        }
        if !changed {
            return Ok(if open {
                Propagation::Undecided
            } else {
                Propagation::Satisfied
            });
        }
    }
}

struct Branch {
    variable: Lit,
    mark: usize,
    positive: bool,
}

fn backtrack(
    trail: &mut Trail,
    decisions: &mut Vec<Branch>,
    work: &mut Work<'_>,
) -> Result<bool, Stop> {
    while let Some(index) = decisions.len().checked_sub(1) {
        work.charge(1)?;
        let branch = &mut decisions[index];
        trail.rollback(branch.mark, work)?;
        if !branch.positive {
            branch.positive = true;
            trail.assign(branch.variable, index + 1, None, work)?;
            return Ok(true);
        }
        decisions.pop();
    }
    Ok(false)
}

fn search(formula: &Formula, prepared: &Prepared, work: &mut Work<'_>) -> Result<Outcome, Stop> {
    work.charge(0)?;
    let mut trail = Trail::new(formula.variables.len());
    let mut decisions = Vec::new();
    loop {
        match propagate(&prepared.clauses, &mut trail, decisions.len(), work)? {
            Propagation::Satisfied => return trail.assignment(&formula.variables, work),
            Propagation::Conflict(_) => {
                if !backtrack(&mut trail, &mut decisions, work)? {
                    return Ok(Outcome::Unsat);
                }
            }
            Propagation::Undecided => {
                let variable = trail.first_unassigned(work)?;
                decisions.push(Branch {
                    variable,
                    mark: trail.assigned.len(),
                    positive: false,
                });
                trail.assign(-variable, decisions.len(), None, work)?;
            }
        }
    }
}

/// Solve an explicit formula with iterative false-first DPLL.
pub fn dpll(request: &Request) -> Result<Report, OperationError> {
    dpll_with_cancel(request, &mut || false)
}

/// Poll an explicit cancellation callback during normalization admission and search.
pub fn dpll_with_cancel(
    request: &Request,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    let mut run = || {
        let mut work = Work::new(request.max_work, MAX_WORK, cancelled)?;
        let result = match Prepared::new(&request.formula, &mut work) {
            Ok(prepared) => search(&request.formula, &prepared, &mut work)
                .unwrap_or_else(|reason| Outcome::Incomplete { reason }),
            Err(CheckError::Stop(reason)) => Outcome::Incomplete { reason },
            Err(CheckError::Operation(error)) => return Err(error),
        };
        Ok::<_, OperationError>(Report {
            result,
            work: work.used(),
        })
    };
    run().map_err(|e| e.operation("logic.dpll"))
}
