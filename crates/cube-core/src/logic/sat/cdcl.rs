//! Bounded first-UIP learning and a separately callable RUP proof verifier.
use super::{
    CheckError, Clause, Formula, Lit, MAX_VARIABLES, MAX_WORK, Outcome, Prepared, Propagation,
    Trail, propagate, translate, variable,
};
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_LEARNED: usize = 65536;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Solve {
        formula: Formula,
        max_work: Option<usize>,
        max_conflicts: Option<usize>,
        max_learned_literals: Option<usize>,
        #[serde(default)]
        proof: bool,
    },
    VerifyProof {
        formula: Formula,
        clauses: Vec<Vec<i32>>,
        max_work: Option<usize>,
    },
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchLimit {
    WorkLimit,
    Cancelled,
    ConflictLimit,
    LearnedLiteralLimit,
}

impl From<Stop> for SearchLimit {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::WorkLimit => Self::WorkLimit,
            Stop::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResultKind {
    Sat { assignment: Vec<super::Binding> },
    Unsat { proof: Option<Vec<Vec<i32>>> },
    Incomplete { reason: SearchLimit },
    Verified,
    InvalidProof { clause_index: usize },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub profile: &'static str,
    pub result: ResultKind,
    pub work: usize,
    pub conflicts: usize,
    pub learned_literals: usize,
}

struct Learning {
    trail: Trail,
    original: usize,
    clauses: Vec<Clause>,
    conflicts: usize,
    learned_literals: usize,
    conflict_limit: usize,
    learned_limit: usize,
    /// Marks where each decision level starts in the assignment trail.
    marks: Vec<usize>,
}

impl Learning {
    fn after_conflict(&mut self, clause: usize, work: &mut Work<'_>) -> Result<bool, SearchLimit> {
        if self.conflicts == self.conflict_limit {
            return Err(SearchLimit::ConflictLimit);
        }
        work.charge(1)?;
        self.conflicts += 1;
        if self.marks.is_empty() {
            return Ok(false);
        }
        let (learned, jump) = self.first_uip(clause, work)?;
        if self.learned_literals + learned.len() > self.learned_limit {
            return Err(SearchLimit::LearnedLiteralLimit);
        }
        let mark = self.marks[jump];
        self.trail.rollback(mark, work)?;
        self.marks.truncate(jump);
        let assertion = learned[0];
        let reason = self.clauses.len();
        self.learned_literals += learned.len();
        self.clauses.push(learned);
        self.trail.assign(assertion, jump, Some(reason), work)?;
        Ok(true)
    }

    fn first_uip(&self, conflict: usize, work: &mut Work<'_>) -> Result<(Clause, usize), Stop> {
        let level = self.marks.len();
        let mut clause = Resolvent {
            signs: [0; MAX_VARIABLES],
            current: 0,
            level,
        };
        clause.include(&self.clauses[conflict], None, &self.trail, work)?;
        let mut prior = self.trail.assigned.iter().rev();
        while clause.current > 1 {
            work.charge(1)?;
            let literal = *prior.next().expect("conflict literals were assigned");
            let pivot = variable(literal);
            if clause.signs[pivot] == 0 || self.trail.levels[pivot] != level {
                continue;
            }
            clause.signs[pivot] = 0;
            clause.current -= 1;
            let cause = self.trail.reasons[pivot]
                .expect("only the last current-level literal can be a decision");
            clause.include(&self.clauses[cause], Some(pivot), &self.trail, work)?;
        }
        clause.finish(&self.trail, work)
    }

    fn solve(
        &mut self,
        formula: &Formula,
        work: &mut Work<'_>,
        proof: bool,
    ) -> Result<ResultKind, SearchLimit> {
        loop {
            match propagate(&self.clauses, &mut self.trail, self.marks.len(), work)? {
                Propagation::Satisfied => {
                    let Outcome::Sat { assignment } =
                        self.trail.assignment(&formula.variables, work)?
                    else {
                        unreachable!()
                    };
                    return Ok(ResultKind::Sat { assignment });
                }
                Propagation::Conflict(index) => {
                    if !self.after_conflict(index, work)? {
                        let certificate =
                            proof.then(|| self.certificate(formula, work)).transpose()?;
                        return Ok(ResultKind::Unsat { proof: certificate });
                    }
                }
                Propagation::Undecided => {
                    let variable = self.trail.first_unassigned(work)?;
                    self.marks.push(self.trail.assigned.len());
                    self.trail.assign(-variable, self.marks.len(), None, work)?;
                }
            }
        }
    }

    fn certificate(&self, formula: &Formula, work: &mut Work<'_>) -> Result<Vec<Vec<i32>>, Stop> {
        work.charge(self.learned_literals + self.clauses.len() - self.original + 1)?;
        let mut certificate: Vec<Vec<i32>> = self.clauses[self.original..]
            .iter()
            .map(|clause| {
                clause
                    .iter()
                    .map(|&lit| i32::from(lit.signum()) * formula.variables[variable(lit)] as i32)
                    .collect()
            })
            .collect();
        certificate.push(Vec::new());
        Ok(certificate)
    }
}

struct Resolvent {
    signs: [Lit; MAX_VARIABLES],
    current: usize,
    level: usize,
}

impl Resolvent {
    fn include(
        &mut self,
        clause: &[Lit],
        pivot: Option<usize>,
        trail: &Trail,
        work: &mut Work<'_>,
    ) -> Result<(), Stop> {
        for &lit in clause {
            work.charge(1)?;
            let index = variable(lit);
            if Some(index) == pivot || self.signs[index] != 0 {
                continue;
            }
            debug_assert_eq!(trail.values[index], -lit.signum());
            self.signs[index] = lit;
            self.current += usize::from(trail.levels[index] == self.level);
        }
        Ok(())
    }

