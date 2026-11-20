//! Pure beta/eta and SKI reduction over explicit bounded terms.
mod parse;
mod rewrite;

use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, parameter, resource},
};
use parse::Parsed;
use rewrite::{find, rewrite};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_LIVE: usize = 65536;
const MAX_WORK: usize = 1 << 20;
const MAX_STEPS: usize = 100000;
const MAX_TRACE: usize = 4096;
const MAX_REPORT: usize = 8 << 20;
type Id = usize;

#[derive(Clone, Copy)]
enum Mode {
    Lambda,
    Ski { free_symbols: bool },
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    #[default]
    NormalOrder,
    ApplicativeOrder,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LambdaRequest {
    pub term: String,
    #[serde(default)]
    pub strategy: Strategy,
    #[serde(default)]
    pub eta: bool,
    pub max_reductions: Option<usize>,
    pub max_work: Option<usize>,
    #[serde(default)]
    pub trace: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SkiRequest {
    pub term: String,
    #[serde(default)]
    pub free_symbols: bool,
    pub max_reductions: Option<usize>,
    pub max_work: Option<usize>,
    #[serde(default)]
    pub trace: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    NormalForm,
    ReductionLimit,
    WorkLimit,
    LiveLimit,
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

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    Beta,
    Eta,
    S,
    K,
    I,
}

#[derive(Serialize, JsonSchema)]
pub struct Step {
    pub step: usize,
    pub rule: Rule,
    pub redex_preorder: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub term: String,
    pub normal_form: bool,
    pub steps: usize,
    pub stop_reason: Reason,
    pub trace: Vec<Step>,
    pub omitted_steps: usize,
    pub work: usize,
}

#[derive(Clone, Copy)]
enum Atom {
    Bound(u32),
    Free(u32),
    S,
    K,
    I,
}

#[derive(Clone, Copy)]
enum Kind {
    Atom(Atom),
    Abs(Id),
    App(Id, Id),
}

#[derive(Clone, Copy)]
struct Node {
    kind: Kind,
    height: usize,
}

struct Arena {
    nodes: Vec<Node>,
    cap: usize,
}

impl Arena {
    fn new(cap: usize) -> Self {
        Self {
            nodes: Vec::new(),
            cap,
        }
    }
    fn get(&self, node: Id) -> Node {
        self.nodes[node]
    }
    fn push(&mut self, kind: Kind) -> Result<Id, Reason> {
        if self.nodes.len() == self.cap {
            return Err(Reason::LiveLimit);
        }
        let height = 1 + match kind {
            Kind::Atom(_) => 0,
            Kind::Abs(body) => self.get(body).height,
            Kind::App(a, b) => self.get(a).height.max(self.get(b).height),
        };
        let id = self.nodes.len();
        self.nodes.push(Node { kind, height });
        Ok(id)
    }
}

struct Options<'a> {
    term: &'a str,
    mode: Mode,
    strategy: Strategy,
    eta: bool,
    max_reductions: Option<usize>,
    max_work: Option<usize>,
    trace: usize,
}

/// Reduce a lambda term using full normal or full applicative order.
pub fn lambda(request: &LambdaRequest) -> Result<Report, OperationError> {
    lambda_with_cancel(request, &mut || false)
}

/// Poll the caller during bounded lambda reduction.
pub fn lambda_with_cancel(
    request: &LambdaRequest,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    run(
        Options {
            term: &request.term,
            mode: Mode::Lambda,
            strategy: request.strategy,
            eta: request.eta,
            max_reductions: request.max_reductions,
            max_work: request.max_work,
            trace: request.trace,
        },
        cancelled,
    )
    .map_err(|e| e.operation("esolang.lambda"))
}

/// Reduce a left-associated SKI term by leftmost-outermost rewriting.
pub fn ski(request: &SkiRequest) -> Result<Report, OperationError> {
    ski_with_cancel(request, &mut || false)
}

/// Poll the caller during bounded SKI reduction.
pub fn ski_with_cancel(
    request: &SkiRequest,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    run(
        Options {
            term: &request.term,
            mode: Mode::Ski {
                free_symbols: request.free_symbols,
            },
            strategy: Strategy::NormalOrder,
            eta: false,
            max_reductions: request.max_reductions,
            max_work: request.max_work,
            trace: request.trace,
        },
        cancelled,
    )
    .map_err(|e| e.operation("esolang.ski"))
}

fn run(
    options: Options<'_>,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Report, OperationError> {
    let step_limit = options.max_reductions.unwrap_or(MAX_STEPS);
    if step_limit > MAX_STEPS || options.trace > MAX_TRACE {
        return Err(parameter(
            "requested reduction or trace limit exceeds its hard ceiling",
        ));
    }
    let mut work = Work::new(options.max_work, MAX_WORK, cancelled)?;
    let Parsed {
        mut arena,
        mut root,
        names,
    } = parse::parse(options.term, options.mode)?;
    let mut steps = 0;
    let mut trace = Vec::new();
    let stop_reason = loop {
        let found = match find(&arena, root, &options, &mut work) {
            Ok(Some(found)) => found,
            Ok(None) => break Reason::NormalForm,
            Err(reason) => break reason,
        };
        if steps == step_limit {
            break Reason::ReductionLimit;
        }
        match rewrite(&arena, root, found, &mut work) {
            Ok((next, next_root)) => {
                (arena, root) = (next, next_root);
                steps += 1;
                if trace.len() < options.trace {
                    trace.push(Step {
                        step: steps,
                        rule: found.redex.rule(),
                        redex_preorder: found.preorder,
                    });
                }
            }
            Err(reason) => break reason,
        }
    };
    let term = format(&arena, root, &names)?;
    let omitted_steps = steps - trace.len();
    Ok(Report {
        term,
        normal_form: stop_reason == Reason::NormalForm,
        steps,
        stop_reason,
        trace,
        omitted_steps,
        work: work.used(),
    })
}

// A flat rendering stack keeps deep expanded terms away from the call stack.
enum Print {
    Term(Id),
    Space,
    Close,
    LeaveBinder,
}

fn format(arena: &Arena, root: Id, names: &[String]) -> Result<String, OperationError> {
    let mut text = String::new();
    let mut stack = vec![Print::Term(root)];
    let mut binders = Vec::<String>::new();
    let free: BTreeSet<_> = names.iter().map(String::as_str).collect();
    let mut fresh = 0;
    while let Some(task) = stack.pop() {
        match task {
            Print::Space => text.push(' '),
            Print::Close => text.push(')'),
            Print::LeaveBinder => {
                binders.pop();
                text.push(')');
            }
            Print::Term(id) => match arena.get(id).kind {
                Kind::Atom(Atom::S) => text.push('S'),
                Kind::Atom(Atom::K) => text.push('K'),
                Kind::Atom(Atom::I) => text.push('I'),
                Kind::Atom(Atom::Free(name)) => text.push_str(&names[name as usize]),
                Kind::Atom(Atom::Bound(depth)) => {
                    text.push_str(&binders[binders.len() - 1 - depth as usize])
                }
                Kind::App(left, right) => {
                    text.push('(');
                    stack.extend([
                        Print::Close,
                        Print::Term(right),
                        Print::Space,
                        Print::Term(left),
                    ]);
                }
                Kind::Abs(body) => {
                    let name = loop {
                        let candidate = format!("v{fresh}");
                        fresh += 1;
                        if !free.contains(candidate.as_str()) {
                            break candidate;
                        }
                    };
                    text.push_str("(\\");
                    text.push_str(&name);
                    text.push('.');
                    binders.push(name);
                    stack.extend([Print::LeaveBinder, Print::Term(body)]);
                }
            },
        }
        // Reserve space for JSON escaping and the bounded trace envelope.
        if text.len() > MAX_REPORT / 2 {
            return Err(resource(
                "rendered term exceeds its serialized report budget",
            ));
        }
    }
    Ok(text)
}
