//! Bounded JSON invocation and exclusive Reed–Solomon codeword output.
use clap::Args;
use cube_core::{
    errors::{ErrorCode, OperationError},
    operations::{
        bounded_formats,
        execution::{self, Budget},
        representation,
    },
};
use std::{error::Error, io::Write, path::PathBuf};
use zeroize::Zeroizing;

#[derive(Args)]
pub(crate) struct FormatArgs {
    /// Binary format or fingerprint profile command
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(
        representation::OPERATIONS.iter().chain(bounded_formats::OPERATIONS)
            .filter_map(|op| op.command.strip_prefix("binary format "))
    ))]
    format: String,
    #[command(flatten)]
    input: crate::Input,
    /// Exclusive new destination for a single Reed–Solomon codeword as raw bytes
    #[arg(long)]
    codeword_file: Option<PathBuf>,
}

pub(crate) fn run(args: FormatArgs, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (id, max_document) = if let Some(id) = representation::command(&args.format) {
        (id, representation::MAX_DOCUMENT)
    } else {
        (
            bounded_formats::command(&args.format).ok_or("unknown binary format")?,
            bounded_formats::MAX_DOCUMENT,
        )
    };
    let data = Zeroizing::new(args.input.read_bounded(max_document, id)?);
    let document = std::str::from_utf8(&data)?;
    let request = cube_core::pipeline::json::parse_with_limits(document, max_document, 16, 400_000)
        .map_err(|failure| (*failure.error).operation(id))?;
    let invalid = || {
        OperationError::new(
            ErrorCode::InvalidInput,
            "expected named inputs and parameters",
        )
        .operation(id)
    };
    let fields = request.as_object().ok_or_else(invalid)?;
    if fields.len() != 2 || !fields.contains_key("inputs") || !fields.contains_key("parameters") {
        return Err(invalid().into());
    }
    let result = execution::invoke(
        id,
        1,
        &request["inputs"],
        &request["parameters"],
        Budget {
            input_bytes: max_document,
            output_bytes: max_document,
            ..Budget::default()
        },
    )?;
    if let Some(path) = args.codeword_file {
        if !matches!(
            id,
            "binary.reed_solomon.encode" | "binary.reed_solomon.decode"
        ) || result["blocks"].as_array().map(Vec::len) != Some(1)
        {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "codeword-file requires one Reed–Solomon block",
            )
            .operation(id)
            .into());
        }
        let text = result["blocks"][0]["codeword"]["base64"]
            .as_str()
            .ok_or_else(invalid)?;
        let codeword = cube_core::encoding::base64_decode(text)?;
        super::pipelinecmd::write_new(&path, &codeword)?;
    }
    serde_json::to_writer(&mut *output, &result)?;
    writeln!(output)?;
    Ok(())
}
