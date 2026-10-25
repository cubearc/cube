//! CLI adaptation of the shared image operations; no pixel algorithms live here.
use clap::{Args, Subcommand};
use cube_core::{
    forensics,
    image::{self, BitwiseOperation, Channel, Image, PixelData, Region},
};
use std::{error::Error, io::Write, path::PathBuf};
mod csv;
mod output;
mod pairs;

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
#[group(required=true,multiple=false,args=["output","stdout","png_output","json","hex"])]
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
    /// Exact hexadecimal bytes with one final newline
    #[arg(long)]
    hex: bool,
}
impl PixelOutput {
    fn write(self, data: PixelData, output: &mut impl Write) -> Result<(), Box<dyn Error>> {
        if self.hex {
            writeln!(output, "{}", cube_core::encoding::hex_encode(&data.data))?;
            return Ok(());
        }
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
    fn write_stream(
        self,
        stream: image::PixelStream,
        output: &mut impl Write,
    ) -> Result<(), Box<dyn Error>> {
        if self.stdout {
            output.write_all(&stream.pixels.data)?;
            return Ok(());
        }
        if self.hex {
            writeln!(
                output,
                "{}",
                cube_core::encoding::hex_encode(&stream.pixels.data)
            )?;
            return Ok(());
        }
        if let Some(path) = self.png_output {
            if stream.traversal != image::Traversal::Row
                || stream.start_corner != image::Corner::TopLeft
                || stream.reverse
            {
                return Err("PNG channel export requires default row traversal; use raw bytes for other orders".into());
            }
            stream.pixels.grayscale_image()?.save_png(&path)?;
        } else if let Some(path) = self.output {
            image::save_new(&path, &stream.pixels.data)?;
        }
        let mut metadata = serde_json::json!(stream);
        if self.json {
            metadata["records"] = serde_json::json!(stream.records()?);
        }
        writeln!(output, "{}", serde_json::to_string_pretty(&metadata)?)?;
        Ok(())
    }
}

#[derive(Args)]
#[group(required=true,multiple=false,args=["output","stdout"])]
pub(crate) struct ByteOutput {
    /// Write exact bytes to a new file; effective extraction parameters go to stdout
    #[arg(long)]
    output: Option<PathBuf>,
    /// Write exact bytes without a newline or metadata
    #[arg(long)]
    stdout: bool,
}
impl PixelOutput {
    fn write_plane(
        self,
        plane: image::BitPlane,
        packed: bool,
        output: &mut impl Write,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = if packed {
            &plane.packed
        } else {
            &plane.pixels.data
        };
        if self.hex {
            writeln!(output, "{}", cube_core::encoding::hex_encode(bytes))?;
            return Ok(());
        }
        if self.stdout {
            output.write_all(bytes)?;
            return Ok(());
        }
        let visual = self.png_output.is_some();
        if let Some(path) = self.output {
            image::save_new(&path, bytes)?;
        } else if let Some(path) = self.png_output {
            plane.visualization()?.save_png(&path)?;
        }
        let mut metadata = serde_json::json!(plane);
        metadata["representation"] = serde_json::json!(if visual {
            "visualization_gray_0_255"
        } else if packed {
            "packed_msb_zero_pad"
        } else {
            "bit_values"
        });
        if self.json {
            metadata["records"] = serde_json::json!(plane.pixels.records()?);
            metadata["packed_hex"] =
                serde_json::json!(cube_core::encoding::hex_encode(&plane.packed));
        }
        writeln!(output, "{}", serde_json::to_string_pretty(&metadata)?)?;
        Ok(())
    }
}

#[derive(Subcommand)]
pub(crate) enum ImageCommand {
    /// Exchange complete bounded exact-sample CSV v1 grids
    Csv {
        #[command(subcommand)]
        command: csv::CsvCommand,
    },
    /// Plan/execute bounded explicit pairs using existing single-pair operations
    Pairs(pairs::PairArgs),
    /// Export exact-color PNGs and manifest.json into an existing directory
    ColorLayers {
        path: PathBuf,
        #[arg(long, value_delimiter = ',')]
        colors: Option<Vec<String>>,
        #[arg(long, default_value = "mask")]
        representation: String,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long, default_value_t = 16)]
        max_colors: usize,
        #[arg(long, default_value_t = 16)]
        max_outputs: usize,
        #[arg(long)]
        output_dir: PathBuf,
    },
    /// Detect/decode QR symbols as inert bytes; default JSON includes payload Hex
    #[command(group(clap::ArgGroup::new("qr_destination").multiple(false).args(["json","stdout","output"])))]
    Qr {
        path: PathBuf,
        #[arg(long, default_value_t = 8)]
        max_symbols: usize,
        #[arg(long)]
        json: bool,
        /// Requires exactly one successfully decoded symbol
        #[arg(long)]
        stdout: bool,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Export a mask or RGBA layer for exact native colors (hex sample tuples)
    Colors {
        path: PathBuf,
        #[arg(long, value_delimiter = ',', required = true)]
        colors: Vec<String>,
        #[arg(long, default_value = "mask")]
        representation: String,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long)]
        output: PathBuf,
    },
    /// Inspect an explicit raw 8-bit layout; optionally export a new PNG
    Raw {
        path: PathBuf,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
        #[arg(long)]
        layout: String,
        /// Complete row bytes, including padding on the final row
        #[arg(long)]
        row_stride: Option<usize>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Compare parsed header observations with the independent raw hypothesis
        #[arg(long)]
        compare_header: bool,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Extract row-major 0/1 bits; optional MSB packing or explicit 0/255 PNG
    BitPlane {
        path: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long)]
        channel: String,
        #[arg(long)]
        bit: u8,
        #[arg(long,conflicts_with_all=["json","png_output"])]
        packed: bool,
        #[command(flatten)]
        destination: PixelOutput,
    },
    /// Extract raw bytes in pixel/channel/bit order, without text conversion
    Lsb {
        path: PathBuf,
        #[command(flatten)]
        view: ViewOptions,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        /// Ordered distinct positions, zero=LSB
        #[arg(long, value_delimiter = ',', default_value = "0")]
        bits: Vec<u8>,
        #[arg(long, default_value = "row")]
        traversal: String,
        #[arg(long, default_value = "top_left")]
        start_corner: String,
        /// Reverse pixel order only
        #[arg(long)]
        reverse: bool,
        #[arg(long, default_value = "msb")]
        bit_order: String,
        #[arg(long, default_value = "reject")]
        incomplete: String,
        #[command(flatten)]
        destination: ByteOutput,
    },
    /// Exact clockwise quarter-turn rotation; write a new lossless PNG
    Rotate {
        path: PathBuf,
        #[arg(long)]
        degrees: u32,
        #[arg(long)]
        output: PathBuf,
    },
    /// Mirror left/right (horizontal) or top/bottom (vertical)
    Mirror {
        path: PathBuf,
        #[arg(long)]
        axis: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Extract an exact x,y,width,height region; discard unselected pixels
    Crop {
        path: PathBuf,
        #[arg(long)]
        region: Region,
        #[arg(long)]
        output: PathBuf,
    },
    /// Invert 8-bit color samples; alpha is unchanged unless explicitly selected
    Invert {
        path: PathBuf,
        #[arg(long, value_delimiter = ',')]
        channels: Option<Vec<String>>,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long)]
        output: PathBuf,
    },
    /// Apply a full channel permutation; alpha movement must be explicit
    Reorder {
        path: PathBuf,
        #[arg(long, value_delimiter = ',', required = true)]
        channels: Vec<String>,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long)]
        output: PathBuf,
    },
    /// Reduce color using (77R+150G+29B+128)>>8; preserve alpha, drop color information
    Grayscale {
        path: PathBuf,
        #[arg(long)]
        expand_palette: bool,
        #[arg(long)]
        output: PathBuf,
    },
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
        #[arg(long, default_value = "row")]
        traversal: String,
        #[arg(long, default_value = "top_left")]
        start_corner: String,
        #[arg(long)]
        reverse: bool,
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
    /// Combine compatible 8-bit images with XOR/AND/OR, including physical alpha
    Bitwise {
        first: PathBuf,
        second: PathBuf,
        /// Exact operation: xor, and, or; no operand cycling or blending
        #[arg(long)]
        operation: String,
        /// New lossless PNG; existing destinations are rejected
        #[arg(long)]
        output: PathBuf,
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
    /// Write decoded 8-bit samples to a new PNG, optionally expanding indexed pixels
    Convert {
        path: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        expand_palette: bool,
    },
    /// Inspect supported raster shape, model and decoding provenance
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
        ImageCommand::Csv { command } => csv::run(command, output)?,
        ImageCommand::Pairs(args) => pairs::run(args, output)?,
        ImageCommand::ColorLayers {
            path,
            colors,
            representation,
            expand_palette,
            max_colors,
            max_outputs,
            output_dir,
        } => {
            if colors
                .as_ref()
                .is_some_and(|v| v.len() > 64 || v.iter().any(|s| s.len() > 8))
            {
                return Err("color selection exceeds 64 four-byte tuples".into());
            }
            let colors = colors
                .map(|v| {
                    v.iter()
                        .map(|s| cube_core::encoding::hex_decode(s))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let image = Image::open(&path)?;
            let report = image.view(expand_palette)?.color_layers(
                colors.as_deref(),
                image::ColorOutput::parse(&representation)?,
                max_colors,
                max_outputs,
            )?;
            let manifest = serde_json::to_vec_pretty(&report)?;
            let mut files: Vec<_> = report
                .outputs
                .into_iter()
                .map(|entry| (entry.name, entry.png))
                .collect();
            files.push(("manifest.json".into(), manifest.clone()));
            output::write_outputs(&output_dir, &files)?;
            output.write_all(&manifest)?;
            writeln!(output)?;
        }
        ImageCommand::Qr {
            path,
            max_symbols,
            json: _,
            stdout,
            output: destination,
        } => {
            let report = Image::open(&path)?.qr(max_symbols)?;
            if stdout || destination.is_some() {
                let payload = match report.symbols.as_slice() {
                    [symbol] => symbol.payload.as_deref(),
                    _ => None,
                }
                .ok_or("raw QR output requires exactly one successfully decoded symbol")?;
                if let Some(path) = destination {
                    image::save_new(&path, payload)?;
                } else {
                    output.write_all(payload)?;
                }
            } else {
                let mut value = serde_json::json!(report);
                for (value, symbol) in value["symbols"]
                    .as_array_mut()
                    .expect("symbol array")
                    .iter_mut()
                    .zip(&report.symbols)
                {
                    value["payload_hex"] = serde_json::json!(
                        symbol
                            .payload
                            .as_ref()
                            .map(|data| cube_core::encoding::hex_encode(data))
                    );
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&value)?)?;
            }
        }
        ImageCommand::Colors {
            path,
            colors,
            representation,
            expand_palette,
            output: destination,
        } => {
            if colors.len() > 256 || colors.iter().any(|s| s.len() > 8) {
                return Err(
                    "select at most 256 colors, each containing at most four hex bytes".into(),
                );
            }
            let colors = colors
                .iter()
                .map(|s| cube_core::encoding::hex_decode(s))
                .collect::<Result<Vec<_>, _>>()?;
            let image = Image::open(&path)?;
            let result = image
                .view(expand_palette)?
                .exact_colors(&colors, image::ColorOutput::parse(&representation)?)?;
            result.image.save_png(&destination)?;
            writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
        }
        ImageCommand::Raw {
            path,
            width,
            height,
            layout,
            row_stride,
            offset,
            compare_header,
            output: destination,
        } => {
            let data = crate::pipelinecmd::read_file(&path, image::MAX_ENCODED)?;
            let layout = image::RawLayout {
                width,
                height,
                layout: image::SampleLayout::parse(&layout)?,
                row_stride,
                offset,
            };
            let info = layout.inspect(data.len())?;
            if let Some(destination) = destination {
                Image::from_raw(&data, &layout)?.save_png(&destination)?;
            }
            if compare_header {
                writeln!(
                    output,
                    "{}",
                    serde_json::to_string_pretty(&layout.inspect_bytes(&data)?)?
                )?;
            } else {
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            }
        }
        ImageCommand::BitPlane {
            path,
            view,
            channel,
            bit,
            packed,
            destination,
        } => {
            let image = view.open(&path)?;
            let result = image.bit_plane(Channel::parse(&channel)?, bit, view.region())?;
            destination.write_plane(result, packed, output)?;
        }
        ImageCommand::Lsb {
            path,
            view,
            channels,
            bits,
            traversal,
            start_corner,
            reverse,
            bit_order,
            incomplete,
            destination,
        } => {
            let options = image::LsbOptions {
                region: view.region(),
                channels: selected(channels)?,
                bits,
                traversal: image::Traversal::parse(&traversal)?,
                reverse,
                bit_order: image::BitOrder::parse(&bit_order)?,
                incomplete: image::Incomplete::parse(&incomplete)?,
            };
            let result = view
                .open(&path)?
                .lsb_with_corner(&options, image::Corner::parse(&start_corner)?)?;
            if destination.stdout {
                output.write_all(&result.result.data)?;
            } else if let Some(path) = destination.output {
                image::save_new(&path, &result.result.data)?;
                writeln!(output, "{}", serde_json::to_string_pretty(&result)?)?;
            }
        }
        ImageCommand::Rotate {
            path,
            degrees,
            output,
        } => Image::open(&path)?.rotate(degrees)?.save_png(&output)?,
        ImageCommand::Mirror { path, axis, output } => {
            Image::open(&path)?.mirror(&axis)?.save_png(&output)?
        }
        ImageCommand::Crop {
            path,
            region,
            output,
        } => Image::open(&path)?.crop(region)?.save_png(&output)?,
        ImageCommand::Invert {
            path,
            channels,
            expand_palette,
            output,
        } => {
            let image = Image::open(&path)?;
            let selected = selected(channels)?;
            image
                .view(expand_palette)?
                .invert(selected.as_deref())?
                .save_png(&output)?;
        }
        ImageCommand::Reorder {
            path,
            channels,
            expand_palette,
            output,
        } => {
            let image = Image::open(&path)?;
            let selected = selected(Some(channels))?.expect("present channels");
            image
                .view(expand_palette)?
                .reorder(&selected)?
                .save_png(&output)?;
        }
        ImageCommand::Grayscale {
            path,
            expand_palette,
            output,
        } => {
            let image = Image::open(&path)?;
            image.view(expand_palette)?.grayscale()?.save_png(&output)?;
        }
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
            traversal,
            start_corner,
            reverse,
            destination,
        } => {
            let image = view.open(&path)?;
            let channels = selected(channels)?;
            destination.write_stream(
                image.pixels_with_traversal(
                    view.region(),
                    channels.as_deref(),
                    image::Traversal::parse(&traversal)?,
                    image::Corner::parse(&start_corner)?,
                    reverse,
                )?,
                output,
            )?;
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
        ImageCommand::Bitwise {
            first,
            second,
            operation,
            output: destination,
            json,
        } => {
            let operation = BitwiseOperation::parse(&operation)?;
            let left = Image::open(&first)?;
            let right = Image::open(&second)?;
            let image = left.bitwise(&right, operation)?;
            image.save_png(&destination)?;
            if json {
                writeln!(
                    output,
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "operation": operation,
                        "image": image.info(),
                        "provenance": image.provenance(),
                    }))?
                )?;
            } else {
                writeln!(output, "Wrote PNG8 {} x {}", image.width(), image.height())?;
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
                info["provenance"] = serde_json::json!(image.provenance());
                if exif {
                    info["exif"] = serde_json::json!(image.exif()?);
                }
                writeln!(output, "{}", serde_json::to_string_pretty(&info)?)?;
            } else {
                writeln!(
                    output,
                    "{} {} x {}\nModel: {:?}\nChannels: {}\nSample bytes: {}\nPalette entries: {}",
                    if image.info().format == "png" {
                        "PNG8"
                    } else {
                        image.info().format
                    },
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
