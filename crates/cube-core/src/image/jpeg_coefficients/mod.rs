//! Read-only evidence from stored quantized baseline JPEG AC coefficients.
mod catalog;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

use crate::errors::{ErrorCode, OperationError};
use jpxl_jpeg::{JpegError, Limits, units::ZIGZAG_TO_NATURAL};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const EXTRACT: &str = "image.jpeg.coefficients.extract";
pub const MAX_INPUT: usize = 16 << 20;
pub const MAX_COEFFICIENTS: usize = 1 << 20;
pub const MAX_BITS: usize = MAX_COEFFICIENTS;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// Magnitude parity of nonzero AC coefficients in baseline JPEG.
    BaselineAcParity,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub profile: Profile,
    pub component_id: u8,
    #[schemars(range(max = 1048576))]
    pub start_bit: u32,
    #[schemars(range(min = 1, max = 1048576))]
    pub max_bits: u32,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Report {
    pub width: u16,
    pub height: u16,
    pub component_id: u8,
    pub block_columns: u32,
    pub block_rows: u32,
    pub start_bit: u32,
    pub candidate_bits: u32,
    pub bit_count: u32,
}

/// Exact bits, packed MSB-first and zero-padded in the last byte, with evidence
/// for which positions and JPEG component were read.
#[derive(Debug)]
pub struct Extraction {
    pub bits: Vec<u8>,
    pub report: Report,
}

fn error(code: ErrorCode, detail: &'static str) -> OperationError {
    OperationError::new(code, detail).operation(EXTRACT)
}
fn malformed() -> OperationError {
    error(
        ErrorCode::InvalidInput,
        "invalid complete JPEG marker, scan, or coefficient data",
    )
}
fn unsupported() -> OperationError {
    error(
        ErrorCode::UnsupportedFormat,
        "use an eight-bit baseline JPEG with one complete scan and one or three components",
    )
}
fn limits() -> OperationError {
    error(
        ErrorCode::ResourceLimit,
        "JPEG exceeds its input, marker, table, or coefficient bound",
    )
}
fn backend(e: JpegError) -> OperationError {
    match e {
        JpegError::LimitExceeded(_) => limits(),
        JpegError::Unsupported(_) => unsupported(),
        _ => malformed(),
    }
}

// Bound marker/table allocation and verify the one-scan profile before calling
// the coefficient decoder. The backend validates the entropy coding itself.
fn preflight(data: &[u8]) -> Result<(), OperationError> {
    if data.len() > MAX_INPUT {
        return Err(limits());
    }
    let mut headers = data.strip_prefix(&[0xff, 0xd8]).ok_or_else(malformed)?;
    let mut components = None;
    let mut tables = 0usize;
    let mut required_quantization = 0u8;
    let mut quantization = 0u8;
    for _ in 0..4096 {
        let &[0xff, marker, high, low, ref rest @ ..] = headers else {
            return Err(malformed());
        };
        let length = usize::from(u16::from_be_bytes([high, low]))
            .checked_sub(2)
            .ok_or_else(malformed)?;
        let (payload, next) = rest.split_at_checked(length).ok_or_else(malformed)?;
        match marker {
            0xc0 => {
                if components.is_some() {
                    return Err(malformed());
                }
                let [8, _, _, _, _, count @ (1 | 3), ..] = payload else {
                    return Err(unsupported());
                };
                let count = *count;
                if payload.len() != 6 + usize::from(count) * 3 {
                    return Err(malformed());
                }
                for [_, _, table] in payload[6..].as_chunks::<3>().0 {
                    if *table > 3 {
                        return Err(malformed());
                    }
                    required_quantization |= 1 << table;
                }
                components = Some(count);
            }
            0xc4 | 0xdb => {
                tables += payload.len();
                if tables > 8192 {
                    return Err(limits());
                }
                if marker == 0xdb {
                    let (entries, remainder) = payload.as_chunks::<65>();
                    if !remainder.is_empty() || entries.is_empty() {
                        return Err(malformed());
                    }
                    for table in entries {
                        if table[0] > 3 || table[1..].contains(&0) {
                            return Err(malformed());
                        }
                        quantization |= 1 << table[0];
                    }
                }
            }
            0xdd | 0xfe | 0xe0..=0xef => (),
            0xda => {
                let count = components.ok_or_else(malformed)?;
                if (required_quantization & quantization) != required_quantization {
                    return Err(malformed());
                }
                if payload.first() != Some(&count)
                    || payload.len() != 1 + usize::from(count) * 2 + 3
                    || !payload.ends_with(&[0, 63, 0])
                {
                    return Err(unsupported());
                }
                return preflight_scan(next);
            }
            _ => return Err(unsupported()),
        }
        headers = next;
    }
    Err(limits())
}

