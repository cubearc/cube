//! Explicit QR Model 2 segments, masks and optional raster output.
use super::{ColorModel, Image, invalid, limit, sample_count};
use crate::errors::OperationError;
use qrcodegen::{Mask, QrCode, QrCodeEcc, QrSegment, Version};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Ecc {
    L,
    M,
    Q,
    H,
}

impl From<Ecc> for QrCodeEcc {
    fn from(ecc: Ecc) -> Self {
        match ecc {
            Ecc::L => Self::Low,
            Ecc::M => Self::Medium,
            Ecc::Q => Self::Quartile,
            Ecc::H => Self::High,
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Segment {
    Numeric {
        #[schemars(length(max = 7089))]
        text: String,
    },
    Alphanumeric {
        #[schemars(length(max = 4296))]
        text: String,
    },
    Byte {
        #[schemars(length(max = 2953))]
        data: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SegmentInfo {
    pub mode: Mode,
    pub length: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub ecc: Ecc,
    #[schemars(range(min = 1, max = 40))]
    pub version: Option<u8>,
    #[schemars(range(max = 7))]
    pub mask: Option<u8>,
    pub raster: Option<Raster>,
}

fn quiet() -> u16 {
    4
}
fn scale() -> u8 {
    1
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Raster {
    #[serde(default = "quiet")]
    pub quiet_zone: u16,
    #[serde(default = "scale")]
    #[schemars(range(min = 1, max = 16))]
    pub scale: u8,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Penalty {
    pub mask: u8,
    pub runs: u32,
    pub blocks: u32,
    pub finder_like: u32,
    pub balance: u32,
    pub total: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub ecc: Ecc,
    pub version: u8,
    pub mask: u8,
    pub size: u32,
    pub segments: Vec<SegmentInfo>,
    pub penalties: Vec<Penalty>,
}

#[derive(Debug)]
pub struct EncodedQr {
    pub modules: Vec<u8>,
    pub report: Report,
    pub raster: Option<Image>,
}

fn segments(input: &[Segment]) -> Result<(Vec<QrSegment>, Vec<SegmentInfo>), OperationError> {
    if input.len() > 256 {
        return Err(limit("QR supports at most 256 explicit segments"));
    }
    let mut prepared = Vec::with_capacity(input.len());
    let mut info = Vec::with_capacity(input.len());
    let (mut total, mut byte_count) = (0, 0);
    for segment in input {
        let (mode, length) = match segment {
            Segment::Numeric { text } => (Mode::Numeric, text.len()),
            Segment::Alphanumeric { text } => (Mode::Alphanumeric, text.len()),
            Segment::Byte { data } => {
                if data.len() > 2953 - byte_count {
                    return Err(limit("QR exceeds the 2953 byte-mode ceiling"));
                }
                byte_count += data.len();
                (Mode::Byte, data.len())
            }
        };
        if length > 7089 - total {
            return Err(limit("QR exceeds the largest Model 2 segment capacity"));
        }
        total += length;
        prepared.push(match segment {
            Segment::Numeric { text } if text.bytes().all(|c| c.is_ascii_digit()) => {
                QrSegment::make_numeric(text)
            }
            Segment::Alphanumeric { text }
                if text
                    .bytes()
                    .all(|c| b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:".contains(&c)) =>
            {
                QrSegment::make_alphanumeric(text)
            }
            Segment::Byte { data } => QrSegment::make_bytes(data),
            _ => {
                return Err(invalid(
                    "QR segment does not match its explicit mode alphabet",
                ));
            }
        });
        info.push(SegmentInfo { mode, length });
    }
    Ok((prepared, info))
}

fn line_penalty(
    signal: impl Iterator<Item = bool>,
    side: usize,
    runs: &mut Vec<(bool, usize)>,
) -> (u32, u32) {
    runs.clear();
    for bit in signal {
        if let Some((last, length)) = runs.last_mut()
            && *last == bit
        {
            *length += 1;
        } else {
            runs.push((bit, 1));
        }
    }
    let run_cost = runs
        .iter()
        .map(|(_, length)| length.saturating_sub(4) + 2 * usize::from(*length >= 5))
        .sum::<usize>();
    if runs[0].0 {
        runs.insert(0, (false, side));
    } else {
        runs[0].1 += side;
    }
    let end = runs.last_mut().expect("nonempty module line");
    if end.0 {
        runs.push((false, side));
    } else {
        end.1 += side;
    }
    let finder_patterns = runs
        .windows(7)
        .filter(|r| !r[0].0)
        .map(|r| {
            let n = r[1].1;
            let core = [r[2].1, r[3].1, r[4].1, r[5].1] == [n, 3 * n, n, n];
            if core {
                u32::from(r[0].1 >= 4 * n && r[6].1 >= n)
                    + u32::from(r[6].1 >= 4 * n && r[0].1 >= n)
            } else {
                0
            }
        })
        .sum::<u32>();
    (run_cost as u32, 40 * finder_patterns)
}

fn penalty(matrix: &[u8], size: usize, mask: u8) -> Penalty {
    let mut runs = Vec::with_capacity(size + 2);
    let (mut run_cost, mut finder_like, mut blocks) = (0, 0, 0);
    for i in 0..size {
        let row = matrix[i * size..(i + 1) * size]
            .iter()
            .map(|&sample| sample == 1);
        let column = matrix[i..].iter().step_by(size).map(|&sample| sample == 1);
        for (r, f) in [
            line_penalty(row, size, &mut runs),
            line_penalty(column, size, &mut runs),
        ] {
            run_cost += r;
            finder_like += f;
        }
    }
    for y in 0..size - 1 {
        for x in 0..size - 1 {
            let i = y * size + x;
            let bit = matrix[i];
            blocks += 3 * u32::from(
                matrix[i + 1] == bit && matrix[i + size] == bit && matrix[i + size + 1] == bit,
            );
        }
    }
    let dark = matrix.iter().map(|&x| usize::from(x)).sum::<usize>();
    let balance = 10
        * (20 * dark)
            .abs_diff(10 * matrix.len())
            .div_ceil(matrix.len())
            .saturating_sub(1) as u32;
    Penalty {
        mask,
        runs: run_cost,
        blocks,
        finder_like,
        balance,
        total: run_cost + blocks + finder_like + balance,
    }
}

fn modules(symbol: &QrCode) -> Vec<u8> {
    (0..symbol.size())
        .flat_map(|y| (0..symbol.size()).map(move |x| u8::from(symbol.get_module(x, y))))
        .collect()
}

fn render(data: &[u8], side: u32, options: Raster) -> Result<Image, OperationError> {
    if !(1..=16).contains(&options.scale) {
        return Err(invalid("QR raster scale must be 1..16"));
    }
    let quiet = u32::from(options.quiet_zone);
    let scale = u32::from(options.scale);
    let width = (side + 2 * quiet) * scale;
    let count = sample_count(width, width, 1)?;
    let mut output = vec![255; count];
    for y in 0..side {
        for x in 0..side {
            if data[(y * side + x) as usize] == 1 {
                let x = (quiet + x) * scale;
                let row = (quiet + y) * scale;
                for y in row..row + scale {
                    let start = (y * width + x) as usize;
                    output[start..start + scale as usize].fill(0);
                }
            }
        }
    }
    Image::from_samples(width, width, ColorModel::Gray, output, vec![], vec![])
}

/// Encode explicit segments with no ECC boost or inferred text encoding.
/// Automatic masking scores all eight matrices and chooses the lowest mask
/// number among equal totals. A forced mask scores only that matrix.
pub fn encode(input: &[Segment], options: Options) -> Result<EncodedQr, OperationError> {
    (|| {
        if options.version.is_some_and(|v| !(1..=40).contains(&v))
            || options.mask.is_some_and(|v| v > 7)
        {
            return Err(invalid("QR version must be 1..40 and mask 0..7"));
        }
        let (segments, info) = segments(input)?;
        let first = Version::new(options.version.unwrap_or(1));
        let last = Version::new(options.version.unwrap_or(40));
        let (start, end) = options.mask.map_or((0, 7), |m| (m, m));
        let mut best = None;
        let mut penalties = Vec::with_capacity(usize::from(end - start + 1));
        for mask in start..=end {
            let symbol = QrCode::encode_segments_advanced(
                &segments,
                options.ecc.into(),
                first,
                last,
                Some(Mask::new(mask)),
                false,
            )
            .map_err(|_| limit("explicit QR segments exceed the selected Model 2 capacity"))?;
            let matrix = modules(&symbol);
            let score = penalty(&matrix, symbol.size() as usize, mask);
            if best
                .as_ref()
                .is_none_or(|&(previous, _, _, _, _)| score.total < previous)
            {
                best = Some((
                    score.total,
                    mask,
                    symbol.version().value(),
                    symbol.size() as u32,
                    matrix,
                ));
            }
            penalties.push(score);
        }
        let (_, mask, version, size, modules) = best.expect("at least one mask was scored");
        let raster = options
            .raster
            .map(|options| render(&modules, size, options))
            .transpose()?;
        Ok(EncodedQr {
            modules,
            report: Report {
                ecc: options.ecc,
                version,
                mask,
                size,
                segments: info,
                penalties,
            },
            raster,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.qr.encode"))
}
