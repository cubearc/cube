//! Bounded ZIP and ZIP64 framing and metadata validation.
use super::{ByteRange, limit, malformed, read_range, size};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

pub const MAX_ARCHIVE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 1000;
const MAX_METADATA: usize = 1024 * 1024;
const MAX_DECLARED_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct Entry {
    /// Absolute source offsets. File names remain inert exact hexadecimal data.
    pub local_offset: u64,
    pub data_offset: u64,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub crc32: u32,
    pub method: u16,
    pub flags: u16,
    pub name_hex: String,
}
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct Archive {
    pub format: &'static str,
    pub offset: u64,
    pub length: u64,
    pub logical_end: u64,
    pub trailing_bytes: u64,
    pub entries: Vec<Entry>,
    pub payload_verified: bool,
    pub scanned_bytes: u64,
    pub candidate_scan_complete: bool,
    pub evidence: &'static str,
}
fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message)
}
mod records;

fn word(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes(b[p..p + 2].try_into().expect("checked fixed header"))
}
fn dword(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p + 4].try_into().expect("checked fixed header"))
}
fn qword(b: &[u8], p: usize) -> u64 {
    u64::from_le_bytes(b[p..p + 8].try_into().expect("checked fixed header"))
}

#[derive(Default)]
struct Budget {
    records: usize,
    metadata: usize,
}
impl Budget {
    fn charge(&mut self, n: usize) -> Result<(), OperationError> {
        self.records += 1;
        self.metadata += n;
        if self.records > 10_000 || self.metadata > MAX_METADATA {
            return Err(limit("ZIP validation exceeds record/1 MiB metadata budget"));
        }
        Ok(())
    }
}

/// Validate one selected classic ZIP or ZIP64 archive. Offsets in the ZIP are
/// relative to the supplied origin. ZIP64 records outside the inspected prefix
/// are read with bounded seeking. No payload is extracted or verified.
pub fn validate(reader: &mut (impl Read + Seek), offset: u64) -> Result<Archive, OperationError> {
    validate_at(reader, offset).map_err(|mut error| {
        error.offset = error
            .offset
            .and_then(|relative| offset.checked_add(relative));
        error.operation("file.zip.validate")
    })
}

fn validate_at(reader: &mut (impl Read + Seek), offset: u64) -> Result<Archive, OperationError> {
    let total = size(reader)?;
    ByteRange {
        offset,
        length: Some(0),
    }
    .resolve(total, u64::MAX)?;
    let region = total - offset;
    reader.seek(SeekFrom::Start(offset))?;
    let mut signature = [0; 4];
    reader
        .read_exact(&mut signature)
        .map_err(|e| malformed("truncated ZIP candidate header").caused_by(e))?;
    if !matches!(&signature, b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x06\x06") {
        return Err(unsupported("candidate offset does not begin a ZIP archive"));
    }
    let scanned = region.min(MAX_ARCHIVE_BYTES);
    let prefix = read_range(
        reader,
        ByteRange {
            offset,
            length: Some(scanned),
        },
    )?;
    let tail_start = region.saturating_sub(65_557 + 20);
    let tail = if scanned < region {
        read_range(
            reader,
            ByteRange {
                offset: offset + tail_start,
                length: Some(region - tail_start),
            },
        )?
    } else {
        Vec::new()
    };
    let mut source = records::Source {
        reader,
        base: offset,
        length: region,
        budget: Budget::default(),
    };
    let mut valid = None;
    let mut failure = malformed("no structurally valid ZIP end record and directory relationships");
    let mut candidates = 0;
    for (start, bytes, is_prefix) in [
        (0, prefix.as_slice(), true),
        (tail_start, tail.as_slice(), false),
    ] {
        for (index, _) in bytes
            .windows(4)
            .enumerate()
            .filter(|(_, b)| *b == b"PK\x05\x06")
        {
            let position = start + index as u64;
            if !is_prefix && position < scanned {
                continue;
            }
            candidates += 1;
            if candidates > 64 {
                return Err(limit("ZIP validation exceeds 64 end-record candidates"));
            }
            match source.directory(position, scanned, !is_prefix) {
                Ok(archive) => {
                    if valid.is_some() {
                        return Err(malformed("ambiguous ZIP archive extents"));
                    }
                    valid = Some(archive);
                }
                Err(error) if error.code == ErrorCode::ResourceLimit => return Err(error),
                Err(error) if failure.code != ErrorCode::UnsupportedFormat => failure = error,
                Err(_) => {}
            }
        }
    }
    if let Some(archive) = valid {
        Ok(archive)
    } else if scanned < region && failure.code != ErrorCode::UnsupportedFormat {
        Err(limit(
            "no validated ZIP boundary within the candidate search budget",
        ))
    } else {
        Err(failure)
    }
}

pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.zip.validate",
    category: "file",
    description: "Validate bounded classic ZIP or ZIP64 structure at an explicit candidate origin; no decompression or payload integrity claim.",
    input_type: "byte_reader",
    output_type: "file.zip_structure",
    parameters: &[super::OFFSET_PARAMETER],
    deterministic: true,
    reversible: None,
    command: "file zip",
    example: "cube file zip sample.bin --offset 128 --json",
}];
