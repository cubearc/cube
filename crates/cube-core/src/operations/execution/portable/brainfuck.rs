//! Preserve report/partial-output semantics and Core's aggregate interpreter budgets.
use super::field;
use crate::{errors::OperationError, esolang::brainfuck as b};
use serde_json::{Value, json};
pub(super) fn supports(op: &str) -> bool {
    matches!(op, "esolang.brainfuck.report" | "esolang.brainfuck.compare")
}
fn ceilings(budget: crate::operations::execution::Budget) -> [(&'static str, u64); 8] {
    [
        ("max_instructions", budget.instructions),
        ("total_instructions", budget.instructions),
        ("max_output_bytes", budget.memory_bytes as u64),
        ("total_output_bytes", budget.memory_bytes as u64),
        ("max_memory_bytes", budget.memory_bytes as u64),
        ("total_memory_bytes", budget.memory_bytes as u64),
        ("max_trace_bytes", budget.memory_bytes as u64),
        // Diagnostic buffers and report bounds are conservative memory reservations,
        // distinct from the size of the final named JSON output.
        // The complete canonical result is checked against output_bytes after execution.
        ("max_report_bytes", budget.memory_bytes as u64),
    ]
}
pub(super) fn defaults(
    operation: &str,
    parameters: &mut Value,
    budget: crate::operations::execution::Budget,
) {
    if !supports(operation) {
        return;
    }
    for (name, maximum) in ceilings(budget) {
        if let Some(value) = parameters.get(name).and_then(Value::as_u64) {
            parameters[name] = json!(value.min(maximum));
        }
    }
}
pub(super) fn configure(
    operation: &str,
    parameters: &Value,
    budget: crate::operations::execution::Budget,
) -> Result<(), OperationError> {
    if !supports(operation) {
        return Ok(());
    }
    for (name, maximum) in ceilings(budget) {
        if parameters
            .get(name)
            .and_then(Value::as_u64)
            .is_some_and(|value| value > maximum)
        {
            return Err(OperationError::new(
                crate::errors::ErrorCode::ResourceLimit,
                "interpreter configuration exceeds the invocation budget",
            )
            .parameter(name));
        }
    }
    Ok(())
}
pub(super) fn execute(
    op: &str,
    i: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let source: String = field(&i, "source")?;
    let input: Vec<u8> = field(&i, "input")?;
    let limits = b::Limits {
        max_instructions: field(&p, "max_instructions")?,
        max_output_bytes: field(&p, "max_output_bytes")?,
        max_memory_bytes: field(&p, "max_memory_bytes")?,
    };
    let eof = b::EofPolicy::parse(&field::<String>(&p, "eof")?)?;
    if op == "esolang.brainfuck.compare" {
        return Ok(
            json!({"output":b::compare(&source,&input,&field::<Vec<u8>>(&p,"widths")?,eof,limits,b::ComparisonLimits{
    max_instructions:field(&p,"total_instructions")?,max_output_bytes:field(&p,"total_output_bytes")?,max_memory_bytes:field(&p,"total_memory_bytes")?,})?}),
        );
    }
    Ok(
        json!({"output":b::Program::parse(&source)?.report(&input,b::Options{cell_bits:field(&p,"cell_bits")?,eof},limits,b::Diagnostics{
  tape_start:field(&p,"tape_start")?,tape_len:field(&p,"tape_len")?,max_trace_records:field(&p,"max_trace_records")?,max_trace_bytes:field(&p,"max_trace_bytes")?,max_report_bytes:field(&p,"max_report_bytes")?,})?}),
    )
}
