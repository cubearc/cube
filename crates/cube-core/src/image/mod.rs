//! Bounded 8-bit rasters, with exact PNG8 samples and explicit palette expansion.
//!
//! Coordinates start at the top left. Samples are row-major, interleaved, 8-bit,
//! unpremultiplied values. No gamma/profile conversion, compositing or normalization
//! occurs. Indexed images retain indices, palette and tRNS independently.

pub mod analyze;
pub mod bitwise;
pub use bitwise::BitwiseOperation;
pub mod colors;
pub mod csv;
pub mod layers;
pub mod pairs;
pub use colors::{ColorLayer, ColorOutput};
pub mod bits;
pub mod decode;
pub mod qr;
pub mod raw;
pub mod spatial;
pub mod stream;
pub mod traversal;
pub use bits::{BitOrder, BitPlane, Incomplete, LsbOptions, LsbResult, Traversal};
pub use raw::{RawInfo, RawLayout, SampleLayout};
pub use stream::PixelStream;
pub use traversal::Corner;
pub mod transform;
pub use analyze::{Channel, Comparison, Histogram, PixelData, Region};

use crate::{
    errors::{ErrorCode, OperationError},
    forensics,
};
use serde::Serialize;
use std::{
    fs::OpenOptions,
    io::{Cursor, Read, Write},
    path::Path,
};

/// Maximum encoded image/raw input or PNG output bytes (32 MiB).
pub const MAX_ENCODED: usize = 32 * 1024 * 1024;
/// Maximum pixel count (4 Mi pixels).
pub const MAX_PIXELS: u64 = 4 * 1024 * 1024;
/// Maximum sample buffer (16 MiB).
pub const MAX_SAMPLES: usize = 16 * 1024 * 1024;
/// Maximum side length, independently of the total pixel budget.
pub const MAX_DIMENSION: u32 = 16384;

pub(super) fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
pub(super) fn malformed(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
pub(super) fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}
pub(super) fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message)
}

/// Supported physical sample layouts; indexed samples are palette indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColorModel {
    Gray,
    GrayAlpha,
    Rgb,
    Rgba,
    Indexed,
}
impl ColorModel {
    /// Parse an exact stable sample-model name.
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "gray" => Ok(Self::Gray),
            "gray_alpha" => Ok(Self::GrayAlpha),
            "rgb" => Ok(Self::Rgb),
            "rgba" => Ok(Self::Rgba),
            "indexed" => Ok(Self::Indexed),
            _ => Err(
                invalid("model must be gray, gray_alpha, rgb, rgba or indexed").parameter("model"),
            ),
        }
    }
    pub fn channels(self) -> usize {
        match self {
            Self::Gray | Self::Indexed => 1,
            Self::GrayAlpha => 2,
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }
    fn png(self) -> png::ColorType {
        match self {
            Self::Gray => png::ColorType::Grayscale,
            Self::GrayAlpha => png::ColorType::GrayscaleAlpha,
            Self::Rgb => png::ColorType::Rgb,
            Self::Rgba => png::ColorType::Rgba,
            Self::Indexed => png::ColorType::Indexed,
        }
    }
}

/// Decoded or caller-constructed samples with validated shape. Fields are private so callers
/// cannot bypass resource, palette or transparency invariants.
#[derive(Clone, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    model: ColorModel,
    samples: Vec<u8>,
    palette: Vec<[u8; 3]>,
    transparency: Vec<u8>,
    source_chunks: Vec<forensics::png::Chunk>,
    exif: Option<Vec<u8>>,
    source_format: &'static str,
    representation: &'static str,
}

/// Image facts separate from sample bytes. Metadata is explicit and never decoded
/// into color corrections; eXIf is available only through `Image::exif`.
#[derive(Serialize, schemars::JsonSchema)]
pub struct ImageInfo<'a> {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub color_model: ColorModel,
    pub bit_depth: u8,
    pub channels: usize,
    pub row_stride: usize,
    pub sample_bytes: usize,
    pub origin: &'static str,
    pub palette: &'a [[u8; 3]],
    pub transparency: &'a [u8],
    pub chunks: &'a [forensics::png::Chunk],
    pub has_exif: bool,
}

