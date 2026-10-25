//! Exact custom-radix transport; arithmetic stays in core.
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long)]
    source_alphabet: String,
    #[arg(long)]
    target_alphabet: String,
    #[arg(long)]
    from_base: u32,
    #[arg(long)]
    to_base: u32,
}
pub(crate) fn run(args: Args, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let text = args.input.read_text_limit(
        cube_core::numbers::alphabet::MAX_INPUT,
        "number.alphabet.convert",
    )?;
    crate::write_text(
        output,
        &cube_core::numbers::alphabet::convert(
            &text,
            &args.source_alphabet,
            &args.target_alphabet,
            args.from_base,
            args.to_base,
        )?,
    )?;
    Ok(())
}
