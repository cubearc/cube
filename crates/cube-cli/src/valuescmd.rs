//! Numeric text report presentation; exact values remain structured integers.
use cube_core::text::values;
use std::{error::Error, io::Write};
#[derive(clap::Args)]
pub(crate) struct Codepoints {
    #[command(flatten)]
    input: crate::Input,
    #[arg(long, default_value = "decimal")]
    format: String,
}
#[derive(clap::Args)]
pub(crate) struct WordValues {
    #[command(flatten)]
    input: crate::Input,
    /// Exact JSON [scalar,u64] pairs; omitted means Unicode code points
    #[arg(long)]
    table: Option<String>,
    #[arg(long, default_value = "error")]
    unmapped: String,
    #[arg(long)]
    sums: bool,
    #[arg(long,default_value_t=u64::MAX)]
    max_sum: u64,
}
pub(crate) fn lengths(input: crate::Input, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(
        output,
        "{}",
        serde_json::to_string(&values::word_lengths(
            &input.read_text("text.word.lengths")?
        )?)?
    )?;
    Ok(())
}
pub(crate) fn codepoints(args: Codepoints, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    writeln!(
        output,
        "{}",
        serde_json::to_string(&values::codepoints(
            &args.input.read_text("text.codepoints")?,
            &args.format
        )?)?
    )?;
    Ok(())
}
pub(crate) fn words(args: WordValues, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let table = args.table.as_deref().map(values::parse_table).transpose()?;
    let report = values::word_values(
        &args.input.read_text("text.word.values")?,
        table.as_deref(),
        &args.unmapped,
        args.sums,
        args.max_sum,
    )?;
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