fn sample_count(width: u32, height: u32, channels: usize) -> Result<usize, OperationError> {
    if width == 0 || height == 0 {
        return Err(invalid("image dimensions must be positive"));
    }
    if width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(limit("image exceeds dimension or 4 Mi pixel limit"));
    }
    let count = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(channels))
        .ok_or_else(|| limit("image sample count overflow"))?;
    if count > MAX_SAMPLES {
        return Err(limit("decoded samples exceed 16 MiB"));
    }
    Ok(count)
}

fn decode_error(error: png::DecodingError) -> OperationError {
    if matches!(error, png::DecodingError::LimitsExceeded) {
        limit("PNG decoder memory budget exceeded").caused_by(error)
    } else {
        malformed("malformed or unsupported PNG pixel stream").caused_by(error)
    }
}

impl Image {
    /// Validate borrowed buffers before allocating copies at an SDK boundary.
    pub fn from_bytes(
        width: u32,
        height: u32,
        model: ColorModel,
        samples: &[u8],
        palette: &[[u8; 3]],
        transparency: &[u8],
    ) -> Result<Self, OperationError> {
        if samples.len() != sample_count(width, height, model.channels())?
            || palette.len() > 256
            || transparency.len() > 256
        {
            return Err(invalid(
                "invalid sample shape or palette/transparency length",
            ));
        }
        Self::from_samples(
            width,
            height,
            model,
            samples.to_vec(),
            palette.to_vec(),
            transparency.to_vec(),
        )
    }

    /// Construct a validated sample image. tRNS is raw PNG transparency: indexed
    /// alpha entries, a big-endian gray key, or three big-endian RGB keys.
    pub fn from_samples(
        width: u32,
        height: u32,
        model: ColorModel,
        samples: Vec<u8>,
        palette: Vec<[u8; 3]>,
        transparency: Vec<u8>,
    ) -> Result<Self, OperationError> {
        if samples.len() != sample_count(width, height, model.channels())? {
            return Err(invalid("sample length does not match image shape"));
        }
        if palette.len() > 256
            || (matches!(model, ColorModel::Gray | ColorModel::GrayAlpha) && !palette.is_empty())
        {
            return Err(invalid("invalid palette for sample model"));
        }
        match model {
            ColorModel::Indexed => {
                if palette.is_empty()
                    || transparency.len() > palette.len()
                    || samples.iter().any(|index| *index as usize >= palette.len())
                {
                    return Err(malformed("invalid palette, transparency or pixel index"));
                }
            }
            ColorModel::Gray | ColorModel::Rgb => {
                let expected = if model == ColorModel::Gray { 2 } else { 6 };
                if !transparency.is_empty()
                    && (transparency.len() != expected
                        || transparency.as_chunks::<2>().0.iter().any(|v| v[0] != 0))
                {
                    return Err(malformed("invalid 8-bit transparency color key"));
                }
            }
            _ if !transparency.is_empty() => {
                return Err(malformed("tRNS is invalid for an alpha sample model"));
            }
            _ => (),
        }
        Ok(Self {
            width,
            height,
            model,
            samples,
            palette,
            transparency,
            source_chunks: vec![],
            exif: None,
            source_format: "png",
            representation: "constructed_8bit_samples",
        })
    }

    /// Decode supported bounded rasters. PNG8 uses identity transformations and
    /// shared strict framing/CRC validation; text/ICC decompression stays disabled.
    /// Additional codecs expose decoded pixels, explicitly labeled by provenance.
    pub fn decode(encoded: &[u8]) -> Result<Self, OperationError> {
        if encoded.len() > MAX_ENCODED {
            return Err(limit("encoded image exceeds 32 MiB").operation("image.decode"));
        }
        if encoded.starts_with(b"\x89PNG\r\n\x1a\n") {
            Self::decode_png(encoded)
        } else {
            decode::raster(encoded).map_err(|e| e.operation("image.decode"))
        }
    }

