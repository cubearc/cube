//! Preserve report/partial-output semantics and Core's aggregate interpreter budgets.
use super::field;
use cube_core::{errors::OperationError, esolang::brainfuck as b};
use serde_json::{Value, json};
pub(super) fn supports(op: &str) -> bool {
    matches!(op, "esolang.brainfuck.report" | "esolang.brainfuck.compare")
}
pub(super) fn execute(op: &str, i: Value, p: Value) -> Result<Value, OperationError> {
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
