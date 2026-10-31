//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::field;
use cube_core::{
    errors::{ErrorCode, OperationError},
    graph,
    logic::boolean,
    probability,
};
use serde::Serialize;
use serde_json::{Value, json};

enum Adapter {
    Components,
    Shortest,
    Cycle,
    Euler,
    Evaluate,
    Analyze,
    Finite,
    DiceSum,
    DiceCompare,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "graph.components" => Adapter::Components,
        "graph.shortest_path" => Adapter::Shortest,
        "graph.cycle" => Adapter::Cycle,
        "graph.euler_trail" => Adapter::Euler,
        "logic.boolean.evaluate" => Adapter::Evaluate,
        "logic.boolean.analyze" => Adapter::Analyze,
        "probability.finite.analyze" => Adapter::Finite,
        "probability.dice.sum" => Adapter::DiceSum,
        "probability.dice.compare" => Adapter::DiceCompare,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use the exact structured Core model and parameter fields",
    )
}
fn document(value: &Value) -> Result<String, OperationError> {
    serde_json::to_string(value).map_err(|_| invalid())
}
fn output(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|value| json!({"output":value}))
        .map_err(|_| invalid())
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| match selected {
        Adapter::Components | Adapter::Shortest | Adapter::Cycle | Adapter::Euler => {
            let limits = graph::Limits {
                max_work: field(&p, "max_work")?,
                max_memory_bytes: field(&p, "max_memory_bytes")?,
                max_report_bytes: field(&p, "max_report_bytes")?,
            };
            let graph = graph::Graph::new(graph::parse(&document(&inputs["input"])?)?, limits)?;
            match selected {
                Adapter::Components => output(graph.components(limits)?),
                Adapter::Shortest => output(graph.shortest_path(
                    &field::<String>(&p, "source")?,
                    &field::<String>(&p, "target")?,
                    limits,
                )?),
                Adapter::Cycle => output(graph.cycle(limits)?),
                _ => output(
                    graph.euler_trail(
                        p.get("start")
                            .map(|_| field::<String>(&p, "start"))
                            .transpose()?
                            .as_deref(),
                        limits,
                    )?,
                ),
            }
        }
        Adapter::Evaluate | Adapter::Analyze => {
            let limits = boolean::Limits {
                max_work: field(&p, "max_work")?,
                max_rows: field(&p, "max_rows")?,
                max_memory_bytes: field(&p, "max_memory_bytes")?,
                max_report_bytes: field(&p, "max_report_bytes")?,
            };
            if matches!(selected, Adapter::Evaluate) {
                let model = boolean::parse(&document(&inputs["model"])?)?;
                let assignment = boolean::parse_assignment(&document(&inputs["assignment"])?)?;
                output(boolean::evaluate(&model, &assignment, limits)?)
            } else {
                output(boolean::analyze(
                    &boolean::parse(&document(&inputs["input"])?)?,
                    limits,
                )?)
            }
        }
        Adapter::Finite | Adapter::DiceSum | Adapter::DiceCompare => {
            let limits = probability::Limits {
                max_work: field(&p, "max_work")?,
                max_memory_bytes: field(&p, "max_memory_bytes")?,
                max_report_bytes: field(&p, "max_report_bytes")?,
            };
            let document = document(&inputs["input"])?;
            match selected {
                Adapter::Finite => output(probability::finite_analyze(
                    &probability::parse_finite(&document)?,
                    limits,
                )?),
                Adapter::DiceSum => output(probability::dice_sum(
                    &probability::parse_dice(&document)?,
                    limits,
                )?),
                _ => output(probability::dice_compare(
                    &probability::parse_pair(&document)?,
                    limits,
                )?),
            }
        }
    })();
    result.map_err(|error: OperationError| error.operation(identity))
}
