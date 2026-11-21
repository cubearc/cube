//! Explicit one-dimensional barcode scans over bounded, inert samples.
use super::{ColorModel, Image, invalid, limit, plane::MIB, resample, transform::rgb_luma};
use crate::errors::OperationError;
use rxing::{
    DecodeHints, Exceptions, RXingResult,
    common::BitArray,
    oned::{Code39Reader, Code128Reader, EAN13Reader, OneDReader, UPCAReader},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Code39Checksum {
    None,
    Mod43,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Symbology {
    Ean13,
    Upca,
    Code39 { checksum: Code39Checksum },
    Code128,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    Forward,
    Reverse,
}

fn middle() -> u8 {
    128
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScanOptions {
    pub symbology: Symbology,
    pub orientation: Orientation,
    #[serde(default = "middle")]
    pub threshold: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    NoDetection,
    Invalid,
    Decoded,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Checksum {
    pub verified: bool,
    pub value: Option<u8>,
    pub algorithm: &'static str,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Scan {
    pub row: u32,
    pub status: Status,
    pub payload: Option<Vec<u8>>,
    /// Includes the start, checksum and stop Code 128 codewords.
    pub codewords: Option<Vec<u8>>,
    pub checksum: Option<Checksum>,
    pub start: Option<f32>,
    pub stop: Option<f32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ScanReport {
    pub options: ScanOptions,
    pub charged_steps: usize,
    pub rows: Vec<Scan>,
}

fn blank(row: u32, status: Status) -> Scan {
    Scan {
        row,
        status,
        payload: None,
        codewords: None,
        checksum: None,
        start: None,
        stop: None,
    }
}

fn decode(bits: &BitArray, row: u32, profile: Symbology) -> Result<RXingResult, Exceptions> {
    let hints = DecodeHints::default();
    match profile {
        Symbology::Ean13 => EAN13Reader.decode_row(row, bits, &hints),
        Symbology::Upca => UPCAReader::default().decode_row(row, bits, &hints),
        Symbology::Code39 { checksum } => {
            Code39Reader::with_use_check_digit(matches!(checksum, Code39Checksum::Mod43))
                .decode_row(row, bits, &hints)
        }
        Symbology::Code128 => Code128Reader.decode_row(row, bits, &hints),
    }
}

fn record(result: RXingResult, row: u32, width: usize, settings: ScanOptions) -> Scan {
    let mut scan = blank(row, Status::Decoded);
    let position = |x| match settings.orientation {
        Orientation::Forward => x,
        Orientation::Reverse => (width - 1) as f32 - x,
    };
    if let [a, b, ..] = result.getPoints() {
        let (a, b) = (position(a.x), position(b.x));
        scan.start = Some(a.min(b));
        scan.stop = Some(a.max(b));
    }
    let payload: Option<Vec<u8>> = result
        .getText()
        .chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect();
    let Some(payload) = payload else {
        return blank(row, Status::Invalid);
    };
    scan.checksum = match settings.symbology {
        Symbology::Ean13 | Symbology::Upca => Some(Checksum {
            verified: true,
            value: payload.last().and_then(|&b| b.checked_sub(b'0')),
            algorithm: "mod10",
        }),
        Symbology::Code39 {
            checksum: Code39Checksum::Mod43,
        } => {
            const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ-. $/+%";
            let value = payload
                .iter()
                .filter_map(|b| ALPHABET.iter().position(|a| a == b))
                .sum::<usize>()
                % 43;
            Some(Checksum {
                verified: true,
                value: Some(value as u8),
                algorithm: "mod43",
            })
        }
        Symbology::Code39 {
            checksum: Code39Checksum::None,
        } => None,
        Symbology::Code128 => {
            let codes = result.getRawBytes();
            let value = codes
                .len()
                .checked_sub(2)
                .and_then(|i| codes.get(i))
                .copied();
            scan.codewords = Some(codes.to_vec());
            Some(Checksum {
                verified: true,
                value,
                algorithm: "mod103",
            })
        }
    };
    scan.payload = Some(payload);
    scan
}

fn scan<'a>(
    rows: impl ExactSizeIterator<Item = (u32, &'a [u8])> + Clone,
    settings: ScanOptions,
) -> Result<ScanReport, OperationError> {
    if rows.len() > 1024 {
        return Err(limit("barcode batch exceeds 1024 lines"));
    }
    let (mut samples, mut steps) = (0usize, 0usize);
    for (_, row) in rows.clone() {
        if row.is_empty() || row.len() > 4096 {
            return Err(invalid("scanlines must have 1..4096 samples"));
        }
        let transitions = row
            .windows(2)
            .filter(|p| (p[0] < settings.threshold) != (p[1] < settings.threshold))
            .count();
        // Each reader scans linear runs and compares a fixed codebook: at most
        // 107 candidate patterns per Code 128 symbol. Charge conservatively for
        // scan passes, start candidates and fixed-width pattern comparisons.
        samples += row.len();
        steps += 16 * row.len() + 128 * transitions;
        if samples > MIB || steps > 16 * MIB {
            return Err(limit(
                "barcode batch exceeds its 1 Mi sample or 16 Mi decoder work ceiling",
            ));
        }
    }
    let mut output = Vec::with_capacity(rows.len());
    let (mut symbols, mut bytes) = (0, 0);
    for (y, row) in rows {
        let mut bits = BitArray::with_size(row.len());
        for (x, &sample) in row.iter().enumerate() {
            if sample < settings.threshold {
                let i = match settings.orientation {
                    Orientation::Forward => x,
                    Orientation::Reverse => row.len() - 1 - x,
                };
                bits.set(i);
            }
        }
        let result = match decode(&bits, y, settings.symbology) {
            Ok(result) => record(result, y, row.len(), settings),
            Err(Exceptions::NotFoundException(_)) => blank(y, Status::NoDetection),
            Err(_) => blank(y, Status::Invalid),
        };
        if let Some(data) = &result.payload {
            symbols += 1;
            bytes += data.len();
            if symbols > 64 || bytes > 64 * 1024 {
                return Err(limit(
                    "barcode batch exceeds 64 symbols or 64 KiB decoded payload",
                ));
            }
        }
        output.push(result);
    }
    Ok(ScanReport {
        options: settings,
        charged_steps: steps,
        rows: output,
    })
}

/// Decode exactly the supplied lines in the caller's chosen orientation.
/// Dark samples are strictly below the selected threshold.
pub fn scanlines(rows: &[Vec<u8>], settings: ScanOptions) -> Result<ScanReport, OperationError> {
    scan(
        rows.iter()
            .enumerate()
            .map(|(i, row)| (i as u32, row.as_slice())),
        settings,
    )
    .map_err(|error| error.operation("image.barcode.scan"))
}

/// Decode only the requested rows of one opaque gray or RGB image.
pub fn image_rows(
    image: &Image,
    rows: &[u32],
    settings: ScanOptions,
) -> Result<ScanReport, OperationError> {
    (|| {
        resample::physical_samples(image)?;
        let gray = match image.model() {
            ColorModel::Gray => true,
            ColorModel::Rgb => false,
            _ => return Err(invalid("barcode image must be opaque gray or RGB")),
        };
        let w = image.width() as usize;
        if rows.len() > 1024 || w > 4096 || rows.len() * w > MIB {
            return Err(limit(
                "requested image scanlines exceed the shape or sample budget",
            ));
        }
        if rows.iter().any(|&y| y >= image.height()) {
            return Err(invalid("requested barcode row is outside the image"));
        }
        let data = image.samples();
        if gray {
            scan(
                rows.iter()
                    .map(|&y| (y, &data[y as usize * w..(y as usize + 1) * w])),
                settings,
            )
        } else {
            let extracted: Vec<Vec<u8>> = rows
                .iter()
                .map(|&y| {
                    data[y as usize * w * 3..(y as usize + 1) * w * 3]
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .map(|rgb| rgb_luma(rgb))
                        .collect()
                })
                .collect();
            scan(
                rows.iter()
                    .copied()
                    .zip(extracted.iter().map(Vec::as_slice)),
                settings,
            )
        }
    })()
    .map_err(|error: OperationError| error.operation("image.barcode.scan"))
}
