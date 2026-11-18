//! Bounded JSON dispatch through the owning sequence analysis module.
use clap::Args;
use cube_core::{
    errors::{ErrorCode, OperationError},
    sequence,
};
use std::{error::Error, io::Write};

#[derive(Args)]
pub(crate) struct Command {
    /// Exact profile name from the sequence operation catalog
    operation: String,
    #[command(flatten)]
    input: crate::Input,
}

pub(crate) fn run(args: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let operation = sequence::command(&args.operation).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown sequence analysis; see cube ops list",
        )
        .operation("sequence")
        .parameter("operation")
    })?;
    let source = args
        .input
        .read_text_limit(sequence::MAX_DOCUMENT, operation)?;
    let request = sequence::parse(&source)?;
    serde_json::to_writer(&mut *out, &sequence::invoke(operation, request)?)?;
    writeln!(out)?;
    Ok(())
}
