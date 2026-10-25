//! CSV transport only; schema and validation belong to cube-core.
use super::{ByteOutput, Image};
use clap::Subcommand;
use cube_core::{
    errors::{ErrorCode, OperationError},
    image,
};
use std::{error::Error, io::Write, path::PathBuf};

#[derive(Subcommand)]
pub(crate) enum CsvCommand {
    /// Export exact CSV v1 samples (10000 pixels); no palette/tRNS conversion
    Export {
        path: PathBuf,
        #[command(flatten)]
        destination: ByteOutput,
    },
    /// Validate a complete CSV v1 grid and write a new PNG
    Import {
        #[command(flatten)]
        input: crate::Input,
        /// Read bounded UTF-8 CSV from a file instead of stdin
        #[arg(long = "input", conflicts_with = "text")]
        input_file: Option<PathBuf>,
        /// New PNG destination; existing files are never replaced
        #[arg(long)]
        output: PathBuf,
        /// Print structured constructed-image facts after writing
        #[arg(long)]
        json: bool,
    },
}

pub(super) fn run(command: CsvCommand, out: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        CsvCommand::Export { path, destination } => {
            let text = Image::open(&path)?.to_csv()?;
            if let Some(path) = destination.output {
                image::save_new(&path, text.as_bytes())?;
                writeln!(out, "Wrote CSV v1 to {}", path.display())?;
            } else {
                out.write_all(text.as_bytes())?;
            }
        }
        CsvCommand::Import {
            input,
            input_file,
            output,
            json,
        } => {
            let text = if let Some(path) = input_file {
                let bytes = crate::pipelinecmd::read_file(&path, image::csv::MAX_BYTES)?;
                String::from_utf8(bytes).map_err(|e| {
                    OperationError::new(ErrorCode::InvalidInput, "CSV input must be UTF-8")
                        .operation("image.csv.import")
                        .caused_by(e)
                })?
            } else {
                input.read_text_limit(image::csv::MAX_BYTES, "image.csv.import")?
            };
            let image = Image::from_csv(&text)?;
            image.save_png(&output)?;
            if json {
                writeln!(out, "{}", serde_json::to_string_pretty(&image.info())?)?;
            } else {
                writeln!(out, "Wrote PNG to {}", output.display())?;
            }
        }
    }
    Ok(())
}
