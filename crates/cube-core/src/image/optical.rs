//! Narrow Data Matrix ECC200, Aztec and PDF417 crop readers.
use super::{ColorModel, Image, invalid, limit, plane::MIB, resample};
use crate::errors::OperationError;
use rxing::{
    BinaryBitmap, DecodeHints, Exceptions, Luma8Source, RXingResult,
    RXingResultMetadataType as Key, RXingResultMetadataValue as Metadata, Reader,
    aztec::AztecReader, common::HybridBinarizer, datamatrix::DataMatrixReader,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

mod pdf417;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    DataMatrix,
    Aztec,
    Pdf417,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Orientation {
    #[serde(rename = "0")]
    Deg0,
    #[serde(rename = "90")]
    Deg90,
    #[serde(rename = "180")]
    Deg180,
    #[serde(rename = "270")]
    Deg270,
}

impl Orientation {
    fn degrees(self) -> u32 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }

    fn original(self, [x, y]: [f32; 2], w: u32, h: u32) -> [f32; 2] {
        let (w, h) = ((w - 1) as f32, (h - 1) as f32);
        match self {
            Self::Deg0 => [x, y],
            Self::Deg90 => [y, h - x],
            Self::Deg180 => [w - x, h - y],
            Self::Deg270 => [w - y, x],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub format: Format,
    pub orientation: Orientation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    NoDetection,
    DetectedUndecodable,
    Decoded,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Crop {
    pub status: Status,
    pub text: Option<String>,
    /// Decoder-provided raw bytes, which may be corrected symbol data rather than text.
    pub raw_bytes: Option<Vec<u8>>,
    pub raw_bits: Option<usize>,
    pub byte_segments: Vec<Vec<u8>>,
    pub codewords: Option<Vec<u32>>,
    pub corrected_errors: Option<usize>,
    pub correction_level: Option<String>,
    pub symbology_identifier: Option<String>,
    pub points: Vec<[f32; 2]>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub options: Options,
    pub charged_steps: usize,
    pub crops: Vec<Crop>,
}

fn charge(image: &Image, format: Format) -> Result<usize, OperationError> {
    resample::physical_samples(image)?;
    if image.model() != ColorModel::Gray {
        return Err(invalid(
            "2D barcode crops require explicit opaque gray samples",
        ));
    }
    let (w, h) = (image.width() as usize, image.height() as usize);
    if w > 1024 || h > 1024 || w * h > 262_144 {
        return Err(limit("barcode crop exceeds side 1024 or 262144 pixels"));
    }
    Ok(match format {
        Format::DataMatrix => 24 * w * h + 2_500_000,
        Format::Aztec => 32 * w * h + 3_000_000,
        Format::Pdf417 => 32 * w * h,
    })
}

fn blank(status: Status, points: Vec<[f32; 2]>) -> Crop {
    Crop {
        status,
        points,
        text: None,
        raw_bytes: None,
        raw_bits: None,
        byte_segments: vec![],
        codewords: None,
        corrected_errors: None,
        correction_level: None,
        symbology_identifier: None,
    }
}

fn rectangle([x, y, w, h]: [u32; 4]) -> Vec<[f32; 2]> {
    [
        [x, y],
        [x + w - 1, y],
        [x + w - 1, y + h - 1],
        [x, y + h - 1],
    ]
    .map(|point| point.map(|v| v as f32))
    .to_vec()
}

fn decode_crop(
    original: &Image,
    options: Options,
    remaining: &mut usize,
) -> Result<Crop, OperationError> {
    let rotated = match options.orientation {
        Orientation::Deg0 => Cow::Borrowed(original),
        other => Cow::Owned(original.rotate(other.degrees())?),
    };
    let lines = Luma8Source::new_with_slice(rotated.samples(), rotated.width(), rotated.height())
        .map_err(|_| invalid("invalid optical crop bytes"))?;
    let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(lines));
    let Some(bounds) = bitmap.get_black_matrix().getEnclosingRectangle() else {
        return Ok(blank(Status::NoDetection, vec![]));
    };
    let hints = DecodeHints {
        PureBarcode: matches!(options.format, Format::DataMatrix).then_some(true),
        TryHarder: Some(false),
        ..DecodeHints::default()
    };
    let decode_result = |result| match result {
        Ok(result) => record(result, options.format, bounds),
        Err(Exceptions::NotFoundException(_)) => Ok(blank(Status::NoDetection, vec![])),
        Err(_) => Ok(blank(Status::DetectedUndecodable, rectangle(bounds))),
    };
    let mut crop = match options.format {
        Format::DataMatrix => {
            decode_result(DataMatrixReader.decode_with_hints(&mut bitmap, &hints))?
        }
        Format::Aztec => decode_result(AztecReader.decode_with_hints(&mut bitmap, &hints))?,
        Format::Pdf417 => pdf417::decode(bitmap.get_black_matrix(), bounds, remaining)?,
    };
    for point in &mut crop.points {
        *point = options
            .orientation
            .original(*point, original.width(), original.height());
    }
    Ok(crop)
}

fn record(result: RXingResult, format: Format, bounds: [u32; 4]) -> Result<Crop, OperationError> {
    let meta = result.getRXingResultMetadata();
    let raw = result.getRawBytes();
    if meta.contains_key(&Key::STRUCTURED_APPEND_SEQUENCE)
        || (matches!(format, Format::DataMatrix) && raw.first() == Some(&233))
    {
        return Ok(blank(Status::DetectedUndecodable, vec![]));
    }
    let mut points: Vec<_> = result.getPoints().iter().map(|p| [p.x, p.y]).collect();
    if points.is_empty() {
        points = rectangle(bounds);
    }
    if points.len() > 8 || points.iter().flatten().any(|point| !point.is_finite()) {
        return Ok(blank(Status::DetectedUndecodable, vec![]));
    }
    let byte_segments = match meta.get(&Key::BYTE_SEGMENTS) {
        Some(Metadata::ByteSegments(segments)) => segments.clone(),
        _ => vec![],
    };
    let size =
        result.getText().len() + raw.len() + byte_segments.iter().map(Vec::len).sum::<usize>();
    if size > 64 * 1024 {
        return Err(limit("barcode decoded material exceeds 64 KiB"));
    }
    let correction_level = match meta.get(&Key::ERROR_CORRECTION_LEVEL) {
        Some(Metadata::ErrorCorrectionLevel(level)) => Some(level.clone()),
        _ => None,
    };
    let symbology_identifier = match meta.get(&Key::SYMBOLOGY_IDENTIFIER) {
        Some(Metadata::SymbologyIdentifier(id)) => Some(id.clone()),
        _ => None,
    };
    Ok(Crop {
        status: Status::Decoded,
        points,
        text: Some(result.getText().to_owned()),
        raw_bytes: (!raw.is_empty()).then(|| raw.to_vec()),
        raw_bits: (!raw.is_empty()).then(|| result.getNumBits()),
        byte_segments,
        corrected_errors: None,
        codewords: None,
        correction_level,
        symbology_identifier,
    })
}

/// Read exactly one candidate in each of at most eight caller-selected crops.
/// Raw code data, byte segments and decoded text are separate representations.
pub fn decode(crops: &[&Image], options: Options) -> Result<Report, OperationError> {
    (|| {
        if crops.len() > 8 {
            return Err(limit(
                "optical analysis supports at most eight explicit crops",
            ));
        }
        let mut charged_steps = 0;
        for image in crops {
            charged_steps += charge(image, options.format)?;
            if charged_steps > 32 * MIB {
                return Err(limit(
                    "optical batch exceeds 32 Mi candidate and decoder work units",
                ));
            }
        }
        let mut decoded = Vec::with_capacity(crops.len());
        let mut remaining = 32 * MIB - charged_steps;
        let mut bytes = 0;
        for image in crops {
            let result = decode_crop(image, options, &mut remaining)?;
            bytes += result.text.as_ref().map_or(0, String::len)
                + result.raw_bytes.as_ref().map_or(0, Vec::len)
                + result.codewords.as_ref().map_or(0, |codes| 4 * codes.len())
                + result.byte_segments.iter().map(Vec::len).sum::<usize>();
            if bytes > 64 * 1024 {
                return Err(limit("optical batch exceeds 64 KiB decoded material"));
            }
            decoded.push(result);
        }
        Ok(Report {
            options,
            charged_steps: 32 * MIB - remaining,
            crops: decoded,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.barcode.decode"))
}
