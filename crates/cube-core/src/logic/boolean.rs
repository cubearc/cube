//! Exact bounded propositional evaluation. Assignment order is declared variable
//! order, last variable changing fastest, false before true. Every formula node
//! is visited (no short circuit); exhausted scans return no mathematical claims.
mod metadata;
use crate::errors::{ErrorCode, OperationError};
pub use metadata::OPERATIONS;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_VARIABLES: usize = 12;
pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: usize = 32;
pub const MAX_WORK: u64 = 2 * 1024 * 1024;
pub const MAX_MEMORY: usize = 8 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Formula {
    Const {
        value: bool,
    },
    Var {
        name: String,
    },
    Not {
        operand: Box<Formula>,
    },
    And {
        left: Box<Formula>,
        right: Box<Formula>,
    },
    Or {
        left: Box<Formula>,
        right: Box<Formula>,
    },
    Xor {
        left: Box<Formula>,
        right: Box<Formula>,
    },
    Implies {
        left: Box<Formula>,
        right: Box<Formula>,
    },
    Iff {
        left: Box<Formula>,
        right: Box<Formula>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub variables: Vec<String>,
    pub formula: Formula,
    #[serde(default)]
    pub other: Option<Formula>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub name: String,
    pub value: bool,
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Limits {
    pub max_work: u64,
    pub max_rows: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_rows: 32,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Evaluation {
    pub value: bool,
    pub other_value: Option<bool>,
    pub work: u64,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TruthRow {
    pub assignment: Vec<Binding>,
    pub value: bool,
    pub other_value: Option<bool>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Relationship {
    pub implies: bool,
    pub equivalent: bool,
    pub implication_counterexample: Option<Vec<Binding>>,
    pub equivalence_counterexample: Option<Vec<Binding>>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Analysis {
    pub variables: Vec<String>,
    pub assignments: usize,
    pub true_count: usize,
    pub false_count: usize,
    pub satisfiable: bool,
    pub tautology: bool,
    pub first_satisfying: Option<Vec<Binding>>,
    pub first_falsifying: Option<Vec<Binding>>,
    pub relationship: Option<Relationship>,
    pub rows: Vec<TruthRow>,
    pub listing_complete: bool,
    pub work: u64,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
fn error(op: &'static str, code: ErrorCode, name: &'static str, msg: &str) -> OperationError {
    OperationError::new(code, msg).operation(op).parameter(name)
}
fn invalid(op: &'static str, name: &'static str, msg: &str) -> OperationError {
    error(op, ErrorCode::InvalidInput, name, msg)
}
/// Parse a fixed tagged schema using the shared duplicate-key/depth/node-bounded
/// JSON reader. Formula objects are counted/depth-checked before Box allocation.
pub fn parse(document: &str) -> Result<Model, OperationError> {
    const OP: &str = "logic.boolean.analyze";
    let value = crate::pipeline::json::parse_with_limits(document, MAX_INPUT, 96, 4096)
        .map_err(|e| (*e.error).operation(OP))?;
    let mut stack = vec![(&value, 0usize)];
    let mut count = 0;
    while let Some((v, depth)) = stack.pop() {
        match v {
            serde_json::Value::Object(m) => {
                let d = depth + usize::from(m.contains_key("op"));
                if m.contains_key("op") {
                    count += 1;
                    if count > MAX_NODES || d > MAX_DEPTH {
                        return Err(error(
                            OP,
                            ErrorCode::ResourceLimit,
                            "formula",
                            "formula exceeds 256 combined nodes or depth 32",
                        ));
                    }
                }
                for v in m.values() {
                    stack.push((v, d));
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    stack.push((v, depth));
                }
            }
            _ => (),
        }
    }
    let model: Model = serde_json::from_value(value)
        .map_err(|e| invalid(OP, "model", "invalid tagged formula/model schema").caused_by(e))?;
    validate(&model, Limits::default(), false, OP)?;
    Ok(model)
}
/// Ordered named bindings, not a map that could discard duplicate assignments.
pub fn parse_assignment(document: &str) -> Result<Vec<Binding>, OperationError> {
    const OP: &str = "logic.boolean.evaluate";
    let value = crate::pipeline::json::parse_with_limits(document, MAX_INPUT, 8, 128)
        .map_err(|e| (*e.error).operation(OP))?;
    let bindings: Vec<Binding> = serde_json::from_value(value).map_err(|e| {
        invalid(
            OP,
            "assignment",
            "assignment must be an array of explicit name/Boolean records",
        )
        .caused_by(e)
    })?;
    if bindings.len() > MAX_VARIABLES {
        return Err(invalid(
            OP,
            "assignment",
            "at most 12 bindings are supported",
        ));
    }
    Ok(bindings)
}
struct Validated {
    indices: BTreeMap<String, usize>,
    nodes: usize,
    memory: usize,
    report: usize,
}
fn validate(
    model: &Model,
    limits: Limits,
    listing: bool,
    op: &'static str,
) -> Result<Validated, OperationError> {
    if limits.max_work > MAX_WORK
        || limits.max_rows > 4096
        || limits.max_memory_bytes > MAX_MEMORY
        || limits.max_report_bytes > MAX_REPORT
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "limits",
            "Boolean limits exceed hard ceilings",
        ));
    }
    if model.variables.len() > MAX_VARIABLES {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "variables",
            "at most 12 declared variables are supported",
        ));
    }
    let mut indices = BTreeMap::new();
    for (i, name) in model.variables.iter().enumerate() {
        if name.is_empty() || name.len() > 64 || indices.insert(name.clone(), i).is_some() {
            return Err(invalid(
                op,
                "variables",
                "variable IDs must be unique nonempty UTF-8 strings <=64 bytes",
            ));
        }
    }
    let mut stack = vec![(&model.formula, 1usize)];
    if let Some(other) = &model.other {
        stack.push((other, 1));
    }
    let mut nodes = 0;
    while let Some((f, depth)) = stack.pop() {
        nodes += 1;
        if nodes > MAX_NODES || depth > MAX_DEPTH {
            return Err(error(
                op,
                ErrorCode::ResourceLimit,
                "formula",
                "formula exceeds 256 combined nodes or depth 32",
            ));
        }
        match f {
            Formula::Var { name } => {
                if !indices.contains_key(name) {
                    return Err(invalid(
                        op,
                        "formula",
                        "formula references an undeclared variable",
                    ));
                }
            }
            Formula::Not { operand } => stack.push((operand, depth + 1)),
            Formula::And { left, right }
            | Formula::Or { left, right }
            | Formula::Xor { left, right }
            | Formula::Implies { left, right }
            | Formula::Iff { left, right } => {
                stack.push((right, depth + 1));
                stack.push((left, depth + 1));
            }
            _ => (),
        }
    }
    let row_count = if listing {
        limits.max_rows.min(1 << model.variables.len())
    } else {
        0
    };
    let bindings_json: usize = model
        .variables
        .iter()
        .map(|s| 32 + crate::pipeline::json::string_bytes(s))
        .sum();
    let bindings_memory: usize = model.variables.iter().map(|s| 32 + s.len()).sum();
    let report = 4096 + (row_count + 4) * (128 + bindings_json);
    let memory = 2 * MAX_INPUT
        + nodes * 512
        + model.variables.len() * 256
        + (row_count + 4) * (128 + bindings_memory)
        + report;
    if report > limits.max_report_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_report_bytes",
            "conservative Boolean report bound exceeds budget",
        ));
    }
    if memory > limits.max_memory_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_memory_bytes",
            "conservative Boolean allocation bound exceeds budget",
        ));
    }
    Ok(Validated {
        indices,
        nodes,
        memory,
        report,
    })
}
fn eval(f: &Formula, values: &[bool], indices: &BTreeMap<String, usize>) -> bool {
    match f {
        Formula::Const { value } => *value,
        Formula::Var { name } => values[indices[name]],
        Formula::Not { operand } => !eval(operand, values, indices),
        Formula::And { left, right }
        | Formula::Or { left, right }
        | Formula::Xor { left, right }
        | Formula::Implies { left, right }
        | Formula::Iff { left, right } => {
            let a = eval(left, values, indices);
            let b = eval(right, values, indices);
            match f {
                Formula::And { .. } => a & b,
                Formula::Or { .. } => a | b,
                Formula::Xor { .. } => a ^ b,
                Formula::Implies { .. } => !a | b,
                _ => a == b,
            }
        }
    }
}
fn witness(model: &Model, values: &[bool]) -> Vec<Binding> {
    model
        .variables
        .iter()
        .zip(values)
        .map(|(name, &value)| Binding {
            name: name.clone(),
            value,
        })
        .collect()
}
/// Evaluate with exactly one Boolean per declared variable. Missing, duplicate,
/// extra and unknown bindings fail before any formula evaluation.
pub fn evaluate(
    model: &Model,
    assignment: &[Binding],
    limits: Limits,
) -> Result<Evaluation, OperationError> {
    const OP: &str = "logic.boolean.evaluate";
    let v = validate(model, limits, false, OP)?;
    if assignment.len() != model.variables.len() {
        return Err(invalid(
            OP,
            "assignment",
            "require exactly one binding per declared variable",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut values = vec![false; model.variables.len()];
    for b in assignment {
        let Some(&i) = v.indices.get(&b.name) else {
            return Err(invalid(OP, "assignment", "unknown assigned variable"));
        };
        if !seen.insert(i) {
            return Err(invalid(OP, "assignment", "duplicate assigned variable"));
        }
        values[i] = b.value;
    }
    if v.nodes as u64 > limits.max_work {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "max_work",
            "Boolean node visit budget exhausted",
        ));
    }
    Ok(Evaluation {
        value: eval(&model.formula, &values, &v.indices),
        other_value: model.other.as_ref().map(|f| eval(f, &values, &v.indices)),
        work: v.nodes as u64,
        memory_bytes_upper_bound: v.memory,
        report_bytes_upper_bound: v.report,
    })
}
/// Complete truth scan; listed rows are independent of exact global claims.
/// The last declared variable changes fastest. A constant-only model has one row.
/// A comparison uses the same declared union order and evaluator for both trees.
pub fn analyze(model: &Model, limits: Limits) -> Result<Analysis, OperationError> {
    const OP: &str = "logic.boolean.analyze";
    let v = validate(model, limits, true, OP)?;
    let assignments = 1usize << model.variables.len();
    let work = (v.nodes as u64) * (assignments as u64);
    if work > limits.max_work {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "max_work",
            "complete truth scan exceeds node visit budget; no global conclusion produced",
        ));
    }
    let mut true_count = 0;
    let mut first_satisfying = None;
    let mut first_falsifying = None;
    let mut rows = Vec::with_capacity(limits.max_rows.min(assignments));
    let mut relation = model.other.as_ref().map(|_| Relationship {
        implies: true,
        equivalent: true,
        implication_counterexample: None,
        equivalence_counterexample: None,
    });
    for mask in 0..assignments {
        let values: Vec<bool> = (0..model.variables.len())
            .map(|i| mask & (1 << (model.variables.len() - 1 - i)) != 0)
            .collect();
        let value = eval(&model.formula, &values, &v.indices);
        let other_value = model.other.as_ref().map(|f| eval(f, &values, &v.indices));
        if value {
            true_count += 1;
            if first_satisfying.is_none() {
                first_satisfying = Some(witness(model, &values));
            }
        } else if first_falsifying.is_none() {
            first_falsifying = Some(witness(model, &values));
        }
        if let (Some(r), Some(b)) = (&mut relation, other_value) {
            if value && !b {
                r.implies = false;
                if r.implication_counterexample.is_none() {
                    r.implication_counterexample = Some(witness(model, &values));
                }
            }
            if value != b {
                r.equivalent = false;
                if r.equivalence_counterexample.is_none() {
                    r.equivalence_counterexample = Some(witness(model, &values));
                }
            }
        }
        if rows.len() < limits.max_rows {
            rows.push(TruthRow {
                assignment: witness(model, &values),
                value,
                other_value,
            });
        }
    }
    let complete = rows.len() == assignments;
    Ok(Analysis {
        variables: model.variables.clone(),
        assignments,
        true_count,
        false_count: assignments - true_count,
        satisfiable: true_count > 0,
        tautology: true_count == assignments,
        first_satisfying,
        first_falsifying,
        relationship: relation,
        rows,
        listing_complete: complete,
        work,
        memory_bytes_upper_bound: v.memory,
        report_bytes_upper_bound: v.report,
    })
}
