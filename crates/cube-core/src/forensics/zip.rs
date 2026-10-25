//! Bounded classic ZIP structure validation, not decompression or payload integrity.
use super::{ByteRange, invalid, limit, malformed, read_range, size};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

pub const MAX_ARCHIVE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_ENTRIES: usize = 1000;
const MAX_METADATA: usize = 1024 * 1024;
const MAX_DECLARED_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
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
#[derive(Clone, Debug, Serialize)]
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
fn part(bytes: &[u8], start: usize, length: usize) -> Result<&[u8], OperationError> {
    let end = start
        .checked_add(length)
        .ok_or_else(|| malformed("ZIP extent overflow"))?;
    bytes.get(start..end).ok_or_else(|| {
        malformed("ZIP structure extends beyond selected bounds").offset(start as u64)
    })
}
fn word(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes(b[p..p + 2].try_into().expect("checked fixed header"))
}
fn dword(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p + 4].try_into().expect("checked fixed header"))
}
#[derive(Default)]
struct Budget {
    records: usize,
    metadata: usize,
}
impl Budget {
    fn charge(&mut self, n: usize) -> Result<(), OperationError> {
        self.records += 1;
        self.metadata = self
            .metadata
            .checked_add(n)
            .ok_or_else(|| limit("ZIP metadata overflow"))?;
        if self.records > 10000 || self.metadata > MAX_METADATA {
            return Err(limit("ZIP validation exceeds record/1 MiB metadata budget"));
        }
        Ok(())
    }
}
fn extra(mut bytes: &[u8], mut offset: usize) -> Result<(), OperationError> {
    while !bytes.is_empty() {
        let header = part(bytes, 0, 4).map_err(|e| e.offset(offset as u64))?;
        if word(header, 0) == 1 {
            return Err(unsupported("ZIP64 extra fields are not supported").offset(offset as u64));
        }
        let n = word(header, 2) as usize;
        part(bytes, 4, n).map_err(|e| e.offset((offset + 4) as u64))?;
        bytes = &bytes[4 + n..];
        offset += 4 + n;
    }
    Ok(())
}

/// Validate one explicitly selected archive origin. Offsets in ZIP records must
/// be relative to that origin. At most a 16 MiB prefix, 64 EOCD candidates, 1000
/// entries, 10000 local/central header visits and 1 MiB cumulative metadata are admitted;
/// declared uncompressed totals must not exceed 64 MiB. No payload is expanded.
/// Classic single-disk stored/deflate entries and signed 32-bit data descriptors
/// are supported. ZIP64, encryption, alternate offset conventions, record gaps,
/// digital signatures and unsigned descriptors are outside this subset.
pub fn validate(reader: &mut (impl Read + Seek), offset: u64) -> Result<Archive, OperationError> {
    let result = (|| {
        let total = size(reader)?;
        ByteRange {
            offset,
            length: Some(0),
        }
        .resolve(total, u64::MAX)?;
        reader.seek(SeekFrom::Start(offset))?;
        let mut header = [0; 4];
        reader
            .read_exact(&mut header)
            .map_err(|e| malformed("truncated ZIP candidate header").caused_by(e))?;
        if &header != b"PK\x03\x04" && &header != b"PK\x05\x06" {
            return Err(unsupported(
                "candidate offset does not begin a supported classic ZIP archive",
            ));
        }
        let selected = (total - offset).min(MAX_ARCHIVE_BYTES);
        let bytes = read_range(
            reader,
            ByteRange {
                offset,
                length: Some(selected),
            },
        )?;
        let mut candidates = 0;
        let mut budget = Budget::default();
        let mut valid = None;
        let mut failure =
            malformed("no structurally valid ZIP end record and directory relationships");
        for (pos, _) in bytes
            .windows(4)
            .enumerate()
            .filter(|(_, b)| *b == b"PK\x05\x06")
        {
            candidates += 1;
            if candidates > 64 {
                return Err(limit("ZIP validation exceeds 64 end-record candidates"));
            }
            match directory(&bytes, pos, offset, total, &mut budget) {
                Ok(archive) => {
                    if valid.is_some() {
                        return Err(malformed("ambiguous ZIP archive extents"));
                    }
                    valid = Some(archive);
                }
                Err(e) if e.code == ErrorCode::ResourceLimit => return Err(e),
                Err(e) => {
                    if failure.code != ErrorCode::UnsupportedFormat {
                        failure = e;
                    }
                }
            }
        }
        if let Some(archive) = valid {
            Ok(archive)
        } else if selected < total - offset {
            Err(limit(
                "no validated ZIP boundary within the 16 MiB candidate budget",
            ))
        } else {
            Err(failure)
        }
    })();
    result.map_err(|mut e: OperationError| {
        e.offset = e.offset.and_then(|relative| offset.checked_add(relative));
        e.operation("file.zip.validate")
    })
}

