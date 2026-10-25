//! Caller-selected offline files. References never become file/network actions.
use clap::{Args, Subcommand};
use std::{error::Error, io::Write, path::PathBuf};
#[derive(Args)]
#[group(required=true,multiple=false,args=["stdout","output","json"])]
pub(crate) struct AssetOutput {
    #[arg(long)]
    stdout: bool,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}
#[derive(Subcommand)]
pub(crate) enum AssetCommand {
    Html {
        path: PathBuf,
        #[arg(long)]
        index: usize,
        #[arg(long, default_value = "inline")]
        selector: String,
        #[command(flatten)]
        destination: AssetOutput,
    },
    Har {
        path: PathBuf,
        #[arg(long)]
        index: usize,
        #[command(flatten)]
        destination: AssetOutput,
    },
}
fn document(
    path: &std::path::Path,
    limit: usize,
    op: &'static str,
) -> Result<String, Box<dyn Error>> {
    let bytes = crate::pipelinecmd::read_file(path, limit)?;
    String::from_utf8(bytes).map_err(|e| {
        cube_core::errors::OperationError::new(
            cube_core::errors::ErrorCode::InvalidInput,
            "artifact must be valid UTF-8",
        )
        .operation(op)
        .caused_by(e)
        .into()
    })
}
#[derive(Subcommand)]
pub(crate) enum WebCommand {
    /// Select embedded bytes explicitly; never resolve references
    Asset {
        #[command(subcommand)]
        source: AssetCommand,
    },
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
        WebCommand::Asset { source } => {
            let (result, destination) = match source {
                AssetCommand::Html {
                    path,
                    index,
                    selector,
                    destination,
                } => {
                    let selector = cube_core::web::assets::HtmlSelector::parse(&selector)?;
                    (
                        cube_core::web::assets::html(
                            &document(&path, cube_core::web::MAX_HTML_BYTES, "web.html.asset")?,
                            index,
                            selector,
                        )?,
                        destination,
                    )
                }
                AssetCommand::Har {
                    path,
                    index,
                    destination,
                } => (
                    cube_core::web::assets::har(
                        &document(
                            &path,
                            cube_core::pipeline::json::MAX_DOCUMENT_BYTES,
                            "web.har.asset",
                        )?,
                        index,
                    )?,
                    destination,
                ),
            };
            if destination.stdout {
                output.write_all(&result.data)?;
            } else {
                let mut report = serde_json::json!(result);
                if destination.json {
                    report["data_hex"] =
                        serde_json::json!(cube_core::encoding::hex_encode(&result.data));
                }
                if let Some(path) = destination.output {
                    crate::pipelinecmd::write_new(&path, &result.data)?;
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            }
        }
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