fn preflight_scan(mut bytes: &[u8]) -> Result<(), OperationError> {
    let mut restarts = 0;
    while let Some(index) = bytes.iter().position(|&byte| byte == 0xff) {
        let marker = &bytes[index + 1..];
        let Some((&code, tail)) = marker.split_first() else {
            break;
        };
        match code {
            0 => (),
            0xd0..=0xd7 => {
                restarts += 1;
                if restarts > MAX_COEFFICIENTS / 64 {
                    return Err(limits());
                }
            }
            0xd9 if tail.is_empty() => return Ok(()),
            0xd9 => return Err(malformed()),
            _ => return Err(unsupported()),
        }
        bytes = tail;
    }
    Err(malformed())
}

/// Extract magnitude parity for an exact JPEG component. Visit real blocks
/// in row-major order, then traverse AC coefficients in JPEG zigzag order.
/// Skip zero coefficients and whole MCU padding blocks. No raster conversion
/// or source writes are performed.
pub fn extract(data: &[u8], settings: Settings) -> Result<Extraction, OperationError> {
    let Settings {
        profile: Profile::BaselineAcParity,
        component_id,
        start_bit,
        max_bits,
    } = settings;
    if max_bits == 0 || max_bits > MAX_BITS as u32 {
        return Err(error(
            ErrorCode::InvalidParameter,
            "max_bits must be between 1 and 1048576",
        )
        .parameter("max_bits"));
    }
    if start_bit > MAX_BITS as u32 {
        return Err(
            error(ErrorCode::InvalidParameter, "start_bit exceeds 1048576").parameter("start_bit"),
        );
    }
    preflight(data)?;
    let jpeg = jpxl_jpeg::parse_with_limits(
        data,
        &Limits {
            max_dimension: u32::from(u16::MAX),
            max_total_blocks: (MAX_COEFFICIENTS / 64) as u64,
            max_input_len: MAX_INPUT,
        },
    )
    .map_err(backend)?;
    let frame = jpeg.frame.as_ref().ok_or_else(malformed)?;
    let selected = frame
        .components
        .iter()
        .position(|component| component.id.0 == component_id)
        .ok_or_else(|| {
            error(
                ErrorCode::InvalidParameter,
                "component ID is absent from the JPEG",
            )
            .parameter("component_id")
        })?;
    let geometry = frame.geometry().map_err(backend)?;
    let dims = frame.component_dims(selected, &geometry).map_err(backend)?;
    let plane = jpeg.planes.get(selected).ok_or_else(malformed)?;
    let cols = dims.blocks_per_line_noninterleaved;
    let rows = dims.block_rows_noninterleaved;

    let mut bit_count = 0;
    let mut candidate_bits = 0;
    let mut bits = Vec::with_capacity((max_bits as usize).div_ceil(8));
    for y in 0..rows {
        for x in 0..cols {
            let block = plane.block(x, y).ok_or_else(malformed)?;
            for index in &ZIGZAG_TO_NATURAL[1..] {
                let value = block[usize::from(*index)];
                if value == 0 {
                    continue;
                }
                if candidate_bits >= start_bit && bit_count < max_bits {
                    if bit_count % 8 == 0 {
                        bits.push(0);
                    }
                    bits[bit_count as usize / 8] |=
                        (value.unsigned_abs() as u8 & 1) << (7 - bit_count % 8);
                    bit_count += 1;
                }
                candidate_bits += 1;
            }
        }
    }
    if start_bit > candidate_bits {
        return Err(error(
            ErrorCode::InvalidParameter,
            "start_bit exceeds available coefficient parity positions",
        )
        .parameter("start_bit"));
    }
    Ok(Extraction {
        bits,
        report: Report {
            width: frame.width,
            height: frame.height,
            component_id,
            block_columns: cols as u32,
            block_rows: rows as u32,
            start_bit,
            candidate_bits,
            bit_count,
        },
    })
}
