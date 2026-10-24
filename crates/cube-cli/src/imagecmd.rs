//! CLI adaptation of the shared image operations; no pixel algorithms live here.
use clap::Subcommand;
use cube_core::{forensics, image::Image};
use std::{error::Error, io::Write, path::PathBuf};

#[derive(Subcommand)]
pub(crate) enum ImageCommand {
    /// Write exact PNG8 samples to a new PNG, optionally expanding indexed pixels
    Convert {
        path: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        expand_palette: bool,
    },
    /// Inspect exact PNG8 shape, source model and palette without dumping samples
    Inspect {
        path: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        exif: bool,
    },
    /// Inspect PNG framing and CRC evidence without decoding pixels
    Chunks {
        path: PathBuf,
        /// Report CRC mismatches while still rejecting unsafe structure
        #[arg(long)]
        allow_bad_crc: bool,
        /// Optional total chunk payload export cap, at most 1048576 bytes
        #[arg(long, default_value_t = 0)]
        payload_budget: usize,
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn run(command: ImageCommand, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        ImageCommand::Convert {
            path,
            output: destination,
            expand_palette,
        } => {
            let image = Image::open(&path)?;
            let image = if expand_palette {
                image.expand_palette()?
            } else {
                image
            };
            image.save_png(&destination)?;
            writeln!(output, "Wrote PNG8 {} x {}", image.width(), image.height())?;
        }
        ImageCommand::Inspect { path, json, exif } => {
            let image = Image::open(&path)?;
            if json {
                let mut info = serde_json::json!(image.info());
                if exif {
                    info["exif"] = serde_json::json!(image.exif()?);
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    output,
                    "PNG8 {} x {}\nModel: {:?}\nChannels: {}\nSample bytes: {}\nPalette entries: {}",
                    image.width(),
                    image.height(),
                    image.model(),
                    image.model().channels(),
                    image.samples().len(),
                    image.palette().len()
                )?;
            }
            if exif && !json {
                writeln!(output, "{}", serde_json::to_string_pretty(&image.exif()?)?)?;
            }
        }
        ImageCommand::Chunks {
            path,
            allow_bad_crc,
            payload_budget,
            json,
        } => {
            let report = forensics::png::inspect(
                &mut forensics::open_regular(&path)?,
                !allow_bad_crc,
                payload_budget,
            )?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
            } else {
                for chunk in report.chunks {
                    writeln!(
                        output,
                        "{:08x} {} length={} CRC={:08x}/{:08x} valid={}",
                        chunk.offset,
                        chunk.kind,
                        chunk.length,
                        chunk.stored_crc,
                        chunk.calculated_crc,
                        chunk.valid
                    )?;
                    if let Some(payload) = chunk.payload_hex {
                        writeln!(output, "payload_hex={payload}")?;
                    }
                }
                writeln!(
                    output,
                    "CRCs valid: {}; trailing bytes: {}",
                    report.crc_valid, report.boundary.trailing_bytes
                )?;
            }
        }
    }
    Ok(())
}
