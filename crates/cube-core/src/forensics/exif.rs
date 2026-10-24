//! Explicit local JPEG/TIFF EXIF inspection through kamadak-exif (BSD-2-Clause).
//! Metadata stays typed; rational pairs and original ASCII bytes are preserved.

use super::{limit, malformed};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::collections::BTreeSet;

/// Maximum raw TIFF metadata/prefix length (1 MiB).
pub const MAX_METADATA_BYTES: usize = 1024 * 1024;

/// Read at most a 1 MiB prefix plus one sentinel byte, then inspect JPEG/TIFF.
/// TIFF beyond the cap is rejected; JPEG data beyond this prefix is not read.
pub fn read(reader: &mut impl std::io::Read) -> Result<Metadata, OperationError> {
    use std::io::Read;
    let mut input = Vec::new();
    reader
        .take((MAX_METADATA_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|e| OperationError::io(e).operation("file.exif.inspect"))?;
    inspect(&input)
}

/// Original EXIF values, without flattening typed values to display text.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "values", rename_all = "snake_case")]
pub enum TagValue {
    Byte(Vec<u8>),
    Ascii(Vec<Vec<u8>>),
    Short(Vec<u16>),
    Long(Vec<u32>),
    Rational(Vec<(u32, u32)>),
    SByte(Vec<i8>),
    Undefined(Vec<u8>),
    SShort(Vec<i16>),
    SLong(Vec<i32>),
    SRational(Vec<(i32, i32)>),
    /// Exact IEEE bit patterns preserve NaN/Infinity and signed zero in JSON.
    FloatBits(Vec<u32>),
    DoubleBits(Vec<u64>),
    Unknown {
        field_type: u16,
        count: u32,
        offset: u32,
    },
}

/// IFD/tag identity, typed original values, and an additional human display with units.
#[derive(Debug, Serialize)]
pub struct Tag {
    pub ifd: u16,
    pub context: String,
    pub tag: u16,
    pub name: String,
    pub value: TagValue,
    pub display: String,
}
/// Absence is a successful result, distinct from malformed or unsupported input.
#[derive(Debug, Serialize)]
pub struct Metadata {
    pub present: bool,
    pub little_endian: Option<bool>,
    pub tags: Vec<Tag>,
}

fn preflight(data: &[u8]) -> Result<(), OperationError> {
    if data.len() > MAX_METADATA_BYTES {
        return Err(limit("EXIF metadata exceeds 1 MiB"));
    }
    if data.len() < 8 {
        return Err(malformed("truncated TIFF header"));
    }
    let little = data.starts_with(b"II\x2a\0");
    if !little && !data.starts_with(b"MM\0\x2a") {
        return Err(malformed("invalid TIFF byte order or version"));
    }
    let u16_at = |offset: usize| -> Result<u16, OperationError> {
        let bytes: [u8; 2] = data
            .get(offset..offset + 2)
            .ok_or_else(|| malformed("truncated TIFF entry"))?
            .try_into()
            .expect("checked length");
        Ok(if little {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    };
    let u32_at = |offset: usize| -> Result<u32, OperationError> {
        let bytes: [u8; 4] = data
            .get(offset..offset + 4)
            .ok_or_else(|| malformed("truncated TIFF offset"))?
            .try_into()
            .expect("checked length");
        Ok(if little {
            u32::from_le_bytes(bytes)
        } else {
            u32::from_be_bytes(bytes)
        })
    };
    let mut pending = vec![u32_at(4)? as usize];
    let mut visited = BTreeSet::new();
    let mut entries = 0;
    let mut allocated = 0u64;
    while let Some(offset) = pending.pop() {
        if offset == 0 {
            continue;
        }
        if !visited.insert(offset) {
            return Err(malformed("cyclic or shared TIFF directory offset"));
        }
        if visited.len() > 32 {
            return Err(limit("EXIF directory count exceeds 32"));
        }
        let count = u16_at(offset)? as usize;
        entries += count;
        if entries > 4096 {
            return Err(limit("EXIF entry count exceeds 4096"));
        }
        let end = offset + 2 + count * 12;
        if end + 4 > data.len() {
            return Err(malformed("truncated TIFF directory"));
        }
        for index in 0..count {
            let entry = offset + 2 + index * 12;
            let tag = u16_at(entry)?;
            let kind = u16_at(entry + 2)?;
            let count = u32_at(entry + 4)? as u64;
            let width = match kind {
                3 | 8 => 2,
                4 | 9 | 11 => 4,
                5 | 10 | 12 => 8,
                _ => 1,
            };
            allocated += count * width;
            if allocated > 2 * MAX_METADATA_BYTES as u64 {
                return Err(limit("EXIF decoded value budget exceeds 2 MiB"));
            }
            if matches!(tag, 0x8769 | 0x8825 | 0xa005) {
                if kind != 4 || count != 1 {
                    return Err(malformed("invalid EXIF directory pointer"));
                }
                pending.push(u32_at(entry + 8)? as usize);
            }
        }
        pending.push(u32_at(end)? as usize);
    }
    Ok(())
}

fn tiff_bytes(input: &[u8]) -> Result<Option<&[u8]>, OperationError> {
    if input.starts_with(b"II") || input.starts_with(b"MM") {
        return Ok(Some(input));
    }
    if !input.starts_with(b"\xff\xd8") {
        return Err(OperationError::new(
            ErrorCode::UnsupportedFormat,
            "EXIF supports JPEG and TIFF only",
        ));
    }
    let mut offset = 2;
    loop {
        if offset >= MAX_METADATA_BYTES {
            return Err(limit("JPEG metadata prefix exceeds 1 MiB"));
        }
        if input.get(offset) != Some(&255) {
            return Err(malformed("invalid JPEG metadata marker").offset(offset as u64));
        }
        while input.get(offset) == Some(&255) {
            offset += 1;
            if offset > MAX_METADATA_BYTES {
                return Err(limit("JPEG marker padding exceeds metadata budget"));
            }
        }
        let marker = *input
            .get(offset)
            .ok_or_else(|| malformed("truncated JPEG marker"))?;
        offset += 1;
        if marker == 0xda || marker == 0xd9 {
            return Ok(None);
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let header = input
            .get(offset..offset + 2)
            .ok_or_else(|| malformed("truncated JPEG segment length"))?;
        let length = u16::from_be_bytes([header[0], header[1]]) as usize;
        if length < 2 {
            return Err(malformed("invalid JPEG segment length"));
        }
        let end = offset + length;
        if end > MAX_METADATA_BYTES {
            return Err(limit("JPEG metadata prefix exceeds 1 MiB"));
        }
        let payload = input
            .get(offset + 2..end)
            .ok_or_else(|| malformed("truncated JPEG segment"))?;
        if marker == 0xe1 && payload.starts_with(b"Exif\0\0") {
            return Ok(Some(&payload[6..]));
        }
        offset = end;
    }
}

/// Inspect JPEG APP1 EXIF or classic TIFF, with no pixel decoding. The safety
/// preflight limits metadata to 1 MiB, 32 unique directories, 4096 entries and
/// 2 MiB of decoded values. The dependency additionally bounds primary IFDs to 8
/// and restricts child contexts. Unknown/repeated tags remain separate records.
pub fn inspect(input: &[u8]) -> Result<Metadata, OperationError> {
    let result = (|| {
        let Some(tiff) = tiff_bytes(input)? else {
            return Ok(Metadata {
                present: false,
                little_endian: None,
                tags: vec![],
            });
        };
        preflight(tiff)?;
        let metadata = exif::Reader::new()
            .read_raw(tiff.to_vec())
            .map_err(|e| malformed("malformed EXIF directory or value").caused_by(e))?;
        let mut tags = Vec::new();
        for field in metadata.fields() {
            use exif::Value as V;
            let value = match &field.value {
                V::Byte(v) => TagValue::Byte(v.clone()),
                V::Ascii(v) => TagValue::Ascii(v.clone()),
                V::Short(v) => TagValue::Short(v.clone()),
                V::Long(v) => TagValue::Long(v.clone()),
                V::Rational(v) => TagValue::Rational(v.iter().map(|v| (v.num, v.denom)).collect()),
                V::SByte(v) => TagValue::SByte(v.clone()),
                V::Undefined(v, _) => TagValue::Undefined(v.clone()),
                V::SShort(v) => TagValue::SShort(v.clone()),
                V::SLong(v) => TagValue::SLong(v.clone()),
                V::SRational(v) => {
                    TagValue::SRational(v.iter().map(|v| (v.num, v.denom)).collect())
                }
                V::Float(v) => TagValue::FloatBits(v.iter().map(|v| v.to_bits()).collect()),
                V::Double(v) => TagValue::DoubleBits(v.iter().map(|v| v.to_bits()).collect()),
                V::Unknown(kind, count, offset) => TagValue::Unknown {
                    field_type: *kind,
                    count: *count,
                    offset: *offset,
                },
            };
            tags.push(Tag {
                ifd: field.ifd_num.0,
                context: format!("{:?}", field.tag.0),
                tag: field.tag.1,
                name: field.tag.to_string(),
                value,
                display: field.display_value().with_unit(&metadata).to_string(),
            });
        }
        Ok(Metadata {
            present: !tags.is_empty(),
            little_endian: Some(metadata.little_endian()),
            tags,
        })
    })();
    result.map_err(|e: OperationError| e.operation("file.exif.inspect"))
}

/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.exif.inspect",
    category: "file",
    description: "Inspect typed JPEG/TIFF EXIF metadata under explicit resource budgets.",
    input_type: "bytes",
    output_type: "file.exif",
    command: "file exif",
    example: "cube file exif sample.jpg",
    parameters: &[],
    deterministic: true,
    reversible: None,
}];
