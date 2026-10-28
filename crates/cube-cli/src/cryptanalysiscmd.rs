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
    /// Finite Caesar candidate ranking
    Caesar(crate::Input),
    /// Finite Affine candidate ranking
    Affine(crate::Input),
    /// Explicit Vigenere beam search
    Vigenere(crate::Input),
    /// Explicit transposition candidates
    Transposition(crate::Input),
    /// Exact homophonic token evidence
    Homophonic(crate::Input),
    /// Explicit known-plaintext verification
    Crib(crate::Input),
}
pub(crate) fn run(command: Command, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Score(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.language.score")?;
            let request: a::scoring::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::scoring::score(&request, None)?)?,
            )?;
        }
        Command::Period(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.period.estimate")?;
            let request: a::period::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::period::estimate(&request, None)?)?,
            )?;
        }
        Command::Caesar(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.caesar.rank")?;
            let request: a::classical::Caesar = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::classical::caesar(&request, None)?)?,
            )?;
        }
        Command::Affine(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.affine.rank")?;
            let request: a::classical::Affine = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::classical::affine(&request, None)?)?,
            )?;
        }
        Command::Vigenere(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.vigenere.rank")?;
            let request: a::vigenere::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::vigenere::rank(&request, None)?)?,
            )?;
        }
        Command::Transposition(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.transposition.rank")?;
            let request: a::transposition::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::transposition::rank(&request, None)?)?,
            )?;
        }
        Command::Homophonic(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.homophonic.inspect")?;
            let request: a::homophonic::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::homophonic::inspect(&request, None)?)?,
            )?;
        }
        Command::Crib(input) => {
            let doc = input.read_text_limit(a::MAX_REQUEST, "analysis.crib.verify")?;
            let request: a::crib::Request = a::parse_request(&doc)?;
            write_report(
                &mut *out,
                serde_json::to_value(a::crib::verify(&request, None)?)?,
            )?;
        }
    }
    writeln!(out)?;
    Ok(())
}

fn write_report(out: &mut impl Write, report: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let value = cube_core::operations::contracts::canonical_value(report);
    serde_json::to_writer(out, &value)?;
    Ok(())
}
