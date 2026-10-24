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
#[derive(Debug, Serialize)]
pub struct Boundary {
    pub format: &'static str,
    pub logical_end: u64,
    pub trailing_offset: u64,
    pub trailing_bytes: u64,
    pub validated_chunks: usize,
    pub evidence: &'static str,
}

/// Locate IEND after bounded framing/CRC/order validation. Supports basic PNG
/// (IHDR, optional PLTE, consecutive IDAT, IEND and ancillary chunks). Unknown
/// critical chunks and APNG are unsupported. Pixel decompression is intentionally
/// excluded. At most 64 MiB before IEND and 10000 chunks are processed.
pub fn appended(reader: &mut (impl Read + Seek)) -> Result<Boundary, OperationError> {
    let result = (|| {
        let total = size(reader)?;
        reader.seek(SeekFrom::Start(0))?;
        let mut signature = [0; 8];
        read(reader, &mut signature)?;
        if &signature != b"\x89PNG\r\n\x1a\n" {
            return Err(OperationError::new(
                ErrorCode::UnsupportedFormat,
                "appended-data boundaries currently support PNG only",
            ));
        }
        let mut offset = 8u64;
        let mut chunks = 0;
        let mut palette = false;
        let mut idat = false;
        let mut idat_ended = false;
        let mut indexed = false;
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
                && (palette || idat || length == 0 || length > 768 || !length.is_multiple_of(3))
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
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(kind);
            let mut remaining = length;
            let mut buffer = [0; 64 * 1024];
            while remaining > 0 {
                let count = remaining.min(buffer.len() as u64) as usize;
                read(reader, &mut buffer[..count])?;
                hasher.update(&buffer[..count]);
                if kind == b"IHDR" {
                    let width = u32::from_be_bytes(buffer[..4].try_into().expect("IHDR width"));
                    let height = u32::from_be_bytes(buffer[4..8].try_into().expect("IHDR height"));
                    let depth = buffer[8];
                    let color = buffer[9];
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
            if u32::from_be_bytes(crc) != hasher.finalize() {
                return Err(malformed("PNG chunk CRC mismatch").offset(offset));
            }
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
                return Ok(Boundary {
                    format: "png",
                    logical_end: end,
                    trailing_offset: end,
                    trailing_bytes: total - end,
                    validated_chunks: chunks,
                    evidence: "PNG framing, IHDR/order and all chunk CRCs validated; pixel stream not decoded",
                });
            }
        }
    })();
    result.map_err(|e: OperationError| e.operation("file.appended.detect"))
}

/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
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
}];
