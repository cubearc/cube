//! Strict Brainfuck: eight commands and ASCII space/tab/CR/LF only. All brackets
//! are validated before dispatch, including unreachable code. Cells wrap; the
//! fixed tape never wraps or grows. Wider cells retain byte input/low-byte output.
pub(crate) mod execution;
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
mod comparison;
mod contracts;
pub(crate) use contracts::CONTRACTS;
mod metadata;
mod types;
pub use comparison::{Comparison, ComparisonLimits, compare};
pub use metadata::OPERATIONS;
use std::mem::size_of;
pub use types::*;

pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_COMMANDS: usize = 65_536;
pub const MAX_DEPTH: usize = 256;
pub const TAPE_CELLS: usize = 30_000;
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
pub const MAX_INSTRUCTIONS: u64 = 1_000_000;
pub const MAX_MEMORY_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TRACE_RECORDS: usize = 4096;
pub const MAX_TRACE_BYTES: usize = 1024 * 1024;
pub const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;
const OP: &str = "esolang.brainfuck.execute";

// A portable conservative ledger, rather than target-dependent allocator sizes.
// Charges cover the supported 32/64-bit layouts and keep budgets/reports stable.
const INSTRUCTION_BYTES: usize = 12;
const INDEX_BYTES: usize = 8;
const TRACE_RECORD_BYTES: usize = 48;
const CELL_BYTES: usize = 16;
const _: () = assert!(size_of::<Instruction>() <= INSTRUCTION_BYTES);
const _: () = assert!(size_of::<usize>() <= INDEX_BYTES);
const _: () = assert!(size_of::<TraceRecord>() <= TRACE_RECORD_BYTES);
const _: () = assert!(size_of::<Cell>() <= CELL_BYTES);

