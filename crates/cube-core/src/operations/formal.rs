//! Unified named-port transport for graph, logic, term and stream operations.
use crate::{
    errors::OperationError,
    esolang::{cellular, terms},
    formal_work::{check_bytes, invalid},
    graph::{disjoint, matching, traversal, weighted},
    logic::{exact_cover, sat},
    numbers::streams,
};

use crate::operations::{self as o, contracts as c};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}

struct Row {
    contract: c::Entry,
    execute: Execute,
}
type Execute = fn(Value, &'static str, &mut dyn FnMut() -> bool) -> Result<Value, OperationError>;

const fn declare(id: &'static str, mut row: Row) -> Row {
    row.contract.descriptor.id = id;
    row
}

macro_rules! profile {
    ($id:literal, $category:literal, $cmd:literal, $slug:literal, $description:literal,
     $request:ty => $report:ty, $run:expr,
     ($input:expr, $output:expr, $items:expr, $work:expr, $memory:expr),
     $sample:literal => $answer:literal) => {
        Row {
            contract: c::Entry {
                operation_version: 1,
                descriptor: o::Descriptor {
                    id: "",
                    category: $category,
                    description: $description,
                    input_type: "json",
                    output_type: "json",
                    parameters: &[],
                    deterministic: true,
                    reversible: Some(false),
                    command: concat!($cmd, " analysis ", $slug),
                    example: concat!("cube ", $cmd, " analysis ", $slug, " --text '", $sample, "'"),
                },
                inputs: &[c::Port {
                    name: "request",
                    semantic_type: concat!($id, ".request"),
                    schema: c::schema::<$request>,
                    required: true,
                }],
                parameters: c::schema::<NoParameters>,
                outputs: &[c::Port {
                    name: "report",
                    semantic_type: concat!($id, ".report"),
                    schema: c::schema::<$report>,
                    required: true,
                }],
                limits: c::Limits {
                    max_input_bytes: Some($input),
                    max_output_bytes: Some($output),
                    max_items: Some($items),
                    max_work: Some($work),
                    max_memory_bytes: Some($memory),
                },
                fixture: || Ok(c::Fixture {
                    inputs: json!({"request": serde_json::from_str::<Value>($sample).expect("fixture request")}),
                    parameters: json!({}),
                    outputs: json!({"report": serde_json::from_str::<Value>($answer).expect("fixture answer")}),
                }),
            },
            execute: |value, operation, cancelled| {
                let request: $request = serde_json::from_value(value)
                    .map_err(|_| invalid("request does not match the selected formal operation")
                        .operation(operation))?;
                let report = ($run)(&request, cancelled).map_err(|e| e.operation(operation))?;
                check_bytes(&report, $output).map_err(|e| e.operation(operation))?;
                serde_json::to_value(report)
                    .map_err(|_| invalid("formal report could not be serialized").operation(operation))
            },
        }
    };
}

const ROWS: [Row; 13] = [
    declare(
        "graph.traversal",
        profile!("graph.traversal", "graph", "graph", "traversal", "Return ordered depth-first visits, topological order or Tarjan components.",
    traversal::Request => traversal::Report, |request, _| traversal::traverse(request),
    (4 << 20, 8 << 20, 32768, 4 << 20, 32 << 20),
    r#"{"graph":{"vertices":["A","B"],"directed":true,"edges":[{"id":"ab","from":"A","to":"B"}]},"mode":"depth_first"}"#
    => r#"{"result":{"mode":"depth_first","discovery":[{"vertex":"A","parent_edge":null},{"vertex":"B","parent_edge":"ab"}],"finish":["B","A"]},"work":11}"#),
    ),
    declare(
        "graph.weighted_path",
        profile!("graph.weighted_path", "graph", "graph", "weighted-path", "Find one exact weighted path with Dijkstra, Bellman–Ford or A*.",
    weighted::Request => weighted::Report, |request, _| weighted::shortest(request),
    (4 << 20, 8 << 20, 32768, 16 << 20, 32 << 20),
    r#"{"graph":{"vertices":["A","B"],"directed":true,"edges":[{"id":"ab","from":"A","to":"B","weight":7}]},"start":"A","goal":"B","algorithm":{"method":"dijkstra"}}"#
    => r#"{"result":{"status":"finite","cost":"7","path":{"vertices":["A","B"],"edges":["ab"]}},"negative_cycle":null,"work":9}"#),
    ),
    declare(
        "graph.all_pairs_paths",
        profile!("graph.all_pairs_paths", "graph", "graph", "all-pairs-paths", "Compute exact Floyd–Warshall distances and identify unbounded pairs.",
    weighted::AllPairsRequest => weighted::AllPairsReport, |request, _| weighted::all_pairs(request),
    (4 << 20, 8 << 20, 256, 16 << 20, 32 << 20),
    r#"{"graph":{"vertices":["A","B"],"directed":true,"edges":[{"id":"ab","from":"A","to":"B","weight":-3}]},"paths":[{"start":"A","goal":"B"}]}"#
    => r#"{"vertices":["A","B"],"distances":[["0","-3"],[null,"0"]],"unbounded":[[false,false],[false,false]],"paths":[{"status":"finite","cost":"-3","path":{"vertices":["A","B"],"edges":["ab"]}}],"work":6}"#),
    ),
    declare(
        "graph.disjoint_sets",
        profile!("graph.disjoint_sets", "graph", "graph", "disjoint-sets", "Apply an ordered union and query batch with canonical representatives.",
    disjoint::Request => disjoint::Report, |request, _| disjoint::disjoint_sets(request),
    (8 << 20, 8 << 20, 1 << 20, 4 << 20, 16 << 20),
    r#"{"ids":[20,10],"actions":[{"action":"union","left":10,"right":20},{"action":"find","id":10}]}"#
    => r#"{"components":[[20,10]],"answers":[{"action":"find","action_index":1,"representative":20}],"work":11}"#),
    ),
    declare(
        "graph.bipartite_matching",
        profile!("graph.bipartite_matching", "graph", "graph", "bipartite-matching", "Return a Hopcroft–Karp maximum matching and an optional minimum cover.",
    matching::Request => matching::Report, |request, _| matching::maximum(request),
    (4 << 20, 8 << 20, 32768, 16 << 20, 32 << 20),
    r#"{"left":["A"],"right":["B"],"edges":[{"id":"ab","from":"A","to":"B"}],"minimum_cover":true}"#
    => r#"{"matches":[{"left":"A","right":"B","edge":"ab"}],"unmatched_left":[],"unmatched_right":[],"duplicates":[],"minimum_cover":{"left":["A"],"right":[]},"work":14}"#),
    ),
    declare(
        "logic.exact_cover",
        profile!("logic.exact_cover", "logic", "logic", "exact-cover", "Enumerate a bounded exact cover prefix with Algorithm X and dancing links.",
    exact_cover::Request => exact_cover::Report, exact_cover::solve_with_cancel,
    (8 << 20, 8 << 20, 65536, 1 << 20, 32 << 20),
    r#"{"columns":[],"rows":[],"max_solutions":1}"#
    => r#"{"solutions":[[]],"complete":true,"stop_reason":"complete","work":1}"#),
    ),
    declare(
        "logic.dpll",
        profile!("logic.dpll", "logic", "logic", "dpll", "Solve explicit CNF with deterministic false-first DPLL and bounded unit propagation.",
    sat::Request => sat::Report, sat::dpll_with_cancel,
    (1 << 20, 8 << 20, 65536, 1 << 20, 32 << 20),
    r#"{"formula":{"variables":[7],"clauses":[[7]]}}"#
    => r#"{"result":{"status":"sat","assignment":[{"variable":7,"value":true}]},"work":9}"#),
    ),
    declare(
        "logic.cdcl",
        profile!("logic.cdcl", "logic", "logic", "cdcl", "Solve bounded CNF using first-UIP learning or verify an explicit RUP certificate.",
    sat::cdcl::Request => sat::cdcl::Report, sat::cdcl::run_with_cancel,
    (2 << 20, 8 << 20, 65536, 1 << 20, 64 << 20),
    r#"{"action":"solve","formula":{"variables":[],"clauses":[[]]},"proof":true}"#
    => r#"{"profile":"cube_first_uip_rup_v1","result":{"status":"unsat","proof":[[]]},"conflicts":1,"learned_literals":0,"work":4}"#),
    ),
    declare(
        "esolang.lambda",
        profile!("esolang.lambda", "esolang", "esolang", "lambda", "Perform bounded full beta and optional eta reduction on explicit lambda terms.",
    terms::LambdaRequest => terms::Report, terms::lambda_with_cancel,
    (1 << 20, 8 << 20, 4096, 1 << 20, 32 << 20),
    r#"{"term":"(\\x.x) y","trace":1}"#
    => r#"{"term":"y","normal_form":true,"steps":1,"stop_reason":"normal_form","trace":[{"step":1,"rule":"beta","redex_preorder":0}],"omitted_steps":0,"work":5}"#),
    ),
    declare(
        "esolang.ski",
        profile!("esolang.ski", "esolang", "esolang", "ski", "Reduce explicit SKI combinator terms with bounded leftmost-outermost rewrites.",
    terms::SkiRequest => terms::Report, terms::ski_with_cancel,
    (1 << 20, 8 << 20, 4096, 1 << 20, 32 << 20),
    r#"{"term":"I K"}"#
    => r#"{"term":"K","normal_form":true,"steps":1,"stop_reason":"normal_form","trace":[],"omitted_steps":1,"work":4}"#),
    ),
    declare(
        "esolang.cellular_automaton",
        profile!("esolang.cellular_automaton", "esolang", "esolang", "cellular-automaton", "Evolve a finite Life grid or an elementary rule with explicit boundaries.",
    cellular::Request => cellular::Report, |request, _| cellular::evolve(request),
    (2 << 20, 8 << 20, 1 << 20, 16 << 20, 16 << 20),
    r#"{"initial":{"kind":"elementary","rule":90,"cells":"00100"},"generations":1,"snapshots":[0]}"#
    => r#"{"rule":{"kind":"elementary","rule":90},"rows":1,"cols":5,"boundary":"finite_dead","final_grid":{"generation":1,"cells":"01010"},"snapshots":[{"generation":0,"cells":"00100"}],"work":5}"#),
    ),
    declare(
        "number.lfsr",
        profile!("number.lfsr", "numbers", "number", "lfsr", "Emit a bounded Fibonacci right-shift LFSR bit stream with explicit packing.",
    streams::LfsrRequest => streams::LfsrReport, |request, _| streams::lfsr(request),
    (4096, 4 << 20, 1 << 20, 16 << 20, 16 << 20),
    r#"{"width":3,"state":1,"tap_mask":3,"count":8,"packing":"msb_first","tail":"reject"}"#
    => r#"{"hex":"97","valid_bits":8,"final_state":4,"packing":"msb_first","work":8}"#),
    ),
    declare(
        "number.prng",
        profile!("number.prng", "numbers", "number", "prng", "Generate reproducible MT19937ar-2002 or xorshift32 words and continuation state.",
    streams::PrngRequest => streams::PrngReport, |request, _| streams::prng(request),
    (1 << 20, 1 << 20, 65536, 4 << 20, 16 << 20),
    r#"{"source":{"profile":"xorshift32","seed":1},"count":3}"#
    => r#"{"values":[270369,67634689,2647435461],"final_state":{"profile":"xorshift32","state":2647435461},"work":4}"#),
    ),
];

const fn contracts() -> [c::Entry; ROWS.len()] {
    let mut entries = [ROWS[0].contract; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract;
        index += 1;
    }
    entries
}

const fn descriptors() -> [o::Descriptor; ROWS.len()] {
    let mut entries = [ROWS[0].contract.descriptor; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract.descriptor;
        index += 1;
    }
    entries
}

pub const OPERATIONS: &[o::Descriptor] = &descriptors();
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &[]).with_contracts(&contracts());

/// Maximum compact input document for native structured transport.
pub const MAX_DOCUMENT: usize = 8 << 20;

/// Resolve an explicit CLI leaf in the selected command family.
pub fn command(group: &str, slug: &str) -> Option<&'static str> {
    OPERATIONS.iter().find_map(|row| {
        let (family, rest) = row.command.split_once(" analysis ")?;
        (family == group && rest == slug).then_some(row.id)
    })
}

/// Parse one bounded JSON request with duplicate-key diagnostics.
pub fn parse(document: &str, operation: &'static str) -> Result<Value, OperationError> {
    crate::pipeline::json::parse_with_limits(document, MAX_DOCUMENT, 16, 2_100_000)
        .map_err(|failure| (*failure.error).operation(operation))
}

/// Call the owning typed implementation with cooperative polling.
pub fn invoke_with_cancel(
    operation: &str,
    value: Value,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Value, OperationError> {
    let row = ROWS
        .iter()
        .find(|row| row.contract.descriptor.id == operation)
        .ok_or_else(|| invalid("unknown formal operation"))?;
    check_bytes(
        &value,
        row.contract
            .limits
            .max_input_bytes
            .map(|bytes| bytes as usize)
            .expect("bounded request"),
    )
    .map_err(|e| e.operation(row.contract.descriptor.id))?;
    (row.execute)(value, row.contract.descriptor.id, cancelled)
}

/// Execute named-port validation and canonical full-width JSON transport.
pub fn invoke_transport(operation: &str, request: Value) -> Result<Value, OperationError> {
    invoke_transport_with_cancel(operation, request, &mut || false)
}

/// Execute a named request and check cancellation within cooperative formal searches.
pub fn invoke_transport_with_cancel(
    operation: &str,
    request: Value,
    cancelled: &mut dyn FnMut() -> bool,
) -> Result<Value, OperationError> {
    use super::{
        contracts::canonical_value,
        execution::{Budget, portable},
    };
    let row = OPERATIONS
        .iter()
        .find(|row| row.id == operation)
        .ok_or_else(|| invalid("unknown formal operation"))?;
    let budget = Budget {
        input_bytes: MAX_DOCUMENT + 2,
        output_bytes: 8 << 20,
        ..Budget::default()
    };
    let mut outputs = portable::execute_formal(
        operation,
        &json!({"request": canonical_value(request)}),
        budget,
        cancelled,
    )
    .map_err(|error| error.operation(row.id))?;
    Ok(outputs["report"].take())
}
