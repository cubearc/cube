//! PNG framing and CRC evidence, shared by boundary detection and future chunk inspection.

use super::{MAX_SCAN_BYTES, limit, malformed, size};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

fn read(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), OperationError> {
    reader.read_exact(bytes).map_err(|error| {
        if error.kind() == std::io::ErrorKind::UnexpectedEof {
            malformed("truncated PNG framing").caused_by(error)
        } else {
            OperationError::io(error)
        }
    })
}

/// A validated PNG framing boundary, not proof that compressed pixels decode.
#[derive(Clone, Debug, Serialize)]
pub struct Boundary {
    pub format: &'static str,
    pub logical_end: u64,
    pub trailing_offset: u64,
    pub trailing_bytes: u64,
    pub validated_chunks: usize,
    pub evidence: &'static str,
}

/// Exact chunk framing and CRC evidence. Payload hex is opt-in and budgeted.
#[derive(Clone, Debug, Serialize)]
pub struct Chunk {
    pub kind: String,
    pub offset: u64,
    pub length: u32,
    pub stored_crc: u32,
    pub calculated_crc: u32,
    pub valid: bool,
    pub payload_hex: Option<String>,
}
/// PNG framing report; valid CRCs do not prove that compressed pixels decode.
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub boundary: Boundary,
    pub chunks: Vec<Chunk>,
    pub crc_valid: bool,
}

/// Inspect basic PNG framing without pixel decoding. Strict mode rejects any CRC
/// mismatch; inspection mode reports it and continues only within checked lengths.
/// Structural errors, unknown critical chunks and APNG fail in either mode.
/// Processing is capped at 64 MiB before IEND and 10000 chunks. Payload export is
/// disabled by zero; a positive total payload budget must not exceed 1 MiB.
pub fn inspect(
    reader: &mut (impl Read + Seek),
    strict: bool,
    payload_budget: usize,
) -> Result<Report, OperationError> {
    let result = (|| {
        if payload_budget > 1024 * 1024 {
            return Err(limit("PNG payload export budget exceeds 1 MiB"));
        }
        let mut exported = 0usize;
        let mut records = Vec::new();
        let mut crc_valid = true;
        let total = size(reader)?;
        reader.seek(SeekFrom::Start(0))?;
        let mut signature = [0; 8];
        read(reader, &mut signature)?;
        if &signature != b"\x89PNG\r\n\x1a\n" {
            return Err(OperationError::new(
                ErrorCode::UnsupportedFormat,
                "PNG framing inspection supports PNG only",
            ));
        }
        let mut offset = 8u64;
        let mut chunks = 0;
        let mut palette = false;
        let mut idat = false;
        let mut idat_ended = false;
        let mut indexed = false;
        let mut color_model = 0;
        let mut sample_depth = 0;
        loop {
            chunks += 1;
            if chunks > 10_000 {
                return Err(limit("PNG chunk count exceeds 10000"));
            }
            if offset + 12 > total {
                return Err(malformed("missing or truncated PNG IEND").offset(offset));
            }
            let mut header = [0; 8];
            read(reader, &mut header)?;
            let length = u32::from_be_bytes(header[..4].try_into().expect("fixed length")) as u64;
            let kind = &header[4..8];
            let end = offset
                .checked_add(length + 12)
                .ok_or_else(|| malformed("PNG length overflow"))?;
            if end > total {
                return Err(malformed("PNG chunk extends beyond EOF").offset(offset));
            }
            if end > MAX_SCAN_BYTES {
                return Err(limit("PNG boundary scan exceeds 64 MiB").offset(offset));
            }
            if length > 0x7fff_ffff
                || !kind.iter().all(u8::is_ascii_alphabetic)
                || !kind[2].is_ascii_uppercase()
            {
                return Err(malformed("invalid PNG chunk length or type").offset(offset));
            }
            if chunks == 1 && (kind != b"IHDR" || length != 13) {
                return Err(malformed("PNG must start with a 13-byte IHDR").offset(offset));
            }
            if chunks != 1 && kind == b"IHDR" {
                return Err(malformed("duplicate PNG IHDR").offset(offset));
            }
            if kind == b"PLTE"
                && (palette
                    || idat
                    || length == 0
                    || length > 768
                    || !length.is_multiple_of(3)
                    || matches!(color_model, 0 | 4)
                    || (indexed && length / 3 > (1u64 << sample_depth)))
            {
                return Err(malformed("invalid PNG palette or order").offset(offset));
            }
            if kind == b"IDAT" && (idat_ended || (indexed && !palette)) {
                return Err(malformed("invalid PNG IDAT order or missing palette").offset(offset));
            }
            if kind == b"IEND" && (length != 0 || !idat) {
                return Err(malformed("invalid PNG IEND or missing IDAT").offset(offset));
            }
            if kind == b"acTL"
                || (kind[0].is_ascii_uppercase()
                    && ![&b"IHDR"[..], b"PLTE", b"IDAT", b"IEND"].contains(&kind))
            {
                return Err(OperationError::new(
                    ErrorCode::UnsupportedFormat,
                    "unsupported PNG critical chunk or animation",
                )
                .offset(offset));
            }
            let mut payload = Vec::new();
            if payload_budget > 0 {
                exported = exported
                    .checked_add(length as usize)
                    .ok_or_else(|| limit("PNG payload export overflow"))?;
                if exported > payload_budget {
                    return Err(limit("PNG payload export exceeds requested total budget"));
                }
            }
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(kind);
            let mut remaining = length;
            let mut buffer = [0; 64 * 1024];
            while remaining > 0 {
                let count = remaining.min(buffer.len() as u64) as usize;
                read(reader, &mut buffer[..count])?;
                hasher.update(&buffer[..count]);
                if payload_budget > 0 {
                    payload.extend_from_slice(&buffer[..count]);
                }
                if kind == b"IHDR" {
                    let width = u32::from_be_bytes(buffer[..4].try_into().expect("IHDR width"));
                    let height = u32::from_be_bytes(buffer[4..8].try_into().expect("IHDR height"));
                    let depth = buffer[8];
                    let color = buffer[9];
                    color_model = color;
                    sample_depth = depth;
                    indexed = color == 3;
                    let valid_depth = match color {
                        0 => [1, 2, 4, 8, 16].contains(&depth),
                        2 | 4 | 6 => [8, 16].contains(&depth),
                        3 => [1, 2, 4, 8].contains(&depth),
                        _ => false,
                    };
                    if width == 0
                        || height == 0
                        || width > 0x7fffffff
                        || height > 0x7fffffff
                        || !valid_depth
                        || buffer[10] != 0
                        || buffer[11] != 0
                        || buffer[12] > 1
                    {
                        return Err(malformed("invalid PNG IHDR dimensions or color parameters")
                            .offset(offset));
                    }
                }
                remaining -= count as u64;
            }
            let mut crc = [0; 4];
            read(reader, &mut crc)?;
            let stored_crc = u32::from_be_bytes(crc);
            let calculated_crc = hasher.finalize();
            let valid = stored_crc == calculated_crc;
            if !valid && strict {
                return Err(malformed("PNG chunk CRC mismatch").offset(offset));
            }
            crc_valid &= valid;
            records.push(Chunk {
                kind: String::from_utf8(kind.to_vec()).expect("validated ASCII chunk type"),
                offset,
                length: length as u32,
                stored_crc,
                calculated_crc,
                valid,
                payload_hex: (payload_budget > 0).then(|| hex::encode(payload)),
            });
            if kind == b"PLTE" {
                palette = true;
            }
            if kind == b"IDAT" {
                idat = true;
            } else if idat {
                idat_ended = true;
            }
            offset = end;
            if kind == b"IEND" {
                return Ok(Report {
                    boundary: Boundary {
                        format: "png",
                        logical_end: end,
                        trailing_offset: end,
                        trailing_bytes: total - end,
                        validated_chunks: chunks,
                        evidence: if crc_valid {
                            "PNG framing, IHDR/order and all chunk CRCs validated; pixel stream not decoded"
                        } else {
                            "PNG framing inspected with CRC mismatches; not a validated boundary"
                        },
                    },
                    chunks: records,
                    crc_valid,
                });
            }
        }
    })();
    result.map_err(|e: OperationError| e.operation("image.png.inspect"))
}

