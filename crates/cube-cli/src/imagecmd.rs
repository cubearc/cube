//! CLI adaptation of the shared image operations; no pixel algorithms live here.
use clap::{Args, Subcommand};
use cube_core::{
    forensics,
    image::{self, Channel, Image, PixelData, Region},
};
use std::{error::Error, io::Write, path::PathBuf};

#[derive(Args)]
pub(crate) struct ViewOptions {
    /// Checked x,y,width,height rectangle; top-left origin
    #[arg(long)]
    region: Option<Region>,
    /// Explicitly expand indexed samples to RGBA before the operation
    #[arg(long)]
    expand_palette: bool,
}
impl ViewOptions {
    fn region(&self) -> Option<Region> {
        self.region
    }
    fn open(&self, path: &std::path::Path) -> Result<Image, Box<dyn Error>> {
        let image = Image::open(path)?;
        Ok(if self.expand_palette {
            image.expand_palette()?
        } else {
            image
        })
    }
}
fn selected(names: Option<Vec<String>>) -> Result<Option<Vec<Channel>>, Box<dyn Error>> {
    names
        .map(|v| {
            v.iter()
                .map(|name| Channel::parse(name).map_err(Into::into))
                .collect()
        })
        .transpose()
}

#[derive(Args)]
#[group(required=true,multiple=false,args=["output","stdout","png_output","json"])]
pub(crate) struct PixelOutput {
    /// New file containing exact raw samples; shape metadata goes to stdout
    #[arg(long)]
    output: Option<PathBuf>,
    /// Exact binary stdout without a newline or metadata
    #[arg(long)]
    stdout: bool,
    /// New lossless grayscale PNG (requires one selected channel)
    #[arg(long)]
    png_output: Option<PathBuf>,
    /// Explicit coordinate records, capped at 10000 pixels
    #[arg(long)]
    json: bool,
}
impl PixelOutput {
    fn write(self, data: PixelData, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
        if self.stdout {
            output.write_all(&data.data)?;
            return Ok(());
        }
        if let Some(path) = self.output {
            image::save_new(&path, &data.data)?;
        } else if let Some(path) = self.png_output {
            data.grayscale_image()?.save_png(&path)?;
        }
        let mut metadata = serde_json::json!(data);
        if self.json {
            metadata["records"] = serde_json::json!(data.records()?);
        }
        writeln!(output, "{}", serde_json::to_string_pretty(&metadata)?)?;
        Ok(())
    }
}

#[derive(Subcommand)]
pub(crate) enum ImageCommand {
    /// Read native pixel samples at a zero-based coordinate
    Pixel {
        path: PathBuf,
        #[arg(long)]
        x: u32,
        #[arg(long)]
        y: u32,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long)]
        json: bool,
    },
    /// Export selected pixel samples with explicit shape/channel information
    Pixels {
        path: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        #[command(flatten)]
        destination: PixelOutput,
    },
    /// Extract an exact named channel as bytes, coordinate records or grayscale PNG
    Channel {
        path: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long)]
        channel: String,
        #[command(flatten)]
        destination: PixelOutput,
    },
    /// Count exact channel tuples; exceeding the color budget fails explicitly
    Histogram {
        path: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        #[arg(long, default_value_t = 1000)]
        max_colors: usize,
        #[arg(long)]
        json: bool,
    },
    /// Compare exact pixel/channel values without resizing or blending
    Compare {
        first: PathBuf,
        second: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        #[arg(long, default_value_t = 1000)]
        max_results: usize,
        #[arg(long)]
        mask_output: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
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
        ImageCommand::Pixel {
            path,
            x,
            y,
            expand_palette,
            json,
        } => {
            let image = Image::open(&path)?;
            let view = image.view(expand_palette)?;
            let values = view.pixel(x, y)?;
            if json {
                writeln!(
                    output,
                    "{}",
                    serde_json::json!({"x":x,"y":y,"channels":view.channels(),"values":values})
                )?;
            } else {
                writeln!(output, "({x}, {y}) {:?}: {:?}", view.channels(), values)?;
            }
        }
        ImageCommand::Pixels {
            path,
            view,
            channels,
            destination,
        } => {
            let image = view.open(&path)?;
            let channels = selected(channels)?;
            destination.write(image.pixels(view.region(), channels.as_deref())?, output)?;
        }
        ImageCommand::Channel {
            path,
            view,
            channel,
            destination,
        } => {
            let image = view.open(&path)?;
            destination.write(
                image.channel(Channel::parse(&channel)?, view.region())?,
                output,
            )?;
        }
        ImageCommand::Histogram {
            path,
            view,
            channels,
            max_colors,
            json,
        } => {
            let image = view.open(&path)?;
            let channels = selected(channels)?;
            let histogram = image.histogram(view.region(), channels.as_deref(), max_colors)?;
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&histogram)?)?;
            } else {
                writeln!(
                    output,
                    "Pixels: {}; channels: {:?}",
                    histogram.pixels, histogram.channels
                )?;
                for count in histogram.colors {
                    writeln!(output, "{:?}: {}", count.values, count.count)?;
                }
            }
        }
        ImageCommand::Compare {
            first,
            second,
            view,
            channels,
            max_results,
            mask_output,
            json,
        } => {
            let first = view.open(&first)?;
            let second = view.open(&second)?;
            let channels = selected(channels)?;
            let result = first.compare(
                &second,
                view.region(),
                channels.as_deref(),
                max_results,
                mask_output.is_some(),
            )?;
            if let Some(path) = mask_output {
                result
                    .mask
                    .as_ref()
                    .expect("requested mask")
                    .save_png(&path)?;
            }
            if json {
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            } else {
                writeln!(
                    output,
                    "Equal: {}; differing pixels: {}; listing complete: {}",
                    result.equal, result.differing_pixels, result.complete
                )?;
                for difference in result.differences {
                    writeln!(
                        output,
                        "({}, {}) {:?} -> {:?}",
                        difference.x, difference.y, difference.left, difference.right
                    )?;
                }
            }
        }
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