    fn decode_png(encoded: &[u8]) -> Result<Self, OperationError> {
        let result = (|| {
            if encoded.len() > MAX_ENCODED {
                return Err(limit("encoded PNG exceeds 32 MiB"));
            }
            if !encoded.starts_with(b"\x89PNG\r\n\x1a\n") {
                return Err(unsupported("image decoding currently supports PNG8 only"));
            }
            let report = forensics::png::inspect(&mut Cursor::new(encoded), true, 0)?;
            let raw_chunk = |kind: &str| -> Option<&[u8]> {
                report.chunks.iter().find(|c| c.kind == kind).map(|c| {
                    &encoded[c.offset as usize + 8..c.offset as usize + 8 + c.length as usize]
                })
            };
            let exif = raw_chunk("eXIf");
            if exif.is_some_and(|v| v.len() > forensics::exif::MAX_METADATA_BYTES) {
                return Err(limit("PNG EXIF exceeds 1 MiB"));
            }
            let exif = exif.map(<[u8]>::to_vec);
            let mut decoder = png::Decoder::new_with_limits(
                Cursor::new(encoded),
                png::Limits {
                    bytes: 32 * 1024 * 1024,
                },
            );
            decoder.set_transformations(png::Transformations::IDENTITY);
            decoder.set_ignore_text_chunk(true);
            decoder.set_ignore_iccp_chunk(true);
            let header = decoder.read_header_info().map_err(decode_error)?;
            if header.bit_depth != png::BitDepth::Eight {
                return Err(unsupported(
                    "only 8-bit PNG samples are supported; no downsampling is performed",
                ));
            }
            sample_count(header.width, header.height, header.color_type.samples())?;
            let mut reader = decoder.read_info().map_err(decode_error)?;
            let info = reader.info();
            let model = match info.color_type {
                png::ColorType::Grayscale => ColorModel::Gray,
                png::ColorType::GrayscaleAlpha => ColorModel::GrayAlpha,
                png::ColorType::Rgb => ColorModel::Rgb,
                png::ColorType::Rgba => ColorModel::Rgba,
                png::ColorType::Indexed => ColorModel::Indexed,
            };
            let width = info.width;
            let height = info.height;
            let palette = info
                .palette
                .as_ref()
                .map(|p| p.as_chunks::<3>().0.to_vec())
                .unwrap_or_default();
            let transparency = raw_chunk("tRNS").unwrap_or_default();
            if transparency.len() > 256 {
                return Err(malformed("invalid PNG transparency length"));
            }
            let transparency = transparency.to_vec();
            let count = reader
                .output_buffer_size()
                .ok_or_else(|| limit("PNG output size overflow"))?;
            if count != sample_count(width, height, model.channels())? {
                return Err(malformed("PNG output layout differs from declared samples"));
            }
            let mut samples = vec![0; count];
            let frame = reader.next_frame(&mut samples).map_err(decode_error)?;
            samples.truncate(frame.buffer_size());
            reader.finish().map_err(decode_error)?;
            let mut image =
                Self::from_samples(width, height, model, samples, palette, transparency)?;
            image.source_chunks = report.chunks;
            image.exif = exif;
            image.representation = "native_png8_samples";
            Ok(image)
        })();
        result.map_err(|e: OperationError| e.operation("image.decode"))
    }

    /// Decode a regular file. Symlinks/devices are rejected; input reads are bounded.
    pub fn open(path: &Path) -> Result<Self, OperationError> {
        let mut file = forensics::open_regular(path).map_err(|e| e.operation("image.decode"))?;
        if file.metadata()?.len() > MAX_ENCODED as u64 {
            return Err(limit("encoded image exceeds 32 MiB").operation("image.decode"));
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_ENCODED as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes)
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn model(&self) -> ColorModel {
        self.model
    }
    pub fn samples(&self) -> &[u8] {
        &self.samples
    }
    pub fn palette(&self) -> &[[u8; 3]] {
        &self.palette
    }
    pub fn transparency(&self) -> &[u8] {
        &self.transparency
    }

    /// Inspect source model/palette/transparency and chunk metadata, not pixel bytes.
    pub fn info(&self) -> ImageInfo<'_> {
        ImageInfo {
            format: self.source_format,
            width: self.width,
            height: self.height,
            color_model: self.model,
            bit_depth: 8,
            channels: self.model.channels(),
            row_stride: self.width as usize * self.model.channels(),
            sample_bytes: self.samples.len(),
            origin: "top_left",
            palette: &self.palette,
            transparency: &self.transparency,
            chunks: &self.source_chunks,
            has_exif: self.exif.is_some(),
        }
    }

