//! Explicit caller-owned local file boundaries for investigation.
use clap::Subcommand;
use cube_core::{forensics, investigation::triage, operations};
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Subcommand)]
pub(crate) enum InvestigateCommand {
    /// Suggest available inspections from up to 12 header bytes; never execute suggestions
    Triage {
        path: PathBuf,
        #[arg(long, default_value_t = 12)]
        max_read_bytes: usize,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: InvestigateCommand, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        InvestigateCommand::Triage {
            path,
            max_read_bytes,
            json,
        } => {
            let mut file = forensics::open_regular(&path)?;
            let result = triage::reader(&mut file, max_read_bytes, &operations::catalog())?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                write!(out, "{}", result.human())?;
            }
        }
    }
    Ok(())
}
