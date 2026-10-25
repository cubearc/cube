//! Caller-selected offline files. References never become file/network actions.
use clap::Subcommand;
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Subcommand)]
pub(crate) enum WebCommand {
    /// Token/reference/text observations from inert UTF-8 HTML
    Html {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Request/response metadata from bounded HAR 1.2
    Har {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
}
pub(crate) fn run(command: WebCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        WebCommand::Html { path, json } => {
            let data = crate::pipelinecmd::read_file(&path, cube_core::web::MAX_HTML_BYTES)?;
            let text = std::str::from_utf8(&data).map_err(|_| {
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::InvalidInput,
                    "HTML must be valid UTF-8",
                )
                .operation("web.html.inspect")
            })?;
            let report = cube_core::web::html::inspect(text)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                writeln!(
                    output,
                    "{}; {} references; {} inline records; {} tokenizer errors",
                    report.scope,
                    report.references.len(),
                    report.embedded.len(),
                    report.parse_errors
                )?;
                for r in report.references {
                    writeln!(
                        output,
                        "{} {} {:?} {:?}",
                        r.tag, r.attribute, r.value, r.observations
                    )?;
                }
            }
        }
        WebCommand::Har { path, json } => {
            let data = crate::pipelinecmd::read_file(
                &path,
                cube_core::pipeline::json::MAX_DOCUMENT_BYTES,
            )?;
            let text = std::str::from_utf8(&data).map_err(|_| {
                cube_core::errors::OperationError::new(
                    cube_core::errors::ErrorCode::InvalidInput,
                    "HAR must be valid UTF-8",
                )
                .operation("web.har.inspect")
            })?;
            let report = cube_core::web::har::inspect(text)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                writeln!(output, "{}; {} entries", report.scope, report.entries.len())?;
                for r in report.entries {
                    writeln!(
                        output,
                        "{} {:?} status={} {:?}",
                        r.method, r.url, r.status, r.observations
                    )?;
                }
            }
        }
    }
    Ok(())
}
