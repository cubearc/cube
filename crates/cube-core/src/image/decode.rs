//! Additional raster codecs. The identity PNG8 decoder remains independent.
use super::{ColorModel, Image, MAX_DIMENSION, malformed, sample_count, unsupported};
use crate::{errors::OperationError, forensics};
use raster_codecs::{AnimationDecoder, ColorType, ImageDecoder, ImageError, Limits, codecs};
use serde::Serialize;
use std::io::Cursor;

fn error(error: ImageError) -> OperationError {
    match &error {
        ImageError::Limits(_) => super::limit("raster decoder limit exceeded"),
        ImageError::Unsupported(_) => unsupported("unsupported raster encoding"),
        _ => malformed("malformed raster image"),
    }
    .caused_by(error)
}
fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(64 * 1024 * 1024);
    limits
}
fn read(mut decoder: impl ImageDecoder) -> Result<Image, OperationError> {
    decoder.set_limits(limits()).map_err(error)?;
    let model = match decoder.color_type() {
        ColorType::L8 => ColorModel::Gray,
        ColorType::La8 => ColorModel::GrayAlpha,
        ColorType::Rgb8 => ColorModel::Rgb,
        ColorType::Rgba8 => ColorModel::Rgba,
        _ => {
            return Err(unsupported(
                "only unsigned 8-bit decoded raster samples are supported",
            ));
        }
    };
    let (width, height) = decoder.dimensions();
    let count = sample_count(width, height, model.channels())?;
    if decoder.total_bytes() != count as u64 {
        return Err(malformed("inconsistent decoded raster layout"));
    }
    let mut samples = vec![0; count];
    decoder.read_image(&mut samples).map_err(error)?;
    Image::from_samples(width, height, model, samples, vec![], vec![])
}

pub(super) fn raster(encoded: &[u8]) -> Result<Image, OperationError> {
    let input = Cursor::new(encoded);
    let (mut image, format) = if encoded.starts_with(b"\xff\xd8") {
        // Reject incomplete framing even when a permissive codec can recover pixels.
        forensics::boundaries::inspect(&mut Cursor::new(encoded))?;
        (
            read(codecs::jpeg::JpegDecoder::new(input).map_err(error)?)?,
            "jpeg",
        )
    } else if encoded.starts_with(b"GIF87a") || encoded.starts_with(b"GIF89a") {
        forensics::boundaries::inspect(&mut Cursor::new(encoded))?;
        let mut decoder = codecs::gif::GifDecoder::new(input).map_err(error)?;
        decoder.set_limits(limits()).map_err(error)?;
        let (width, height) = decoder.dimensions();
        sample_count(width, height, 4)?;
        let mut frames = decoder.into_frames();
        let first = frames
            .next()
            .ok_or_else(|| malformed("GIF contains no image frame"))?
            .map_err(error)?;
        if let Some(next) = frames.next() {
            next.map_err(error)?;
            return Err(unsupported(
                "animated GIF is unsupported; exactly one frame is required",
            ));
        }
        let frame = first.into_buffer();
        if frame.dimensions() != (width, height) {
            return Err(malformed("GIF canvas differs from decoded frame"));
        }
        (
            Image::from_samples(
                width,
                height,
                ColorModel::Rgba,
                frame.into_raw(),
                vec![],
                vec![],
            )?,
            "gif",
        )
    } else if encoded.starts_with(b"BM") {
        (
            read(codecs::bmp::BmpDecoder::new(input).map_err(error)?)?,
            "bmp",
        )
    } else if encoded.starts_with(b"RIFF") && encoded.get(8..12) == Some(b"WEBP") {
        let declared =
            u32::from_le_bytes(encoded[4..8].try_into().expect("RIFF header")) as u64 + 8;
        if declared > encoded.len() as u64 || declared < 12 {
            return Err(malformed("truncated or invalid WebP RIFF extent"));
        }
        let decoder = codecs::webp::WebPDecoder::new(input).map_err(error)?;
        if decoder.has_animation() {
            return Err(unsupported(
                "animated WebP is unsupported; exactly one frame is required",
            ));
        }
        (read(decoder)?, "webp")
    } else {
        return Err(unsupported(
            "supported images are PNG8, baseline JPEG, single-frame GIF, BMP and static WebP",
        ));
    };
    image.source_format = format;
    Ok(image)
}

/// Interpretation of a decoded image, separate from encoded bytes and profiles.
#[derive(Debug, Serialize)]
pub struct Provenance {
    pub format: &'static str,
    pub representation: &'static str,
    pub source_samples_preserved: bool,
    pub color_management: &'static str,
    pub orientation: &'static str,
    pub frame_policy: &'static str,
}
impl Image {
    /// Describe raster conversion explicitly. JPEG YCbCr/CMYK conversion and GIF/
    /// BMP palette expansion are codec behavior; no ICC/gamma/EXIF correction runs.
    /// Transformations describe their resulting samples, not an encoded source.
    pub fn provenance(&self) -> Provenance {
        Provenance {
            format: self.source_format,
            representation: match self.source_format {
                "png" => "native_png8_samples",
                "raw" => "explicit_raw8_samples",
                _ => "decoded_raster8_samples",
            },
            source_samples_preserved: matches!(self.source_format, "png" | "raw"),
            color_management: "none",
            orientation: "stored_coordinates_no_exif_transform",
            frame_policy: "single_frame",
        }
    }
}

/// Dimensions declared by a parsed header; no claim about compressed payloads.
#[derive(Debug, Serialize)]
pub struct DeclaredGeometry {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub verification: &'static str,
}
pub(super) fn header_geometry(encoded: &[u8]) -> Result<Option<DeclaredGeometry>, OperationError> {
    if encoded.len() > super::MAX_ENCODED {
        return Err(super::limit("image header input exceeds 32 MiB"));
    }
    let (format, width, height) = if encoded.starts_with(b"\x89PNG\r\n\x1a\n") {
        let mut decoder = png::Decoder::new_with_limits(
            Cursor::new(encoded),
            png::Limits {
                bytes: 32 * 1024 * 1024,
            },
        );
        decoder.set_ignore_text_chunk(true);
        decoder.set_ignore_iccp_chunk(true);
        let header = decoder.read_header_info().map_err(super::decode_error)?;
        ("png", header.width, header.height)
    } else {
        let (format, name) = match raster_codecs::guess_format(encoded) {
            Ok(raster_codecs::ImageFormat::Jpeg) => (raster_codecs::ImageFormat::Jpeg, "jpeg"),
            Ok(raster_codecs::ImageFormat::Gif) => (raster_codecs::ImageFormat::Gif, "gif"),
            Ok(raster_codecs::ImageFormat::Bmp) => (raster_codecs::ImageFormat::Bmp, "bmp"),
            Ok(raster_codecs::ImageFormat::WebP) => (raster_codecs::ImageFormat::WebP, "webp"),
            _ => return Ok(None),
        };
        let mut reader = raster_codecs::ImageReader::with_format(Cursor::new(encoded), format);
        reader.limits(limits());
        let (width, height) = reader.into_dimensions().map_err(error)?;
        (name, width, height)
    };
    Ok(Some(DeclaredGeometry {
        format,
        width,
        height,
        verification: "header_only_no_payload_validation",
    }))
}
