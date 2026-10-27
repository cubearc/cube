//! Explicit profile and two-source transport for complete scalar inventories.
use clap::{Args, Subcommand};
use cube_core::text::{
    multiset as m,
    statistics::{CaseMode, Normalization, TextFilter},
};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
#[derive(Subcommand)]
pub(crate) enum Command {
    Compare(Input),
    Subtract(Input),
}
#[derive(Args)]
pub(crate) struct Input {
    #[command(flatten)]
    source: crate::Input,
    #[arg(
        long,
        required_unless_present = "requested_file",
        conflicts_with = "requested_file"
    )]
    requested: Option<String>,
    #[arg(
        long,
        required_unless_present = "requested",
        conflicts_with = "requested"
    )]
    requested_file: Option<PathBuf>,
    #[arg(long, default_value = "none")]
    normalization: String,
    #[arg(long, default_value = "preserve")]
    case: String,
    #[arg(long, default_value = "all")]
    filter: String,
    #[arg(long,default_value_t=m::MAX_WORK)]
    max_work: usize,
    #[arg(long,default_value_t=m::MAX_MEMORY)]
    max_memory_bytes: usize,
    #[arg(long,default_value_t=m::MAX_REPORT)]
    max_report_bytes: usize,
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, subtract) = match command {
        Command::Compare(a) => (a, false),
        Command::Subtract(a) => (a, true),
    };
    let op = if subtract {
        "text.multiset.subtract"
    } else {
        "text.multiset.compare"
    };
    let source = args.source.read_text(op)?;
    let requested = if let Some(s) = args.requested {
        s
    } else {
        let mut data = Vec::new();
        cube_core::forensics::open_regular(
            &args
                .requested_file
                .ok_or("explicit requested text/file is required")?,
        )?
        .take(cube_core::text::MAX_INPUT as u64 + 1)
        .read_to_end(&mut data)?;
        if data.len() > cube_core::text::MAX_INPUT {
            return Err(cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::ResourceLimit,
                "requested text exceeds 1 MiB",
            )
            .operation(op)
            .parameter("requested")
            .into());
        }
        String::from_utf8(data).map_err(|_| {
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidInput,
                "requested text must be UTF-8",
            )
            .operation(op)
            .parameter("requested")
        })?
    };
    let profile = m::Profile {
        normalization: Normalization::parse(&args.normalization).map_err(|e| e.operation(op))?,
        case: CaseMode::parse(&args.case).map_err(|e| e.operation(op))?,
        filter: TextFilter::parse(&args.filter).map_err(|e| e.operation(op))?,
    };
    let limits = m::Limits {
        max_work: args.max_work,
        max_memory_bytes: args.max_memory_bytes,
        max_report_bytes: args.max_report_bytes,
    };
    let report = if subtract {
        m::subtract(&source, &requested, profile, limits)?
    } else {
        m::compare(&source, &requested, profile, limits)?
    };
    serde_json::to_writer(&mut *out, &report)?;
    writeln!(out)?;
    Ok(())
}
