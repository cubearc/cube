//! Isolated width comparison with aggregate budgets.
use super::*;
/// Comparison has independent per-run budgets plus shared hard-capped aggregate
/// dispatch, retained output and memory ceilings. Widths execute in ascending
/// order; duplicates/empty sets fail. Budget remainders only lower run budgets.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct ComparisonLimits {
    pub max_instructions: u64,
    pub max_output_bytes: usize,
    pub max_memory_bytes: usize,
}
impl Default for ComparisonLimits {
    fn default() -> Self {
        Self {
            max_instructions: MAX_INSTRUCTIONS,
            max_output_bytes: MAX_OUTPUT_BYTES,
            max_memory_bytes: MAX_MEMORY_BYTES,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Comparison {
    pub runs: Vec<Report>,
    pub per_run_limits: Limits,
    pub all_completed: bool,
    /// None when any execution is incomplete. Equal completed bytes are not a
    /// claim of semantic equivalence of the programs/cell models.
    pub equal_completed_outputs: Option<bool>,
    pub limits: ComparisonLimits,
    pub instructions: u64,
    pub output_bytes: usize,
    pub memory_bytes_upper_bound: usize,
}
/// Compare exact bytes without traces or snapshots; source/parser are shared.
/// All runs receive identical requested policies; aggregate remainders are shown
/// in each report's effective limits and may make a comparison incomplete.
pub fn compare(
    source: &str,
    input: &[u8],
    widths: &[u8],
    eof: EofPolicy,
    limits: Limits,
    aggregate: ComparisonLimits,
) -> Result<Comparison, OperationError> {
    if widths.is_empty()
        || widths.len() > 3
        || widths
            .iter()
            .enumerate()
            .any(|(i, w)| ![8, 16, 32].contains(w) || widths[..i].contains(w))
    {
        return Err(invalid(
            "widths",
            "select a nonempty unique subset of 8, 16 and 32",
        ));
    }
    if aggregate.max_instructions > MAX_INSTRUCTIONS
        || aggregate.max_output_bytes > MAX_OUTPUT_BYTES
        || aggregate.max_memory_bytes > MAX_MEMORY_BYTES
    {
        return Err(invalid(
            "aggregate",
            "aggregate limits exceed hard ceilings",
        ));
    }
    let program = Program::parse(source)?;
    // Preflight every width before any execution, including retained reports and
    // their compact JSON bound. This conservative charge allows one live tape.
    let mut sorted = widths.to_vec();
    sorted.sort_unstable();
    let mut peak = 0;
    for &cell_bits in &sorted {
        let (_, memory, _) = program.preflight(
            input,
            Options { cell_bits, eof },
            limits,
            Diagnostics::default(),
        )?;
        peak = peak.max(memory);
    }
    let retention = aggregate
        .max_output_bytes
        .checked_mul(5)
        .and_then(|n| n.checked_add(sorted.len() * 4096))
        .ok_or_else(|| resource("aggregate", "comparison retention size overflows"))?;
    let memory_bytes_upper_bound = peak
        .checked_sub(limits.max_output_bytes * 5)
        .and_then(|n| n.checked_add(retention))
        .ok_or_else(|| resource("aggregate", "comparison memory size overflows"))?;
    if memory_bytes_upper_bound > aggregate.max_memory_bytes {
        return Err(resource(
            "aggregate",
            "conservative comparison memory bound exceeds aggregate budget",
        ));
    }
    let mut runs = Vec::with_capacity(sorted.len());
    let mut instructions = 0;
    let mut output_bytes = 0;
    for cell_bits in sorted {
        let effective = Limits {
            max_instructions: limits
                .max_instructions
                .min(aggregate.max_instructions - instructions),
            max_output_bytes: limits
                .max_output_bytes
                .min(aggregate.max_output_bytes - output_bytes),
            max_memory_bytes: limits.max_memory_bytes,
        };
        let mut run = program.report(
            input,
            Options { cell_bits, eof },
            effective,
            Diagnostics::default(),
        )?;
        let (requested, remaining) = match run.stop_reason {
            StopReason::InstructionLimit => (
                limits.max_instructions,
                aggregate.max_instructions - instructions,
            ),
            StopReason::OutputLimit => (
                limits.max_output_bytes as u64,
                (aggregate.max_output_bytes - output_bytes) as u64,
            ),
            _ => (0, 0),
        };
        if run.budget_scope.is_some() {
            run.budget_scope = Some(if remaining < requested {
                BudgetScope::Aggregate
            } else if remaining == requested {
                BudgetScope::Both
            } else {
                BudgetScope::PerRun
            });
        }
        instructions += run.instructions;
        output_bytes += run.output.len();
        runs.push(run);
    }
    let all_completed = runs.iter().all(|r| r.completed);
    let equal_completed_outputs =
        all_completed.then(|| runs.iter().all(|r| r.output == runs[0].output));
    Ok(Comparison {
        runs,
        per_run_limits: limits,
        all_completed,
        equal_completed_outputs,
        limits: aggregate,
        instructions,
        output_bytes,
        memory_bytes_upper_bound,
    })
}
