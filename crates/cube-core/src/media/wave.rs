//! Seek-based RIFF/WAVE framing and integer PCM metadata, without sample decoding.
//! RIFF layout: <https://learn.microsoft.com/en-us/windows/win32/xaudio2/resource-interchange-file-format--riff->
use crate::{
    errors::{ErrorCode, OperationError},
    forensics,
};
use serde::Serialize;
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};
/// Maximum physical input file size; unknown/data payloads are skipped, not loaded.
pub const MAX_FILE_BYTES: u64 = 1024 * 1024 * 1024;
/// Maximum top-level chunks inspected per RIFF container.
pub const MAX_CHUNKS: usize = 10000;
/// Maximum PCM channels and sample rate in this deliberately small contract.
pub const MAX_CHANNELS: u16 = 32;
pub const MAX_SAMPLE_RATE: u32 = 384000;
pub(super) fn malformed(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
pub(super) fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}
fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message)
}
pub(super) fn read(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), OperationError> {
    reader.read_exact(bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            malformed("truncated WAVE input").caused_by(e)
        } else {
            OperationError::io(e)
        }
    })
}
/// Exact rational duration in seconds: numerator frames / denominator frames/sec.
#[derive(Clone, Debug, Serialize)]
pub struct Duration {
    pub numerator: u64,
    pub denominator: u32,
}
/// Top-level chunk framing. Offsets are absolute bytes; size excludes header/pad.
#[derive(Clone, Debug, Serialize)]
pub struct Chunk {
    pub id: String,
    pub offset: u64,
    pub data_offset: u64,
    pub size: u32,
    pub padding_bytes: u8,
}
/// Validated integer PCM layout. Physical channels have indexes, not guessed labels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Format {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub block_align: u16,
    pub byte_rate: u32,
}
impl Format {
    pub(super) fn validate(self) -> Result<Self, OperationError> {
        if self.format_tag != 1 {
            return Err(unsupported(
                "only integer PCM format tag 1 is supported; float, extensible and compressed WAVE are unsupported",
            ));
        }
        if ![8, 16, 24, 32].contains(&self.bits_per_sample) {
            return Err(unsupported(
                "only integer PCM widths 8, 16, 24 and 32 are supported",
            ));
        }
        if self.channels == 0 || self.sample_rate == 0 {
            return Err(malformed("channel count and sample rate must be positive"));
        }
        if self.channels > MAX_CHANNELS || self.sample_rate > MAX_SAMPLE_RATE {
            return Err(limit("PCM exceeds 32 channels or 384000 frames/sec"));
        }
        let align = self.channels * (self.bits_per_sample / 8);
        if self.block_align != align || self.byte_rate != self.sample_rate * u32::from(align) {
            return Err(malformed(
                "PCM block alignment or byte rate is inconsistent with format",
            ));
        }
        Ok(self)
    }
}
/// Framing facts separate from audio samples. Trailing bytes are reported, not decoded.
#[derive(Clone, Debug, Serialize)]
pub struct Info {
    pub container: &'static str,
    pub byte_order: &'static str,
    pub file_bytes: u64,
    pub container_end: u64,
    pub trailing_bytes: u64,
    pub format: Format,
    pub data_offset: u64,
    pub data_bytes: u64,
    pub frames: u64,
    pub duration_seconds: Duration,
    pub chunks: Vec<Chunk>,
}
fn le16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().expect("two-byte field"))
}
fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("four-byte field"))
}
/// Inspect one RIFF/WAVE container with exactly one fmt and data chunk. fmt must
/// precede data; unknown chunks may appear anywhere and are never buffered.
/// Accept 16-byte PCM fmt or 18-byte fmt with cbSize=0; reject ambiguous extensions,
/// RF64/RIFX, multiple data chunks and wavl. Enforce padding and container bounds.
/// Reads at most 120030 framing/fmt/LIST-kind bytes, independent of audio payload size.
pub fn inspect(reader: &mut (impl Read + Seek)) -> Result<Info, OperationError> {
    let result = (|| {
        let total = reader.seek(SeekFrom::End(0))?;
        if total > MAX_FILE_BYTES {
            return Err(limit("WAVE file exceeds 1 GiB"));
        }
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0; 12];
        read(reader, &mut header)?;
        if &header[..4] == b"RF64" || &header[..4] == b"RIFX" {
            return Err(unsupported("RF64 and big-endian RIFX are unsupported"));
        }
        if &header[..4] != b"RIFF" || &header[8..] != b"WAVE" {
            return Err(unsupported("expected little-endian RIFF/WAVE"));
        }
        let end = 8 + u64::from(le32(&header[4..8]));
        if end < 12 || end > total || !end.is_multiple_of(2) {
            return Err(malformed("invalid or truncated RIFF container length").offset(4));
        }
        let mut chunks = Vec::new();
        let mut format = None;
        let mut data = None;
        let mut offset = 12;
        while offset < end {
            if chunks.len() == MAX_CHUNKS {
                return Err(limit("RIFF exceeds 10000 chunks").offset(offset));
            }
            if end - offset < 8 {
                return Err(malformed("truncated RIFF chunk header").offset(offset));
            }
            reader.seek(SeekFrom::Start(offset))?;
            let mut chunk = [0; 8];
            read(reader, &mut chunk)?;
            if !chunk[..4].iter().all(|b| (0x20..=0x7e).contains(b)) {
                return Err(malformed("chunk ID must be four printable ASCII bytes").offset(offset));
            }
            let size = le32(&chunk[4..]);
            let data_offset = offset + 8;
            let next = data_offset + u64::from(size) + u64::from(size % 2);
            if next > end {
                return Err(
                    malformed("chunk payload or padding extends beyond RIFF boundary")
                        .offset(offset),
                );
            }
            match &chunk[..4] {
                b"fmt " => {
                    if format.is_some() {
                        return Err(malformed("duplicate fmt chunk").offset(offset));
                    }
                    if size < 16 {
                        return Err(
                            malformed("fmt chunk is shorter than PCM header").offset(offset)
                        );
                    }
                    let mut fields = [0; 16];
                    read(reader, &mut fields)?;
                    let value = Format {
                        format_tag: le16(&fields[..2]),
                        channels: le16(&fields[2..4]),
                        sample_rate: le32(&fields[4..8]),
                        byte_rate: le32(&fields[8..12]),
                        block_align: le16(&fields[12..14]),
                        bits_per_sample: le16(&fields[14..16]),
                    }
                    .validate()
                    .map_err(|e| e.offset(data_offset))?;
                    if size == 17 {
                        return Err(malformed("truncated fmt extension length").offset(offset));
                    }
                    if size != 16 {
                        if size != 18 {
                            return Err(unsupported(
                                "PCM fmt extensions beyond empty cbSize are unsupported",
                            )
                            .offset(offset));
                        }
                        let mut extra = [0; 2];
                        read(reader, &mut extra)?;
                        if le16(&extra) != 0 {
                            return Err(
                                unsupported("PCM fmt extension must have cbSize=0").offset(offset)
                            );
                        }
                    }
                    format = Some(value);
                }
                b"data" => {
                    if format.is_none() {
                        return Err(malformed("fmt must precede data").offset(offset));
                    }
                    if data.is_some() {
                        return Err(
                            unsupported("multiple data chunks are unsupported").offset(offset)
                        );
                    }
                    data = Some((data_offset, u64::from(size)));
                }
                b"LIST" if size >= 4 => {
                    let mut kind = [0; 4];
                    read(reader, &mut kind)?;
                    if &kind == b"wavl" {
                        return Err(unsupported("segmented wavl sample lists are unsupported")
                            .offset(offset));
                    }
                }
                _ => (),
            }
            chunks.push(Chunk {
                id: String::from_utf8(chunk[..4].to_vec()).expect("ASCII ID"),
                offset,
                data_offset,
                size,
                padding_bytes: (size % 2) as u8,
            });
            offset = next;
        }
        let format = format.ok_or_else(|| malformed("missing fmt chunk"))?;
        let (data_offset, data_bytes) = data.ok_or_else(|| malformed("missing data chunk"))?;
        if !data_bytes.is_multiple_of(u64::from(format.block_align)) {
            return Err(
                malformed("data length contains an incomplete PCM frame").offset(data_offset)
            );
        }
        let frames = data_bytes / u64::from(format.block_align);
        Ok(Info {
            container: "RIFF/WAVE",
            byte_order: "little",
            file_bytes: total,
            container_end: end,
            trailing_bytes: total - end,
            format,
            data_offset,
            data_bytes,
            frames,
            duration_seconds: Duration {
                numerator: frames,
                denominator: format.sample_rate,
            },
            chunks,
        })
    })();
    result.map_err(|e: OperationError| e.operation("audio.wave.inspect"))
}
/// Inspect a regular local file read-only; never follow a symlink or open a device.
pub fn inspect_file(path: &Path) -> Result<Info, OperationError> {
    (forensics::open_regular(path).and_then(|mut reader| inspect(&mut reader)))
        .map_err(|e| e.operation("audio.wave.inspect"))
}
/// Header discovery needs no multimedia backend or runtime dispatcher.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "audio.wave.inspect",
    category: "audio",
    description: "Inspect bounded PCM RIFF/WAVE metadata and chunks without decoding samples; <=1 GiB, <=10000 chunks.",
    input_type: "byte_reader",
    output_type: "audio.wave_info",
    parameters: &[],
    deterministic: true,
    reversible: None,
    command: "audio inspect",
    example: "cube audio inspect sample.wav --json",
}];
