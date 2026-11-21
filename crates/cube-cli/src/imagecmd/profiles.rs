//! Typed image profile requests and exclusive PNG export.
use cube_core::{
    errors::{ErrorCode, OperationError},
    image::advanced_catalog as profiles,
    operations::execution::{self, Budget},
};
use serde_json::json;
use std::{error::Error, io::Write, path::PathBuf};

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(
        profiles::OPERATIONS.iter().filter_map(|item| item.command.strip_prefix("image profile "))
    ))]
    profile: String,
    #[command(flatten)]
    input: crate::Input,
    /// Write one selected raster output to a new PNG file
    #[arg(long)]
    output: Option<PathBuf>,
    /// Image port to export when writing a PNG
    #[arg(long, requires = "output")]
    port: Option<String>,
}

pub(super) fn run(args: Args, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let id = profiles::command(&args.profile).ok_or("unknown image profile")?;
    let raw = args.input.read_bounded(profiles::MAX_DOCUMENT, id)?;
    let invalid = |message| OperationError::new(ErrorCode::InvalidInput, message).operation(id);
    let request = cube_core::pipeline::json::parse_with_limits(
        std::str::from_utf8(&raw)?,
        profiles::MAX_DOCUMENT,
        20,
        2_500_000,
    )
    .map_err(|failure| (*failure.error).operation(id))?;
    let fields = request
        .as_object()
        .ok_or_else(|| invalid("expected a named request object"))?;
    if fields.len() != 2 || !fields.contains_key("inputs") || !fields.contains_key("parameters") {
        return Err(invalid("request needs exactly inputs and parameters").into());
    }
    let budget = Budget {
        input_bytes: profiles::MAX_DOCUMENT,
        output_bytes: profiles::MAX_DOCUMENT,
        ..Budget::default()
    };
    let result = execution::invoke(id, 1, &request["inputs"], &request["parameters"], budget)?;
    if let Some(path) = args.output {
        let port = args.port.as_deref().unwrap_or("output");
        let raster = result
            .get(port)
            .ok_or_else(|| invalid("requested raster port is absent"))?;
        let encoded = execution::invoke(
            "image.encode",
            1,
            &json!({"input":raster}),
            &json!({}),
            budget,
        )?;
        let b64 = encoded["output"]["base64"]
            .as_str()
            .ok_or_else(|| invalid("PNG encoder returned no byte output"))?;
        super::super::pipelinecmd::write_new(&path, &cube_core::encoding::base64_decode(b64)?)?;
    }
    serde_json::to_writer(&mut *output, &result)?;
    writeln!(output)?;
    Ok(())
}