    fn finish(self, trail: &Trail, work: &mut Work<'_>) -> Result<(Clause, usize), Stop> {
        work.charge(trail.values.len())?;
        let mut learned = vec![0];
        let mut jump = 0;
        for (i, &lit) in self.signs[..trail.values.len()].iter().enumerate() {
            if lit == 0 {
                continue;
            }
            if trail.levels[i] == self.level {
                learned[0] = lit;
            } else {
                jump = jump.max(trail.levels[i]);
                learned.push(lit);
            }
        }
        debug_assert_ne!(learned[0], 0);
        Ok((learned, jump))
    }
}

fn verify_proof(
    mut known: Prepared,
    incoming: &[Vec<i32>],
    variables: usize,
    work: &mut Work<'_>,
) -> Result<ResultKind, OperationError> {
    if incoming.is_empty() || incoming.last().is_some_and(|step| !step.is_empty()) {
        return Err(invalid("RUP proof must end with an empty clause"));
    }
    let mut size = 0;
    for (i, clause) in incoming.iter().enumerate() {
        size += clause.len();
        if i > MAX_LEARNED || size > MAX_LEARNED {
            return Err(resource(
                "RUP proof exceeds the learned literal and clause ceiling",
            ));
        }
        if clause.is_empty() && i + 1 != incoming.len() {
            return Err(invalid(
                "RUP proof may contain the empty clause only at its end",
            ));
        }
    }
    let check = (|| {
        work.charge(0)?;
        for (i, literals) in incoming.iter().enumerate() {
            work.charge(1 + variables)?;
            let Some(clause) = translate(&known.variables, literals, work)? else {
                continue;
            };
            let mut trail = Trail::new(variables);
            for &lit in &clause {
                trail.assign(-lit, 0, None, work)?;
            }
            if !matches!(
                propagate(&known.clauses, &mut trail, 0, work)?,
                Propagation::Conflict(_)
            ) {
                return Ok(ResultKind::InvalidProof { clause_index: i });
            }
            known.clauses.push(clause);
        }
        Ok::<_, CheckError>(ResultKind::Verified)
    })();
    match check {
        Ok(result) => Ok(result),
        Err(CheckError::Operation(error)) => Err(error),
        Err(CheckError::Stop(stop)) => Ok(ResultKind::Incomplete {
            reason: stop.into(),
        }),
    }
}

/// Solve or independently replay a RUP proof for the caller-owned formula.
pub fn run(request: &Request) -> Result<Report, OperationError> {
    run_with_cancel(request, &mut || false)
}

/// Poll an explicit cancellation callback during propagation, learning and replay.
pub fn run_with_cancel(
    request: &Request,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    apply(request, cancelled).map_err(|e| e.operation("logic.cdcl"))
}

fn apply(request: &Request, cancelled: &mut dyn FnMut() -> bool) -> Result<Report, OperationError> {
    const PROFILE: &str = "cube_first_uip_rup_v1";
    let (formula, max_work) = match request {
        Request::Solve {
            formula, max_work, ..
        }
        | Request::VerifyProof {
            formula, max_work, ..
        } => (formula, *max_work),
    };
    let mut work = Work::new(max_work, MAX_WORK, cancelled)?;
    let normalized = match Prepared::new(formula, &mut work) {
        Ok(prepared) => prepared,
        Err(CheckError::Operation(error)) => return Err(error),
        Err(CheckError::Stop(stop)) => {
            return Ok(Report {
                profile: PROFILE,
                result: ResultKind::Incomplete {
                    reason: stop.into(),
                },
                conflicts: 0,
                learned_literals: 0,
                work: work.used(),
            });
        }
    };
    let Request::Solve {
        max_conflicts,
        max_learned_literals,
        proof,
        ..
    } = request
    else {
        let Request::VerifyProof { clauses, .. } = request else {
            unreachable!()
        };
        let result = verify_proof(normalized, clauses, formula.variables.len(), &mut work)?;
        return Ok(Report {
            profile: PROFILE,
            result,
            conflicts: 0,
            learned_literals: 0,
            work: work.used(),
        });
    };
    let conflict_limit = max_conflicts.unwrap_or(MAX_WORK);
    let learned_limit = max_learned_literals.unwrap_or(MAX_LEARNED);
    if conflict_limit > MAX_WORK || learned_limit > MAX_LEARNED {
        return Err(parameter(
            "CDCL conflict or learned-literal limit exceeds its hard ceiling",
        ));
    }
    let mut state = Learning {
        trail: Trail::new(formula.variables.len()),
        original: normalized.clauses.len(),
        clauses: normalized.clauses,
        conflicts: 0,
        learned_literals: 0,
        conflict_limit,
        learned_limit,
        marks: Vec::new(),
    };
    let result = work
        .charge(0)
        .map_err(SearchLimit::from)
        .and_then(|()| state.solve(formula, &mut work, *proof))
        .unwrap_or_else(|reason| ResultKind::Incomplete { reason });
    Ok(Report {
        profile: PROFILE,
        result,
        conflicts: state.conflicts,
        learned_literals: state.learned_literals,
        work: work.used(),
    })
}
