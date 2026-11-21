//! Bounded native PNG scanlines and GIF palette indices from original bytes.
use super::{
    ColorModel, MAX_ENCODED, Region, invalid, limit, malformed, plane::MIB, sample_count,
    unsupported,
};
use crate::{errors::OperationError, forensics, hashing};
use flate2::{Decompress, FlushDecompress, Status};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, num::NonZeroU64};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PngProfile {
    Filtered,
    Reconstructed,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PngReport {
    pub profile: PngProfile,
    pub width: u32,
    pub height: u32,
    pub model: ColorModel,
    pub row_stride: usize,
    /// Offsets in the filtered stream, retained for either requested output.
    pub filter_offsets: Vec<usize>,
    pub source_sha256: String,
}

#[derive(Debug)]
pub struct PngScanlines {
    pub report: PngReport,
    pub data: Vec<u8>,
}

fn inflate(
    idat: impl Iterator<Item = impl AsRef<[u8]>>,
    expected: usize,
) -> Result<Vec<u8>, OperationError> {
    let mut decoder = Decompress::new(true);
    let mut output = vec![0; expected + 1];
    let mut status = Status::Ok;
    for part in idat {
        let mut source = part.as_ref();
        while !source.is_empty() {
            if status == Status::StreamEnd {
                return Err(malformed("trailing bytes in PNG IDAT compressed data"));
            }
            let (input_at, output_at) = (decoder.total_in(), decoder.total_out());
            status = decoder
                .decompress(
                    source,
                    &mut output[output_at as usize..],
                    FlushDecompress::None,
                )
                .map_err(|e| malformed("invalid PNG IDAT zlib stream").caused_by(e))?;
            if decoder.total_out() > expected as u64 {
                return Err(malformed("PNG expands past its declared scanline length"));
            }
            let consumed = (decoder.total_in() - input_at) as usize;
            if consumed == 0 && decoder.total_out() == output_at && status != Status::StreamEnd {
                return Err(malformed("PNG IDAT inflater did not advance"));
            }
            source = &source[consumed..];
        }
    }
    if status != Status::StreamEnd {
        status = decoder
            .decompress(
                &[],
                &mut output[decoder.total_out() as usize..],
                FlushDecompress::Finish,
            )
            .map_err(|error| malformed("incomplete PNG zlib trailer").caused_by(error))?;
    }
    if status != Status::StreamEnd || decoder.total_out() != expected as u64 {
        return Err(malformed(
            "PNG zlib stream must end at its exact scanline length",
        ));
    }
    output.truncate(expected);
    Ok(output)
}

fn paeth(left: u8, above: u8, diagonal: u8) -> u8 {
    let (a, b, c) = (i16::from(left), i16::from(above), i16::from(diagonal));
    let prediction = a + b - c;
    let (da, db, dc) = (
        (prediction - a).abs(),
        (prediction - b).abs(),
        (prediction - c).abs(),
    );
    if da <= db && da <= dc {
        left
    } else if db <= dc {
        above
    } else {
        diagonal
    }
}

fn reconstruct(
    filtered: &[u8],
    row_stride: usize,
    channels: usize,
    palette: Option<usize>,
) -> Result<Vec<u8>, OperationError> {
    let mut output = Vec::with_capacity(filtered.len() / (row_stride + 1) * row_stride);
    for (y, row) in filtered.chunks_exact(row_stride + 1).enumerate() {
        for (x, &sample) in row[1..].iter().enumerate() {
            let at = output.len();
            let left = if x >= channels {
                output[at - channels]
            } else {
                0
            };
            let above = if y > 0 { output[at - row_stride] } else { 0 };
            let upper_left = if y > 0 && x >= channels {
                output[at - row_stride - channels]
            } else {
                0
            };
            let correction = match row[0] {
                0 => 0,
                1 => left,
                2 => above,
                3 => (u16::from(left) + u16::from(above)).div_euclid(2) as u8,
                4 => paeth(left, above, upper_left),
                _ => return Err(malformed("unknown PNG scanline filter")),
            };
            let restored = sample.wrapping_add(correction);
            if palette.is_some_and(|size| usize::from(restored) >= size) {
                return Err(malformed("PNG index exceeds PLTE"));
            }
            output.push(restored);
        }
    }
    Ok(output)
}

/// Inflate the complete IDAT sequence with the strict shared framing proof.
/// Bytes stop at the exact expanded row length or its first extra byte.
pub fn png_scanlines(source: &[u8], profile: PngProfile) -> Result<PngScanlines, OperationError> {
    (|| {
        if source.len() > MAX_ENCODED {
            return Err(limit("original PNG exceeds 32 MiB"));
        }
        let report = forensics::png::inspect(&mut Cursor::new(source), true, 0)?;
        if report.boundary.trailing_bytes != 0
            || report
                .chunks
                .iter()
                .any(|row| matches!(row.kind.as_str(), "fcTL" | "fdAT"))
        {
            return Err(unsupported(
                "scanlines require one exact PNG without animation or trailing bytes",
            ));
        }
        let width = u32::from_be_bytes(source[16..20].try_into().expect("checked IHDR"));
        let height = u32::from_be_bytes(source[20..24].try_into().expect("checked IHDR"));
        if source[24] != 8 || source[28] != 0 {
            return Err(unsupported("scanlines require noninterlaced PNG8"));
        }
        let model = match source[25] {
            0 => ColorModel::Gray,
            2 => ColorModel::Rgb,
            3 => ColorModel::Indexed,
            4 => ColorModel::GrayAlpha,
            6 => ColorModel::Rgba,
            _ => unreachable!("strict PNG framing validates the color type"),
        };
        let channels = model.channels();
        let samples = sample_count(width, height, channels)?;
        let row_stride = width as usize * channels;
        let expanded = samples + height as usize;
        let visits = source.len()
            + expanded
            + if matches!(profile, PngProfile::Reconstructed) {
                samples
            } else {
                0
            };
        if visits > 64 * MIB {
            return Err(limit("PNG scanlines exceed 64 MiB framing and row work"));
        }
        let idat = report
            .chunks
            .iter()
            .filter(|row| row.kind == "IDAT")
            .map(|row| {
                let offset = row.offset as usize + 8;
                &source[offset..offset + row.length as usize]
            });
        let bytes = inflate(idat, expanded)?;
        let filter_offsets: Vec<_> = (0..height as usize)
            .map(|row| row * (row_stride + 1))
            .collect();
        if filter_offsets.iter().any(|&offset| bytes[offset] > 4) {
            return Err(malformed("unknown PNG scanline filter"));
        }
        let data = match profile {
            PngProfile::Filtered => bytes,
            PngProfile::Reconstructed => {
                let palette = (model == ColorModel::Indexed).then(|| {
                    report
                        .chunks
                        .iter()
                        .find(|row| row.kind == "PLTE")
                        .expect("checked PNG palette")
                        .length as usize
                        / 3
                });
                reconstruct(&bytes, row_stride, channels, palette)?
            }
        };
        Ok(PngScanlines {
            data,
            report: PngReport {
                profile,
                width,
                height,
                model,
                row_stride,
                filter_offsets,
                source_sha256: hashing::sha256(source),
            },
        })
    })()
    .map_err(|error: OperationError| error.operation("image.png.scanlines"))
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GifReport {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background_index: Option<usize>,
    pub frame_index: u32,
    pub frame: Region,
    pub interlaced: bool,
    pub local_palette: bool,
    pub palette: Vec<[u8; 3]>,
    pub transparent_index: Option<u8>,
    pub source_sha256: String,
}

#[derive(Debug)]
pub struct GifIndices {
    pub report: GifReport,
    pub indices: Vec<u8>,
}

fn gif_error(error: gif::DecodingError) -> OperationError {
    match error {
        gif::DecodingError::MemoryLimit | gif::DecodingError::OutOfMemory => {
            limit("GIF codec memory budget exceeded")
        }
        _ => malformed("GIF palette or indexed LZW frame is invalid"),
    }
    .caused_by(error)
}

/// Decode one selected GIF frame in indexed mode. Keep its active palette,
/// offset, transparency and source hash separately from the canvas.
pub fn gif_indices(source: &[u8], frame_index: u32) -> Result<GifIndices, OperationError> {
    (|| {
        if source.len() > MAX_ENCODED {
            return Err(limit("original GIF exceeds 32 MiB"));
        }
        if frame_index != 0 {
            return Err(invalid(
                "the single-frame GIF profile supports frame index zero",
            ));
        }
        let boundary = forensics::boundaries::single_gif(source)?;
        if boundary.trailing_bytes != 0 {
            return Err(malformed("trailing bytes after GIF trailer"));
        }
        let mut settings = gif::DecodeOptions::new();
        settings.set_color_output(gif::ColorOutput::Indexed);
        settings.set_memory_limit(gif::MemoryLimit::Bytes(
            NonZeroU64::new(8 * MIB as u64).unwrap(),
        ));
        settings.check_frame_consistency(true);
        settings.check_lzw_end_code(true);
        let mut decoder = settings.read_info(Cursor::new(source)).map_err(gif_error)?;
        let (canvas_width, canvas_height) =
            (u32::from(decoder.width()), u32::from(decoder.height()));
        let background_index = decoder.bg_color();
        let info = decoder
            .next_frame_info()
            .map_err(gif_error)?
            .ok_or_else(|| malformed("missing GIF frame"))?;
        let frame = Region {
            x: u32::from(info.left),
            y: u32::from(info.top),
            width: u32::from(info.width),
            height: u32::from(info.height),
        };
        let (interlaced, local_palette, transparent_index) =
            (info.interlaced, info.palette.is_some(), info.transparent);
        let count = sample_count(frame.width, frame.height, 1)?;
        let palette: Vec<_> = decoder
            .palette()
            .map_err(gif_error)?
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2]])
            .collect();
        let mut indices = vec![0; count];
        decoder.read_into_buffer(&mut indices).map_err(gif_error)?;
        if decoder.fill_buffer(&mut [0]).map_err(gif_error)? {
            return Err(malformed("GIF LZW expands past the frame size"));
        }
        if indices
            .iter()
            .any(|&index| usize::from(index) >= palette.len())
            || decoder.next_frame_info().map_err(gif_error)?.is_some()
        {
            return Err(malformed(
                "GIF requires exactly one frame with valid palette indices",
            ));
        }
        Ok(GifIndices {
            indices,
            report: GifReport {
                canvas_width,
                canvas_height,
                background_index,
                frame_index,
                frame,
                interlaced,
                local_palette,
                palette,
                transparent_index,
                source_sha256: hashing::sha256(source),
            },
        })
    })()
    .map_err(|error: OperationError| error.operation("image.gif.indices"))
}
