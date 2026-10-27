//! Explicit bounded source/program-input transport. No interpreter logic lives here.
use crate::Output;
use clap::{Args, Subcommand};
use cube_core::{
    encoding,
    errors::{ErrorCode, OperationError},
    esolang::brainfuck as bf,
    forensics,
};
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Strict bounded Brainfuck with byte input/output and wrapping cells
    Brainfuck {
        #[command(subcommand)]
        command: Brainfuck,
    },
}
#[derive(Subcommand)]
pub(crate) enum Brainfuck {
    /// Return exact bytes only on successful completion (hex by default)
    Run(RunArgs),
    /// Emit compact JSON evidence; an incomplete execution exits unsuccessfully
    Report(ReportArgs),
    /// Compare isolated widths in ascending order; incomplete comparisons fail
    Compare(CompareArgs),
}
#[derive(Args)]
#[group(skip)]
struct Common {
    /// Strict commands and ASCII space/tab/CR/LF only, <=1 MiB
    #[arg(
        long,
        allow_hyphen_values = true,
        required_unless_present = "code_file",
        conflicts_with = "code_file"
    )]
    code: Option<String>,
    /// Explicit regular local UTF-8 source file, <=1 MiB
    #[arg(long, required_unless_present = "code", conflicts_with = "code")]
    code_file: Option<PathBuf>,
    /// Program input as exact hex bytes, <=1 MiB (omitted input is empty)
    #[arg(long, conflicts_with = "input_file")]
    input_hex: Option<String>,
    /// Program input from a separate regular local file, <=1 MiB
    #[arg(long)]
    input_file: Option<PathBuf>,
    #[arg(long, default_value="error", value_parser=["error","zero","unchanged"])]
    eof: String,
    /// Dispatch budget, 0..1000000; completion exactly at the limit succeeds
    #[arg(long, default_value_t=bf::MAX_INSTRUCTIONS)]
    max_instructions: u64,
    #[arg(long, default_value_t=bf::MAX_OUTPUT_BYTES)]
    max_output_bytes: usize,
    /// Conservative combined allocation/serialization bound, <=8 MiB
    #[arg(long, default_value_t=bf::MAX_MEMORY_BYTES)]
    max_memory_bytes: usize,
}
#[derive(Args)]
struct ExecutionArgs {
    #[command(flatten)]
    common: Common,
    /// Wrapping unsigned cells: 8, 16 or 32; I/O remains single-byte
    #[arg(long, default_value_t = 8)]
    cell_bits: u8,
}
#[derive(Args)]
pub(crate) struct RunArgs {
    #[command(flatten)]
    execution: ExecutionArgs,
    #[arg(long, value_enum, default_value_t=Output::Hex)]
    output: Output,
}
#[derive(Args)]
pub(crate) struct ReportArgs {
    #[command(flatten)]
    execution: ExecutionArgs,
    #[arg(long, default_value_t = 0)]
    tape_start: usize,
    #[arg(long, default_value_t = 0)]
    tape_len: usize,
    /// Post-dispatch records; zero disables collection, maximum 4096
    #[arg(long, default_value_t = 0)]
    max_trace_records: usize,
    /// Explicit byte cap required with positive trace records, <=1 MiB
    #[arg(long, default_value_t = 0)]
    max_trace_bytes: usize,
    #[arg(long, default_value_t=bf::MAX_REPORT_BYTES)]
    max_report_bytes: usize,
}
#[derive(Args)]
pub(crate) struct CompareArgs {
    #[command(flatten)]
    common: Common,
    /// Unique subset of 8,16,32; executed in ascending order
    #[arg(long, value_delimiter = ',', num_args = 1, required = true)]
    widths: Vec<u8>,
    #[arg(long, default_value_t=bf::MAX_INSTRUCTIONS)]
    total_instructions: u64,
    #[arg(long, default_value_t=bf::MAX_OUTPUT_BYTES)]
    total_output_bytes: usize,
    #[arg(long, default_value_t=bf::MAX_MEMORY_BYTES)]
    total_memory_bytes: usize,
}
fn read(path: &Path, cap: usize) -> Result<Vec<u8>, OperationError> {
    let mut bytes = Vec::new();
    forensics::open_regular(path)?
        .take(cap as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(OperationError::io)?;
    if bytes.len() > cap {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "Brainfuck file exceeds 1 MiB")
                .operation("esolang.brainfuck.execute"),
        );
    }
    Ok(bytes)
}
struct Loaded {
    source: String,
    input: Vec<u8>,
    eof: bf::EofPolicy,
    limits: bf::Limits,
}
impl Common {
    fn load(self) -> Result<Loaded, Box<dyn Error>> {
        let code = if let Some(code) = self.code {
            code
        } else {
            String::from_utf8(read(
                &self.code_file.ok_or("explicit source required")?,
                bf::MAX_SOURCE_BYTES,
            )?)
            .map_err(|e| {
                OperationError::new(ErrorCode::InvalidInput, "source must be UTF-8")
                    .operation("esolang.brainfuck.execute")
                    .parameter("source")
                    .offset(e.utf8_error().valid_up_to() as u64)
            })?
        };
        if code.len() > bf::MAX_SOURCE_BYTES {
            return Err(
                OperationError::new(ErrorCode::ResourceLimit, "source exceeds 1 MiB")
                    .operation("esolang.brainfuck.execute")
                    .parameter("source")
                    .into(),
            );
        }
        let input = if let Some(hex) = self.input_hex {
            if hex.len() > bf::MAX_INPUT_BYTES * 2 {
                return Err(OperationError::new(
                    ErrorCode::ResourceLimit,
                    "program input exceeds 1 MiB",
                )
                .operation("esolang.brainfuck.execute")
                .parameter("input")
                .into());
            }
            encoding::hex_decode(&hex)?
        } else if let Some(path) = self.input_file {
            read(&path, bf::MAX_INPUT_BYTES)?
        } else {
            Vec::new()
        };
        Ok(Loaded {
            source: code,
            input,
            eof: bf::EofPolicy::parse(&self.eof)?,
            limits: bf::Limits {
                max_instructions: self.max_instructions,
                max_output_bytes: self.max_output_bytes,
                max_memory_bytes: self.max_memory_bytes,
            },
        })
    }
}
pub(crate) fn run(command: Command, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Brainfuck { command } = command;
    match command {
        Brainfuck::Run(args) => {
            let Loaded {
                source,
                input,
                eof,
                limits,
            } = args.execution.common.load()?;
            let bytes = bf::Program::parse(&source)?.execute(
                &input,
                bf::Options {
                    cell_bits: args.execution.cell_bits,
                    eof,
                },
                limits,
            )?;
            crate::write_bytes(output, &bytes, args.output)?;
        }
        Brainfuck::Report(args) => {
            let Loaded {
                source,
                input,
                eof,
                limits,
            } = args.execution.common.load()?;
            let report = bf::Program::parse(&source)?.report(
                &input,
                bf::Options {
                    cell_bits: args.execution.cell_bits,
                    eof,
                },
                limits,
                bf::Diagnostics {
                    tape_start: args.tape_start,
                    tape_len: args.tape_len,
                    max_trace_records: args.max_trace_records,
                    max_trace_bytes: args.max_trace_bytes,
                    max_report_bytes: args.max_report_bytes,
                },
            )?;
            serde_json::to_writer(&mut *output, &report)?;
            writeln!(output)?;
            if let Some(error) = report.error {
                output.flush()?;
                return Err(error.into());
            }
        }
        Brainfuck::Compare(args) => {
            let Loaded {
                source,
                input,
                eof,
                limits,
            } = args.common.load()?;
            let report = bf::compare(
                &source,
                &input,
                &args.widths,
                eof,
                limits,
                bf::ComparisonLimits {
                    max_instructions: args.total_instructions,
                    max_output_bytes: args.total_output_bytes,
                    max_memory_bytes: args.total_memory_bytes,
                },
            )?;
            serde_json::to_writer(&mut *output, &report)?;
            writeln!(output)?;
            if let Some(error) = report.runs.into_iter().find_map(|r| r.error) {
                output.flush()?;
                return Err(error.into());
            }
        }
    }
    Ok(())
}
