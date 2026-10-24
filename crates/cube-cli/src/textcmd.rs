//! Adaptation of explicit Unicode text operations. Algorithms live in Rust core.
use crate::Input;
use clap::{Args, Subcommand};
use cube_core::text;
use std::{error::Error, io::Write};
#[derive(Args)]
#[group(required=true,multiple=false,args=["indices","stride"])]
pub(crate) struct ExtractArgs {
    #[command(flatten)]
    input: Input,
    /// Ordered zero-based Unicode scalar indexes; duplicates preserved; empty string means empty list
    #[arg(long,conflicts_with_all=["start","stride","stop"])]
    indices: Option<String>,
    /// Stride start, default zero (requires --stride)
    #[arg(long, requires = "stride")]
    start: Option<usize>,
    /// Positive stride; --start 0/1 --stride 2 extracts alternating characters
    #[arg(long)]
    stride: Option<usize>,
    /// Exclusive stop, default scalar length (requires --stride)
    #[arg(long, requires = "stride")]
    stop: Option<usize>,
}
#[derive(Subcommand)]
pub(crate) enum TextCommand {
    /// Reverse scalar values (not graphemes or UTF-8 bytes), including CRLF/combining marks; <=1 MiB
    Reverse(Input),
    /// Explicit Unicode NFC/NFD/NFKC/NFKD; <=1 MiB input, <=8 MiB output
    Normalize {
        #[command(flatten)]
        input: Input,
        #[arg(long)]
        form: String,
    },
    /// Select scalar positions (not UTF-8 bytes); no normalization, <=1 MiB
    Extract(ExtractArgs),
}
pub(crate) fn run(command: TextCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let result = match command {
        TextCommand::Reverse(input) => text::reverse(&input.read_text("text.reverse")?)?,
        TextCommand::Normalize { input, form } => {
            text::normalize(&input.read_text("text.normalize")?, &form)?
        }
        TextCommand::Extract(args) => {
            let indices = args
                .indices
                .map(|value| {
                    if value.is_empty() {
                        return Ok(Vec::new());
                    }
                    if value.len() > text::MAX_INDICES * 21 {
                        return Err(cube_core::errors::OperationError::new(
                            cube_core::errors::ErrorCode::ResourceLimit,
                            "index argument exceeds byte budget",
                        )
                        .operation("text.extract")
                        .parameter("indices"));
                    }
                    let mut indices = Vec::new();
                    for field in value.split(',') {
                        if indices.len() == text::MAX_INDICES {
                            return Err(cube_core::errors::OperationError::new(
                                cube_core::errors::ErrorCode::ResourceLimit,
                                "selection exceeds 65536 indexes",
                            )
                            .operation("text.extract")
                            .parameter("indices"));
                        }
                        indices.push(field.parse::<usize>().map_err(|_| {
                            cube_core::errors::OperationError::new(
                                cube_core::errors::ErrorCode::InvalidParameter,
                                "indices must be comma-separated unsigned scalar positions",
                            )
                            .operation("text.extract")
                            .parameter("indices")
                        })?);
                    }
                    Ok(indices)
                })
                .transpose()?;
            text::extract(
                &args.input.read_text("text.extract")?,
                indices.as_deref(),
                args.start,
                args.stride,
                args.stop,
            )?
        }
    };
    crate::write_text(output, &result)?;
    Ok(())
}
