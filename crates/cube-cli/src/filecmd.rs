//! File command adapters. All inspection and copying logic lives in cube-core.
use clap::{Args, Subcommand};
use cube_core::forensics::{self, ByteRange, inspect, ranges, signatures};
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
    /// Explicitly inspect JPEG/TIFF EXIF; this may display sensitive metadata
    Exif {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Detect trailing bytes after supported PNG/JPEG/GIF framing (no pixel decoding)
    Appended {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Inspect hexadecimal rows (first 256 bytes by default; source remains unchanged)
    Hex {
        #[command(flatten)]
        selection: Selection,
        /// Bytes per row, from 1 to 256
        #[arg(long, default_value_t = 16)]
        width: usize,
    },
    /// Extract printable ASCII runs with exact byte offsets
    Strings {
        #[command(flatten)]
        selection: Selection,
        #[arg(long, default_value_t = 4)]
        min_length: usize,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
    },
    /// Compare exact byte offsets; differing contents still exit successfully
    Compare {
        first: PathBuf,
        second: PathBuf,
        #[arg(long, default_value_t = 0)]
        offset: u64,
        #[arg(long)]
        length: Option<u64>,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
        #[arg(long)]
        json: bool,
    },
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
        let show_time = |time: Option<inspect::Timestamp>| {
            time.map(|time| time.to_string())
                .unwrap_or_else(|| "unavailable".into())
        };
        writeln!(
            output,
            "Modified: {}\nCreated: {}",
            show_time(result.modified),
            show_time(result.created)
        )?;
        show_signatures(output, &result.signatures)?;
    }
    Ok(())
}

pub(crate) fn run(command: FileCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        FileCommand::Exif { path, json } => {
            let result = forensics::exif::read(&mut forensics::open_regular(&path)?)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(output, "EXIF present: {}", result.present)?;
                for tag in result.tags {
                    writeln!(
                        output,
                        "IFD {} {} {}: {}",
                        tag.ifd, tag.context, tag.name, tag.display
                    )?;
                }
            }
        }
        FileCommand::Appended { path, json } => {
            let result = forensics::boundaries::inspect(&mut forensics::open_regular(&path)?)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "Logical end: {}\nTrailing bytes: {}\nEvidence: {}",
                    result.logical_end, result.trailing_bytes, result.evidence
                )?;
            }
        }
        FileCommand::Hex { selection, width } => {
            let mut file = forensics::open_regular(&selection.path)?;
            let length = match selection.length {
                Some(length) => length,
                None => file
                    .metadata()?
                    .len()
                    .saturating_sub(selection.offset)
                    .min(256),
            };
            let range = ByteRange {
                offset: selection.offset,
                length: Some(length),
            };
            let rows = ranges::hex_rows(&mut file, range, width)?;
            if selection.json {
                writeln!(output, "{}", serde_json::to_string_pretty(&rows)?)?;
            } else {
                for row in rows {
                    let hex = row
                        .bytes
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    writeln!(
                        output,
                        "{:08x}  {:<width$}  |{}|",
                        row.offset,
                        hex,
                        row.ascii,
                        width = width * 3 - 1
                    )?;
                }
            }
        }
        FileCommand::Strings {
            selection,
            min_length,
            max_results,
        } => {
            let mut file = forensics::open_regular(&selection.path)?;
            let result = ranges::strings(&mut file, selection.range(), min_length, max_results)?;
            if selection.json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                for run in &result.strings {
                    writeln!(output, "{:08x}  {}  {}", run.offset, run.length, run.text)?;
                }
                writeln!(
                    output,
                    "Scanned: {} bytes; complete: {}",
                    result.scanned_bytes, result.complete
                )?;
            }
        }
        FileCommand::Compare {
            first,
            second,
            offset,
            length,
            max_results,
            json,
        } => {
            let mut first = forensics::open_regular(&first)?;
            let mut second = forensics::open_regular(&second)?;
            let result = ranges::compare(
                &mut first,
                &mut second,
                ByteRange { offset, length },
                max_results,
            )?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "Equal: {}; differing bytes: {}; listing complete: {}",
                    result.equal, result.differing_bytes, result.complete
                )?;
                for difference in result.differences {
                    writeln!(
                        output,
                        "{:08x}  {} -> {}",
                        difference.offset,
                        difference
                            .left
                            .map(|b| format!("{b:02x}"))
                            .unwrap_or_else(|| "EOF".into()),
                        difference
                            .right
                            .map(|b| format!("{b:02x}"))
                            .unwrap_or_else(|| "EOF".into())
                    )?;
                }
            }
        }
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
