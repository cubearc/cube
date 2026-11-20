use crate::Input;
use std::{error::Error, io::Write};

#[derive(clap::Args)]
pub(crate) struct Args {
    /// Exact operation command from the operation catalog
    operation: String,
    #[command(flatten)]
    input: Input,
}

pub(crate) fn run(args: Args, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    family("number", args, out)
}

pub(crate) fn family(
    group: &'static str,
    args: Args,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    use cube_core::{
        errors::{ErrorCode, OperationError},
        numbers::analysis,
        operations::formal,
    };
    let result = if group == "number"
        && let Some(op) = analysis::command(&args.operation)
    {
        let source = args.input.read_text_limit(analysis::MAX_DOCUMENT, op)?;
        analysis::invoke_transport(op, analysis::parse(&source)?)?
    } else {
        let operation = formal::command(group, &args.operation).ok_or_else(|| {
            OperationError::new(
                ErrorCode::InvalidParameter,
                if group == "number" {
                    "unknown numeric analysis; see cube ops list"
                } else {
                    "unknown analysis; see cube ops list"
                },
            )
            .operation(if group == "number" {
                "number.analysis"
            } else {
                group
            })
            .parameter("operation")
        })?;
        let source = args
            .input
            .read_text_limit(formal::MAX_DOCUMENT, operation)?;
        formal::invoke_transport(operation, formal::parse(&source, operation)?)?
    };
    serde_json::to_writer(&mut *out, &result)?;
    writeln!(out)?;
    Ok(())
}