fn error(code: ErrorCode, message: &str) -> OperationError {
    OperationError::new(code, message).operation(OP)
}
fn invalid(name: &'static str, message: &str) -> OperationError {
    error(ErrorCode::InvalidParameter, message).parameter(name)
}
fn resource(name: &'static str, message: &str) -> OperationError {
    error(ErrorCode::ResourceLimit, message).parameter(name)
}
#[derive(Clone, Copy, Debug)]
struct Instruction {
    command: u8,
    offset: u32,
    jump: u32,
}
/// Parsed strict source, with validated jumps and original UTF-8 byte offsets.
/// It owns no source/input and can be reused across independent executions.
#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
    source_bytes: usize,
}
impl Program {
    pub fn parse(source: &str) -> Result<Self, OperationError> {
        if source.len() > MAX_SOURCE_BYTES {
            return Err(resource("source", "source exceeds 1 MiB"));
        }
        let mut instructions: Vec<Instruction> = Vec::with_capacity(source.len().min(MAX_COMMANDS));
        let mut stack = [0usize; MAX_DEPTH];
        let mut depth = 0;
        for (offset, command) in source.bytes().enumerate() {
            if matches!(command, b' ' | b'\t' | b'\r' | b'\n') {
                continue;
            }
            if !b"><+-.,[]".contains(&command) {
                return Err(error(
                    ErrorCode::InvalidInput,
                    "unsupported source character; only commands and ASCII whitespace are allowed",
                )
                .parameter("source")
                .offset(offset as u64));
            }
            if instructions.len() == MAX_COMMANDS {
                return Err(
                    resource("source", "program exceeds 65536 commands").offset(offset as u64)
                );
            }
            let index = instructions.len();
            if command == b'[' {
                if depth == MAX_DEPTH {
                    return Err(
                        resource("source", "bracket depth exceeds 256").offset(offset as u64)
                    );
                }
                stack[depth] = index;
                depth += 1;
            }
            let mut jump = 0;
            if command == b']' {
                if depth == 0 {
                    return Err(error(ErrorCode::InvalidInput, "unmatched closing bracket")
                        .parameter("source")
                        .offset(offset as u64));
                }
                depth -= 1;
                jump = stack[depth] as u32;
                instructions[stack[depth]].jump = index as u32;
            }
            instructions.push(Instruction {
                command,
                offset: offset as u32,
                jump,
            });
        }
        if depth != 0 {
            return Err(error(ErrorCode::InvalidInput, "unmatched opening bracket")
                .parameter("source")
                .offset(u64::from(instructions[stack[depth - 1]].offset)));
        }
        Ok(Self {
            instructions,
            source_bytes: source.len(),
        })
    }
    /// Return bytes only on successful completion; failures never expose partial
    /// output as success. Use [`Self::report`] for explicitly incomplete evidence.
    pub fn execute(
        &self,
        input: &[u8],
        options: Options,
        limits: Limits,
    ) -> Result<Vec<u8>, OperationError> {
        let mut report = self.report(input, options, limits, Diagnostics::default())?;
        if let Some(error) = report.error.take() {
            return Err(error);
        }
        Ok(report.output)
    }
    fn preflight(
        &self,
        input: &[u8],
        options: Options,
        limits: Limits,
        diagnostics: Diagnostics,
    ) -> Result<(usize, usize, usize), OperationError> {
        if ![8, 16, 32].contains(&options.cell_bits) {
            return Err(invalid("cell_bits", "cell_bits must be 8, 16 or 32"));
        }
        if limits.max_instructions > MAX_INSTRUCTIONS
            || limits.max_output_bytes > MAX_OUTPUT_BYTES
            || limits.max_memory_bytes > MAX_MEMORY_BYTES
        {
            return Err(invalid(
                "limits",
                "limits exceed hard instruction/output/memory ceilings",
            ));
        }
        if input.len() > MAX_INPUT_BYTES {
            return Err(resource("input", "input exceeds 1 MiB"));
        }
        let end = diagnostics
            .tape_start
            .checked_add(diagnostics.tape_len)
            .ok_or_else(|| invalid("tape_len", "tape range overflows"))?;
        if end > TAPE_CELLS {
            return Err(invalid("tape_len", "tape range exceeds 30000 cells"));
        }
        if diagnostics.max_trace_records > MAX_TRACE_RECORDS
            || diagnostics.max_trace_bytes > MAX_TRACE_BYTES
            || diagnostics.max_report_bytes > MAX_REPORT_BYTES
        {
            return Err(invalid(
                "diagnostics",
                "trace or report limits exceed hard ceilings",
            ));
        }
        if (diagnostics.max_trace_records == 0) != (diagnostics.max_trace_bytes == 0) {
            return Err(invalid(
                "max_trace_bytes",
                "trace records and bytes must both be zero or both positive",
            ));
        }
        let trace_capacity = diagnostics
            .max_trace_records
            .min(diagnostics.max_trace_bytes / TRACE_RECORD_BYTES);
        let report_bound = 4096usize
            .checked_add(
                limits
                    .max_output_bytes
                    .checked_mul(4)
                    .ok_or_else(|| resource("limits", "output size overflows"))?,
            )
            .and_then(|n| n.checked_add(diagnostics.tape_len * 48))
            .and_then(|n| n.checked_add(trace_capacity * 320))
            .ok_or_else(|| resource("diagnostics", "report size overflows"))?;
        if report_bound > diagnostics.max_report_bytes {
            return Err(resource(
                "max_report_bytes",
                "conservative compact JSON report bound exceeds budget",
            ));
        }
        let memory = self
            .source_bytes
            .checked_add(input.len())
            .and_then(|n| n.checked_add(self.instructions.capacity() * INSTRUCTION_BYTES))
            .and_then(|n| n.checked_add(MAX_DEPTH * INDEX_BYTES + TAPE_CELLS * size_of::<u32>()))
            .and_then(|n| {
                n.checked_add(
                    limits.max_output_bytes
                        + trace_capacity * TRACE_RECORD_BYTES
                        + diagnostics.tape_len * CELL_BYTES,
                )
            })
            .and_then(|n| n.checked_add(report_bound))
            .ok_or_else(|| resource("limits", "memory size overflows"))?;
        if memory > limits.max_memory_bytes {
            return Err(resource(
                "max_memory_bytes",
                "conservative total memory bound exceeds budget",
            ));
        }
        Ok((trace_capacity, memory, report_bound))
    }
    /// Execute one isolated tape; report stop reason, effective limits and exact
    /// instruction count. `[` zero skips after `]`; `]` nonzero jumps after `[`. A
    /// command is charged before its effect, including failing pointer/I/O effects.
    /// Completion exactly at the budget succeeds; the next dispatch never occurs.
    pub fn report(
        &self,
        input: &[u8],
        options: Options,
        limits: Limits,
        diagnostics: Diagnostics,
    ) -> Result<Report, OperationError> {
        let (trace_capacity, memory, report_bound) =
            self.preflight(input, options, limits, diagnostics)?;
        let mut tape = vec![0u32; TAPE_CELLS];
        let mask = u32::MAX >> (32 - options.cell_bits);
        let mut pointer = 0;
        let mut pc = 0;
        let mut instructions = 0;
        let mut consumed = 0;
        let mut output = Vec::with_capacity(limits.max_output_bytes);
        let mut trace = Vec::with_capacity(trace_capacity);
        let mut trace_complete = true;
        let mut reason = StopReason::Completed;
        let mut failure = None;
        while pc < self.instructions.len() {
            let instruction = self.instructions[pc];
            if instructions == limits.max_instructions {
                reason = StopReason::InstructionLimit;
                failure = Some(
                    resource("max_instructions", "instruction budget exhausted")
                        .offset(u64::from(instruction.offset)),
                );
                break;
            }
            instructions += 1;
            let before_pointer = pointer;
            let before_cell = tape[pointer];
            let mut emitted = None;
            let mut read = false;
            let mut next = pc + 1;
            match instruction.command {
                b'>' if pointer + 1 == TAPE_CELLS => reason = StopReason::PointerOutOfBounds,
                b'<' if pointer == 0 => reason = StopReason::PointerOutOfBounds,
                b'>' => pointer += 1,
                b'<' => pointer -= 1,
                b'+' => tape[pointer] = tape[pointer].wrapping_add(1) & mask,
                b'-' => tape[pointer] = tape[pointer].wrapping_sub(1) & mask,
                b'.' if output.len() == limits.max_output_bytes => reason = StopReason::OutputLimit,
                b'.' => {
                    let byte = tape[pointer] as u8;
                    output.push(byte);
                    emitted = Some(byte);
                }
                b',' if consumed < input.len() => {
                    tape[pointer] = u32::from(input[consumed]);
                    consumed += 1;
                    read = true;
                }
                b',' => match options.eof {
                    EofPolicy::Error => reason = StopReason::InputExhausted,
                    EofPolicy::Zero => tape[pointer] = 0,
                    EofPolicy::Unchanged => (),
                },
                b'[' if tape[pointer] == 0 => next = instruction.jump as usize + 1,
                b']' if tape[pointer] != 0 => next = instruction.jump as usize + 1,
                _ => (),
            }
            if diagnostics.max_trace_records != 0 {
                if trace.len() < trace_capacity {
                    trace.push(TraceRecord {
                        instruction: instructions,
                        source_offset: instruction.offset,
                        command: instruction.command as char,
                        pointer_before: before_pointer,
                        pointer_after: pointer,
                        cell_before: before_cell,
                        cell_after: tape[pointer],
                        output_byte: emitted,
                        input_consumed: read,
                    });
                } else {
                    trace_complete = false;
                }
            }
            if reason != StopReason::Completed {
                failure = Some(
                    match reason {
                        StopReason::OutputLimit => {
                            resource("max_output_bytes", "output budget exhausted")
                        }
                        StopReason::InputExhausted => {
                            error(ErrorCode::InvalidInput, "program input exhausted")
                                .parameter("input")
                        }
                        _ => error(
                            ErrorCode::InvalidInput,
                            "pointer movement exceeds fixed tape",
                        )
                        .parameter("pointer"),
                    }
                    .offset(u64::from(instruction.offset)),
                );
                break;
            }
            pc = next;
        }
        let snapshot = tape[diagnostics.tape_start..diagnostics.tape_start + diagnostics.tape_len]
            .iter()
            .enumerate()
            .map(|(i, &value)| Cell {
                index: diagnostics.tape_start + i,
                value,
            })
            .collect();
        Ok(Report {
            completed: reason == StopReason::Completed,
            budget_scope: matches!(
                reason,
                StopReason::InstructionLimit | StopReason::OutputLimit
            )
            .then_some(BudgetScope::PerRun),
            overflow_policy: "wrapping",
            tape_cells: TAPE_CELLS,
            stop_reason: reason,
            error: failure,
            options,
            limits,
            diagnostics,
            instructions,
            input_consumed: consumed,
            final_pointer: pointer,
            tape: snapshot,
            output: output.into_boxed_slice().into_vec(),
            trace,
            trace_complete,
            memory_bytes_upper_bound: memory,
            report_bytes_upper_bound: report_bound,
        })
    }
}
/// Parse once and execute with default 8-bit cells and EOF errors.
///
/// ```
/// use cube_core::esolang::brainfuck::{execute, Limits};
/// assert_eq!(execute("++++++++[>++++++++<-]>+.", &[], Limits::default())?, b"A");
/// # Ok::<(), cube_core::errors::OperationError>(())
/// ```
pub fn execute(source: &str, input: &[u8], limits: Limits) -> Result<Vec<u8>, OperationError> {
    Program::parse(source)?.execute(input, Options::default(), limits)
}
