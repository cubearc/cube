//! Bounded decimal input; all parsing and digital-root arithmetic is in core.
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: crate::Input,
    /// Only radix 10 is supported
    #[arg(long, default_value_t = 10)]
    radix: u32,
    #[arg(long)]
    json: bool,
}
pub(crate) fn run(args: Args, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let text = args.input.read_text_limit(
        cube_core::numbers::digital::MAX_INPUT_BYTES,
        "number.digital_root",
    )?;
    let value = cube_core::numbers::digital::from_decimal(&text, args.radix)?;
    if args.json {
        writeln!(output, "{}", serde_json::json!({"value":value}))?;
    } else {
        writeln!(output, "{value}")?;
    }
    Ok(())
}
