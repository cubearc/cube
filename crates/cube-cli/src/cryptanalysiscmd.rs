//! Bounded JSON transport; all analysis is implemented by typed Core APIs.
use clap::Subcommand;
use cube_core::cryptanalysis as a;
use std::{error::Error, io::Write};
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Caller-owned language-model scoring
    Score(crate::Input),
    /// Explicit classical key-length evidence
    Period(crate::Input),
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Score(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.language.score")?;
            let request: a::scoring::Request = a::parse_request(&doc)?;
            serde_json::to_writer(&mut *out, &a::scoring::score(&request, None)?)?;
        }
        Command::Period(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.period.estimate")?;
            let request: a::period::Request = a::parse_request(&doc)?;
            serde_json::to_writer(&mut *out, &a::period::estimate(&request, None)?)?;
        }
    }
    writeln!(out)?;
    Ok(())
}
