use crate::Input;
use std::{error::Error, io::Write};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Exact numeric command from the operation catalog
    operation: String,
    #[command(flatten)]
    input: Input,
}

pub(crate) fn run(args: Args, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    use cube_core::{
        errors::{ErrorCode, OperationError},
        numbers::analysis,
    };
    let operation = analysis::command(&args.operation).ok_or_else(|| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown numeric analysis; see cube ops list",
        )
        .operation("number.analysis")
        .parameter("operation")
    })?;
    let source = args
        .input
        .read_text_limit(analysis::MAX_DOCUMENT, operation)?;
    let result = analysis::invoke_transport(operation, analysis::parse(&source)?)?;
    serde_json::to_writer(&mut *out, &result)?;
    writeln!(out)?;
    Ok(())
}
