//! Bounded QR recognition. Payloads stay bytes and are never opened or executed.
use super::{Channel, ColorModel, Image, invalid, limit, unsupported};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
pub const MAX_QR_PIXELS: u64 = 262144;
pub const MAX_QR_DIMENSION: u32 = 1024;
pub const MAX_QR_SYMBOLS: usize = 8;
pub const MAX_QR_PAYLOAD: usize = 65536;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QrStatus {
    NoDetection,
    DetectedUndecodable,
    Decoded,
    PartialDecode,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct QrSymbol {
    pub status: QrStatus,
    pub corners: Option<[[i32; 2]; 4]>,
    pub version: Option<usize>,
    pub ecc_level: Option<String>,
    pub mask: Option<i32>,
    pub data_type: Option<String>,
    pub eci_assignment: Option<i32>,
    pub error: Option<String>,
    #[serde(skip)]
    pub payload: Option<Vec<u8>>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct QrReport {
    pub status: QrStatus,
    pub symbols: Vec<QrSymbol>,
    pub decoded_bytes: usize,
    pub detector_limits_reached: bool,
    pub preprocessing: &'static str,
    pub limitations: &'static str,
}
impl Image {
    /// Decode QR symbols within a 262144-pixel, 1024-side, eight-symbol budget.
    /// Palette expansion and the existing integer luma conversion are applied to
    /// a temporary view. Nonopaque pixels fail rather than guessing a background.
    /// Quircs uses Otsu thresholding, <=32 capstones, <=65534 regions, <=8 grids
    /// and fixed QR version/error-correction bounds. No exhaustive-detection claim.
    pub fn qr(&self, max_symbols: usize) -> Result<QrReport, OperationError> {
        let result = (|| {
            if !(1..=MAX_QR_SYMBOLS).contains(&max_symbols) {
                return Err(invalid("max_symbols must be in 1..=8").parameter("max_symbols"));
            }
            if self.width > MAX_QR_DIMENSION
                || self.height > MAX_QR_DIMENSION
                || u64::from(self.width) * u64::from(self.height) > MAX_QR_PIXELS
            {
                return Err(limit(
                    "QR analysis exceeds 262144 pixels or 1024 pixels per side",
                ));
            }
            let view = self.view(self.model == ColorModel::Indexed)?;
            let gray = view.grayscale()?;
            if gray.model == ColorModel::GrayAlpha
                && gray.samples.as_chunks::<2>().0.iter().any(|p| p[1] != 255)
            {
                return Err(unsupported(
                    "QR analysis requires opaque samples; explicitly select/prepare an opaque channel first",
                ));
            }
            let pixels = gray.channel(Channel::Gray, None)?.data;
            let mut decoder = quircs::Quirc::default();
            let codes: Vec<_> = decoder
                .identify(self.width as usize, self.height as usize, &pixels)
                .collect();
            if codes.len() > max_symbols {
                return Err(limit("detected QR candidates exceed max_symbols"));
            }
            let detector_limits_reached = decoder.num_capstones() >= 32
                || decoder.num_regions() >= 65534
                || decoder.count() >= 8;
            let mut symbols = Vec::with_capacity(codes.len());
            let mut decoded_bytes = 0usize;
            let mut decoded = 0;
            for code in codes {
                let mut symbol = QrSymbol {
                    status: QrStatus::DetectedUndecodable,
                    corners: None,
                    version: None,
                    ecc_level: None,
                    mask: None,
                    data_type: None,
                    eci_assignment: None,
                    error: None,
                    payload: None,
                };
                match code {
                    Err(e) => symbol.error = Some(e.to_string()),
                    Ok(code) => {
                        symbol.corners = Some(code.corners.map(|p| [p.x, p.y]));
                        match code.decode() {
                            Err(e) => symbol.error = Some(e.to_string()),
                            Ok(data) => {
                                decoded_bytes = decoded_bytes
                                    .checked_add(data.payload.len())
                                    .ok_or_else(|| limit("QR payload size overflow"))?;
                                if decoded_bytes > MAX_QR_PAYLOAD {
                                    return Err(limit("QR payloads exceed 64 KiB"));
                                }
                                decoded += 1;
                                symbol.status = QrStatus::Decoded;
                                symbol.version = Some(data.version);
                                symbol.ecc_level = Some(format!("{:?}", data.ecc_level));
                                symbol.mask = Some(data.mask);
                                symbol.data_type = data.data_type.map(|v| v.to_string());
                                symbol.eci_assignment = data.eci.map(|v| v as i32);
                                symbol.payload = Some(data.payload);
                            }
                        }
                    }
                }
                symbols.push(symbol);
            }
            let status = if symbols.is_empty() {
                QrStatus::NoDetection
            } else if decoded == symbols.len() {
                QrStatus::Decoded
            } else if decoded == 0 {
                QrStatus::DetectedUndecodable
            } else {
                QrStatus::PartialDecode
            };
            Ok(QrReport {
                status,
                symbols,
                decoded_bytes,
                detector_limits_reached,
                preprocessing: "explicit_palette_expansion_integer_luma_opaque_only_otsu",
                limitations: "Detection is bounded and may miss symbols; decoded bytes do not establish authenticity. No URL following or text encoding guesses.",
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.qr.decode"))
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "image.qr.decode",
    category: "image",
    description: "Decode bounded QR symbols as raw bytes, separating no detection, undecodable candidates and decoded payloads.",
    input_type: "image.samples",
    output_type: "image.qr_report",
    parameters: &[Parameter {
        name: "max_symbols",
        description: "Maximum detected candidates; <=262144 pixels, <=1024 per side and <=64 KiB total decoded payload.",
        required: false,
        default: Some(ParameterValue::Unsigned(8)),
        constraints: ParameterType::Unsigned { min: 1, max: 8 },
    }],
    deterministic: true,
    reversible: None,
    command: "image qr",
    example: "cube image qr code.png --json",
}];