/// Locate a validated IEND boundary using the same strict chunk inspector.
/// The existing boundary API and error operation ID remain compatible.
pub fn appended(reader: &mut (impl Read + Seek)) -> Result<Boundary, OperationError> {
    inspect(reader, true, 0)
        .map(|report| report.boundary)
        .map_err(|e| e.operation("file.appended.detect"))
}

/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "file.appended.detect",
        category: "file",
        description: "Find trailing bytes after validated PNG framing and CRCs; no pixel decoding.",
        input_type: "byte_reader",
        output_type: "file.png_boundary",
        command: "file appended",
        example: "cube file appended sample.png",
        parameters: &[],
        deterministic: true,
        reversible: None,
    },
    crate::operations::Descriptor {
        id: "image.png.inspect",
        category: "image",
        description: "Inspect PNG chunks, exact offsets and CRC evidence without pixel decoding.",
        input_type: "byte_reader",
        output_type: "image.png_report",
        command: "image chunks",
        example: "cube image chunks sample.png --json",
        deterministic: true,
        reversible: None,
        parameters: &[
            crate::operations::Parameter {
                name: "strict",
                description: "Reject CRC errors; false reports mismatches but still rejects unsafe framing.",
                required: false,
                default: Some(crate::operations::ParameterValue::Boolean(true)),
                constraints: crate::operations::ParameterType::Boolean,
            },
            crate::operations::Parameter {
                name: "payload_budget",
                description: "Total optional raw payload export budget; zero disables payload export.",
                required: false,
                default: Some(crate::operations::ParameterValue::Unsigned(0)),
                constraints: crate::operations::ParameterType::Unsigned {
                    min: 0,
                    max: 1048576,
                },
            },
        ],
    },
];