fn directory(
    bytes: &[u8],
    pos: usize,
    base: u64,
    total: u64,
    budget: &mut Budget,
) -> Result<Archive, OperationError> {
    let end = part(bytes, pos, 22)?;
    let comment = word(end, 20) as usize;
    part(bytes, pos + 22, comment)?;
    if pos >= 20 && &bytes[pos - 20..pos - 16] == b"PK\x06\x07" {
        return Err(unsupported("ZIP64 end records are not supported"));
    }
    if word(end, 4) != 0 || word(end, 6) != 0 || word(end, 8) != word(end, 10) {
        return Err(unsupported("multi-disk ZIP archives are not supported"));
    }
    let count = word(end, 10) as usize;
    let (directory_size, directory_offset) = (dword(end, 12), dword(end, 16));
    if count == 65535 || directory_size == u32::MAX || directory_offset == u32::MAX {
        return Err(unsupported("ZIP64 end-record fields are not supported"));
    }
    if count > MAX_ENTRIES {
        return Err(limit("ZIP archive exceeds 1000 entries"));
    }
    let (cd_size, cd_start) = (directory_size as usize, directory_offset as usize);
    if cd_start.checked_add(cd_size) != Some(pos) {
        return Err(malformed("ZIP central directory does not end at EOCD"));
    }
    part(bytes, cd_start, cd_size)?;
    let mut p = cd_start;
    let mut entries = Vec::new();
    let mut extents = Vec::new();
    let mut declared = 0u64;
    for _ in 0..count {
        let header = part(bytes, p, 46)?;
        if &header[..4] != b"PK\x01\x02" {
            return Err(malformed("invalid ZIP central-directory record"));
        }
        let (needed, flags, method) = (word(header, 6), word(header, 8), word(header, 10));
        let (crc, compressed, uncompressed) =
            (dword(header, 16), dword(header, 20), dword(header, 24));
        let (name_len, extra_len, comment_len) = (
            word(header, 28) as usize,
            word(header, 30) as usize,
            word(header, 32) as usize,
        );
        let local_offset = dword(header, 42);
        if compressed == u32::MAX
            || uncompressed == u32::MAX
            || local_offset == u32::MAX
            || needed == 45
        {
            return Err(unsupported("ZIP64 entries are not supported"));
        }
        if flags & 0x2041 != 0 {
            return Err(unsupported(
                "encrypted ZIP entries are not supported; payloads are not verified",
            ));
        }
        if flags & !0x080e != 0 || !matches!(method, 0 | 8) || needed > 20 || word(header, 34) != 0
        {
            return Err(unsupported(
                "unsupported ZIP flags, compression, version or disk",
            ));
        }
        if needed < 10 || (method == 8 && needed < 20) || (method == 0 && flags & 6 != 0) {
            return Err(malformed("inconsistent ZIP version/method flags"));
        }
        if method == 0 && compressed != uncompressed {
            return Err(malformed("stored ZIP entry sizes differ"));
        }
        declared = declared
            .checked_add(uncompressed as u64)
            .ok_or_else(|| limit("ZIP declared-size overflow"))?;
        if declared > MAX_DECLARED_BYTES {
            return Err(limit("ZIP declared output exceeds 64 MiB"));
        }
        budget.charge(46 + name_len + extra_len + comment_len)?;
        let name = part(bytes, p + 46, name_len)?;
        extra(
            part(bytes, p + 46 + name_len, extra_len)?,
            p + 46 + name_len,
        )?;
        let entry_comment = part(bytes, p + 46 + name_len + extra_len, comment_len)?;
        if flags & 0x800 != 0
            && (std::str::from_utf8(name).is_err() || std::str::from_utf8(entry_comment).is_err())
        {
            return Err(malformed(
                "ZIP UTF-8 flag contradicts filename/comment bytes",
            ));
        }
        p = p
            .checked_add(46 + name_len + extra_len + comment_len)
            .ok_or_else(|| malformed("ZIP directory extent overflow"))?;
        if p > pos {
            return Err(malformed("ZIP entry extends beyond central directory"));
        }
        let local = local_offset as usize;
        let h = part(bytes, local, 30)?;
        if &h[..4] != b"PK\x03\x04"
            || word(h, 4) != needed
            || word(h, 6) != flags
            || word(h, 8) != method
            || h[10..14] != header[12..16]
        {
            return Err(malformed("ZIP local header contradicts central directory"));
        }
        let (local_name, local_extra) = (word(h, 26) as usize, word(h, 28) as usize);
        budget.charge(30 + local_name + local_extra)?;
        let name_start = local
            .checked_add(30)
            .ok_or_else(|| malformed("ZIP local offset overflow"))?;
        if part(bytes, name_start, local_name)? != name {
            return Err(malformed(
                "ZIP local filename differs from central directory",
            ));
        }
        extra(
            part(bytes, name_start + local_name, local_extra)?,
            name_start + local_name,
        )?;
        let data = name_start
            .checked_add(local_name + local_extra)
            .ok_or_else(|| malformed("ZIP data offset overflow"))?;
        let data_end = data
            .checked_add(compressed as usize)
            .ok_or_else(|| malformed("ZIP data extent overflow"))?;
        part(bytes, data, compressed as usize)?;
        let local_fields = [dword(h, 14), dword(h, 18), dword(h, 22)];
        let expected = [crc, compressed, uncompressed];
        let local_end = if flags & 8 != 0 {
            if local_fields
                .iter()
                .zip(expected)
                .any(|(&a, b)| a != 0 && a != b)
            {
                return Err(malformed(
                    "ZIP local descriptor-mode sizes/CRC contradict directory",
                ));
            }
            let descriptor = part(bytes, data_end, 16)?;
            if &descriptor[..4] != b"PK\x07\x08" {
                return Err(unsupported(
                    "only signed 32-bit ZIP data descriptors are supported",
                ));
            }
            if [
                dword(descriptor, 4),
                dword(descriptor, 8),
                dword(descriptor, 12),
            ] != expected
            {
                return Err(malformed("ZIP data descriptor contradicts directory"));
            }
            data_end
                .checked_add(16)
                .ok_or_else(|| malformed("ZIP descriptor extent overflow"))?
        } else {
            if local_fields != expected {
                return Err(malformed("ZIP local sizes/CRC contradict directory"));
            }
            data_end
        };
        if local_end > cd_start {
            return Err(malformed("ZIP entry overlaps central directory"));
        }
        extents.push((local, local_end));
        entries.push(Entry {
            local_offset: base
                .checked_add(local as u64)
                .ok_or_else(|| malformed("ZIP absolute local offset overflow"))?,
            data_offset: base
                .checked_add(data as u64)
                .ok_or_else(|| malformed("ZIP absolute data offset overflow"))?,
            compressed_size: compressed as u64,
            uncompressed_size: uncompressed as u64,
            crc32: crc,
            method,
            flags,
            name_hex: crate::encoding::hex_encode(name),
        });
    }
    if p != pos {
        return Err(malformed("ZIP directory size/count mismatch"));
    }
    extents.sort_unstable();
    let mut previous = 0;
    for (start, end) in extents {
        if start != previous {
            return Err(unsupported(
                "ZIP local records overlap, have gaps or use another offset convention",
            ));
        }
        previous = end;
    }
    if previous != cd_start {
        return Err(unsupported(
            "ZIP local records do not reach central directory contiguously",
        ));
    }
    let length = (pos + 22 + comment) as u64;
    let logical_end = base
        .checked_add(length)
        .ok_or_else(|| invalid("ZIP absolute extent overflow"))?;
    if logical_end > total {
        return Err(malformed("ZIP archive extends beyond source"));
    }
    Ok(Archive {
        format: "zip",
        offset: base,
        length,
        logical_end,
        trailing_bytes: total - logical_end,
        entries,
        payload_verified: false,
        scanned_bytes: bytes.len() as u64,
        candidate_scan_complete: bytes.len() as u64 == total - base,
        evidence: "Classic single-disk ZIP central/local/end relationships and bounded extents; compressed payloads, CRC integrity and authenticity are not verified",
    })
}

pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.zip.validate",
    category: "file",
    description: "Validate bounded classic ZIP structure at an explicit candidate origin; no decompression or payload integrity claim.",
    input_type: "byte_reader",
    output_type: "file.zip_structure",
    parameters: &[super::OFFSET_PARAMETER],
    deterministic: true,
    reversible: None,
    command: "file zip",
    example: "cube file zip sample.bin --offset 128 --json",
}];
