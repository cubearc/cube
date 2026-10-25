//! Explicit shared/per-word signed position transport.
use cube_core::text::word::{self, Positions};
use std::{error::Error, io::Write};
#[derive(clap::Args)]
#[group(required=true,multiple=false,args=["positions","per_word"])]
pub(crate) struct Args {
    #[command(flatten)]
    input: crate::Input,
    /// Exact JSON signed i64 list, e.g. [1,3] or [-1]
    #[arg(long, allow_hyphen_values = true)]
    positions: Option<String>,
    /// Exact JSON list for every source word, e.g. [[1],[-1]]
    #[arg(long)]
    per_word: Option<String>,
    #[arg(long, default_value_t = 1)]
    index_base: u8,
    #[arg(long, default_value = "error")]
    short_word: String,
}
pub(crate) fn run(args: Args, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let shared = args
        .positions
        .as_deref()
        .map(word::parse_positions)
        .transpose()?;
    let per_word = args
        .per_word
        .as_deref()
        .map(word::parse_per_word)
        .transpose()?;
    let positions = if let Some(shared) = &shared {
        Positions::Shared(shared)
    } else {
        Positions::PerWord(
            per_word
                .as_deref()
                .expect("clap requires one position mode"),
        )
    };
    let report = word::extract(
        &args.input.read_text("text.word.extract")?,
        positions,
        args.index_base,
        &args.short_word,
    )?;
    writeln!(output, "{}", serde_json::to_string(&report)?)?;
    Ok(())
}