    /// Explicit eXIf parsing reuses the bounded TIFF implementation from v0.3.0.
    pub fn exif(&self) -> Result<Option<forensics::exif::Metadata>, OperationError> {
        self.exif
            .as_ref()
            .map(|bytes| forensics::exif::inspect(bytes))
            .transpose()
    }

    /// Explicitly expand palette indices to RGBA8, supplying alpha 255 for entries
    /// without tRNS. Direct samples and the original image remain unchanged.
    pub fn expand_palette(&self) -> Result<Self, OperationError> {
        if self.model != ColorModel::Indexed {
            return Err(invalid("palette expansion requires indexed input")
                .operation("image.palette.expand"));
        }
        let mut samples = Vec::with_capacity(sample_count(self.width, self.height, 4)?);
        for index in &self.samples {
            samples.extend_from_slice(&self.palette[*index as usize]);
            samples.push(
                self.transparency
                    .get(*index as usize)
                    .copied()
                    .unwrap_or(255),
            );
        }
        Self::from_samples(
            self.width,
            self.height,
            ColorModel::Rgba,
            samples,
            vec![],
            vec![],
        )
    }

    /// Encode exact samples as PNG8. Palette/tRNS survive; all other original
    /// metadata (including profiles/text/EXIF) is deliberately dropped to avoid stale
    /// attributes. Pixel values remain exact but file bytes need not match the source.
    pub fn encode_png(&self) -> Result<Vec<u8>, OperationError> {
        let result = (|| {
            let mut bytes = Vec::new();
            {
                let mut encoder = png::Encoder::new(&mut bytes, self.width, self.height);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.set_color(self.model.png());
                if !self.palette.is_empty() {
                    encoder.set_palette(self.palette.iter().flatten().copied().collect::<Vec<_>>());
                }
                if !self.transparency.is_empty() {
                    encoder.set_trns(self.transparency.clone());
                }
                let mut writer = encoder
                    .write_header()
                    .map_err(|e| malformed("PNG encoding failed").caused_by(e))?;
                writer
                    .write_image_data(&self.samples)
                    .map_err(|e| malformed("PNG sample encoding failed").caused_by(e))?;
                writer
                    .finish()
                    .map_err(|e| malformed("PNG finalization failed").caused_by(e))?;
            }
            if bytes.len() > MAX_ENCODED {
                return Err(limit("encoded output exceeds 32 MiB"));
            }
            Ok(bytes)
        })();
        result.map_err(|e: OperationError| e.operation("image.encode"))
    }

    /// Save to a newly created file; existing destinations and source aliases fail.
    pub fn save_png(&self, path: &Path) -> Result<(), OperationError> {
        save_new(path, &self.encode_png()?).map_err(|e| e.operation("image.encode"))
    }
}

/// Write an explicit bounded analysis result to a new file, never overwriting.
pub fn save_new(path: &Path, bytes: &[u8]) -> Result<(), OperationError> {
    if bytes.len() > MAX_ENCODED {
        return Err(limit("analysis output exceeds 32 MiB"));
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let result = file.write_all(bytes).and_then(|()| file.flush());
    drop(file);
    if result.is_err() {
        let _cleanup = std::fs::remove_file(path);
    }
    result.map_err(OperationError::io)
}

/// Metadata for the direct codec and explicit conversion functions.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "image.decode",
        category: "image",
        description: "Decode bounded PNG8, baseline JPEG, single-frame GIF, BMP or static WebP; preserve PNG8 source samples and report decoded raster provenance.",
        input_type: "bytes",
        output_type: "image.samples",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "image inspect",
        example: "cube image inspect sample.png --json",
    },
    crate::operations::Descriptor {
        id: "image.encode",
        category: "image",
        description: "Encode exact PNG8 samples, preserving palette/tRNS and dropping other metadata.",
        input_type: "image.samples",
        output_type: "bytes",
        parameters: &[],
        deterministic: true,
        reversible: None,
        command: "image convert",
        example: "cube image convert sample.png --output copy.png",
    },
    crate::operations::Descriptor {
        id: "image.palette.expand",
        category: "image",
        description: "Explicitly expand indexed PNG8 samples to unpremultiplied RGBA8.",
        input_type: "image.samples",
        output_type: "image.samples",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "image convert",
        example: "cube image convert sample.png --expand-palette --output rgba.png",
    },
];
