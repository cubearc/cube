//! File command adapters. All inspection and copying logic lives in cube-core.
use clap::{Args, Subcommand};
use cube_core::forensics::{self, ByteRange, inspect, signatures};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Args)]
pub(crate) struct Selection {
    pub path: PathBuf,
    /// Absolute zero-based start byte
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
    /// Selected byte count; omitted means through EOF
    #[arg(long)]
    pub length: Option<u64>,
    /// Emit exact structured JSON instead of human-readable output
    #[arg(long)]
    pub json: bool,
}
impl Selection {
    fn range(&self) -> ByteRange {
        ByteRange {
            offset: self.offset,
            length: self.length,
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum FileCommand {
    /// Identify header signature candidates; this does not validate file contents
    Signature {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Scan for embedded signature candidates under a 64 MiB scan budget
    Embedded {
        #[command(flatten)]
        selection: Selection,
        /// Exact supported format name, such as png, zip or wave
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
    },
    /// Copy an explicit byte range, without overwriting any existing file
    Extract {
        path: PathBuf,
        #[arg(long)]
        offset: u64,
        #[arg(long)]
        length: u64,
        /// New destination file (exclusive creation)
        #[arg(long, required_unless_present = "stdout", conflicts_with = "stdout")]
        output: Option<PathBuf>,
        /// Explicit raw binary stdout; no newline or status text is added
        #[arg(long, required_unless_present = "output")]
        stdout: bool,
    },
}

fn show_signatures(
    output: &mut impl Write,
    candidates: &[signatures::Signature],
) -> std::io::Result<()> {
    for candidate in candidates {
        writeln!(
            output,
            "{:08x}  {} ({})  {}  evidence={}",
            candidate.offset,
            candidate.format,
            candidate.mime,
            candidate.rule,
            candidate.evidence_hex
        )?;
    }
    if candidates.is_empty() {
        writeln!(output, "No supported signature candidates")?;
    }
    Ok(())
}

pub(crate) fn info(
    path: PathBuf,
    json: bool,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let result = inspect::file_info(&path)?;
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
    } else {
        writeln!(output, "Size: {} bytes\nType: regular file", result.size)?;
        writeln!(
            output,
            "Modified: {:?}\nCreated: {:?}",
            result.modified, result.created
        )?;
        show_signatures(output, &result.signatures)?;
    }
    Ok(())
}

pub(crate) fn run(command: FileCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        FileCommand::Signature { path, json } => {
            let mut file = forensics::open_regular(&path)?;
            let mut header = Vec::new();
            (&mut file).take(12).read_to_end(&mut header)?;
            let result = signatures::detect(&header);
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                show_signatures(output, &result)?;
            }
        }
        FileCommand::Embedded {
            selection,
            format,
            max_results,
        } => {
            let mut file = forensics::open_regular(&selection.path)?;
            let result =
                signatures::scan(&mut file, selection.range(), format.as_deref(), max_results)?;
            if selection.json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                show_signatures(output, &result.candidates)?;
                writeln!(
                    output,
                    "Scanned: {} bytes; complete: {}",
                    result.scanned_bytes, result.complete
                )?;
            }
        }
        FileCommand::Extract {
            path,
            offset,
            length,
            output: destination,
            ..
        } => {
            if let Some(destination) = destination {
                let count = forensics::extract_range(&path, &destination, offset, length)?;
                writeln!(output, "Extracted {count} bytes")?;
            } else {
                forensics::copy_range(
                    &mut forensics::open_regular(&path)?,
                    output,
                    offset,
                    length,
                )?;
            }
        }
    }
    Ok(())
}
