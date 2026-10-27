//! Concrete execution policies, limits and bounded evidence.
use super::*;
/// Effective per-run ceilings. Zero instruction/output limits are valid. The
/// memory budget includes caller source/input, parser, tape, retained evidence
/// and a conservative compact JSON serialization bound; no hidden budget growth.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Limits {
    pub max_instructions: u64,
    pub max_output_bytes: usize,
    pub max_memory_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_instructions: MAX_INSTRUCTIONS,
            max_output_bytes: MAX_OUTPUT_BYTES,
            max_memory_bytes: MAX_MEMORY_BYTES,
        }
    }
}
/// EOF behavior must be selected explicitly to depart from the default error.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EofPolicy {
    #[default]
    Error,
    Zero,
    Unchanged,
}
impl EofPolicy {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "error" => Ok(Self::Error),
            "zero" => Ok(Self::Zero),
            "unchanged" => Ok(Self::Unchanged),
            _ => Err(invalid("eof", "eof must be error, zero or unchanged")),
        }
    }
}
/// Unsigned wrapping cells; input zero-extends a byte and output projects the low
/// eight bits at every width. Nonwrapping arithmetic is intentionally unsupported.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Options {
    pub cell_bits: u8,
    pub eof: EofPolicy,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            cell_bits: 8,
            eof: EofPolicy::Error,
        }
    }
}
/// Final tape selection is an exact half-open range, never silently truncated.
/// Traces record after each attempted dispatch, including a failing dispatch.
/// When either trace budget is exhausted execution continues and trace_complete
/// becomes false. Zero records disables tracing and requires zero trace bytes.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Diagnostics {
    pub tape_start: usize,
    pub tape_len: usize,
    pub max_trace_records: usize,
    pub max_trace_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            tape_start: 0,
            tape_len: 0,
            max_trace_records: 0,
            max_trace_bytes: 0,
            max_report_bytes: MAX_REPORT_BYTES,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Completed,
    InstructionLimit,
    OutputLimit,
    InputExhausted,
    PointerOutOfBounds,
}
/// Budget responsible for an instruction/output stop; pointer and input failures
/// have no budget scope. Comparisons distinguish per-run and shared exhaustion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetScope {
    PerRun,
    Aggregate,
    Both,
}
#[derive(Debug, Serialize)]
pub struct Cell {
    pub index: usize,
    pub value: u32,
}
/// `cell_before` is at pointer_before; `cell_after` is at pointer_after. Output
/// and input effects belong to this dispatch; instruction is one-based.
#[derive(Debug, Serialize)]
pub struct TraceRecord {
    pub instruction: u64,
    pub source_offset: u32,
    pub command: char,
    pub pointer_before: usize,
    pub pointer_after: usize,
    pub cell_before: u32,
    pub cell_after: u32,
    pub output_byte: Option<u8>,
    pub input_consumed: bool,
}
/// Failed executions retain explicitly incomplete evidence. Parsing/invalid
/// options fail before execution and return an OperationError instead.
#[derive(Debug, Serialize)]
pub struct Report {
    pub completed: bool,
    pub budget_scope: Option<BudgetScope>,
    pub overflow_policy: &'static str,
    pub tape_cells: usize,
    pub stop_reason: StopReason,
    pub error: Option<OperationError>,
    pub options: Options,
    pub limits: Limits,
    pub diagnostics: Diagnostics,
    pub instructions: u64,
    pub input_consumed: usize,
    pub final_pointer: usize,
    pub tape: Vec<Cell>,
    pub output: Vec<u8>,
    pub trace: Vec<TraceRecord>,
    pub trace_complete: bool,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
