//! Local, read-only file analysis with explicit byte ranges and resource budgets.
//!
//! Paths must name regular files; symbolic links are rejected. Byte offsets are
//! absolute and zero-based. No operation executes extracted content or uses the
//! network. Reader APIs let callers control transport without a CLI subprocess.

pub mod boundaries;
pub mod carving;
pub mod exif;
pub mod inspect;
pub mod png;
pub mod ranges;
pub mod signatures;
pub mod zip;

use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

/// Hard scan budget (64 MiB); callers may choose a smaller limit.
pub const MAX_SCAN_BYTES: u64 = 64 * 1024 * 1024;
/// Hard allocation/output budget for byte-returning operations (16 MiB).
pub const MAX_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;
/// Hard processing/output limit for streaming range copies (1 GiB).
pub const MAX_COPY_BYTES: u64 = 1024 * 1024 * 1024;
/// Maximum number of result records per request.
pub const MAX_RESULTS: usize = 10_000;

/// Half-open byte selection. `None` length selects through EOF.
#[derive(Clone, Copy, Debug, Default, Serialize, schemars::JsonSchema)]
pub struct ByteRange {
    pub offset: u64,
    pub length: Option<u64>,
}

impl ByteRange {
    /// Validate a selection against a known size and a caller's processing budget.
    pub fn resolve(self, size: u64, budget: u64) -> Result<(u64, u64), OperationError> {
        let length = self
            .length
            .unwrap_or_else(|| size.saturating_sub(self.offset));
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid("offset + length overflows").parameter("length"))?;
        if self.offset > size || end > size {
            return Err(invalid("range extends beyond EOF").parameter("length"));
        }
        if length > budget {
            return Err(limit(
                "selected range exceeds byte budget; choose a smaller range",
            ));
        }
        Ok((self.offset, length))
    }
}

pub(super) fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
pub(super) fn malformed(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
pub(super) fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}
pub(super) fn check_results(count: usize) -> Result<(), OperationError> {
    if count == 0 || count > MAX_RESULTS {
        return Err(invalid("max_results must be between 1 and 10000").parameter("max_results"));
    }
    Ok(())
}

/// Open a regular file for read-only inspection. Symlinks and special files are rejected.
pub fn open_regular(path: &Path) -> Result<File, OperationError> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(invalid(
            "input must be a regular file; symbolic links and devices are not supported",
        ));
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("input is no longer a regular file"));
    }
    Ok(file)
}

pub(super) fn size(reader: &mut (impl Seek + ?Sized)) -> Result<u64, OperationError> {
    Ok(reader.seek(SeekFrom::End(0))?)
}

/// Copy an explicit range to a writer using at most 64 KiB of working memory.
/// The default hard processing limit is 1 GiB. Short reads and writer errors fail.
pub fn copy_range(
    reader: &mut (impl Read + Seek),
    writer: &mut impl Write,
    offset: u64,
    length: u64,
) -> Result<u64, OperationError> {
    let result = (|| {
        ByteRange {
            offset,
            length: Some(length),
        }
        .resolve(size(reader)?, MAX_COPY_BYTES)?;
        reader.seek(SeekFrom::Start(offset))?;
        let mut remaining = length;
        let mut buffer = [0; 64 * 1024];
        while remaining > 0 {
            let count = remaining.min(buffer.len() as u64) as usize;
            reader.read_exact(&mut buffer[..count])?;
            writer.write_all(&buffer[..count])?;
            remaining -= count as u64;
        }
        writer.flush()?;
        Ok(length)
    })();
    result.map_err(|e: OperationError| e.operation("file.range.extract"))
}

/// Read a selected range into exact bytes under the 16 MiB allocation cap.
pub fn read_range(
    reader: &mut (impl Read + Seek),
    range: ByteRange,
) -> Result<Vec<u8>, OperationError> {
    let (offset, length) = range.resolve(size(reader)?, MAX_OUTPUT_BYTES)?;
    reader.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Extract to a new destination. Exclusive creation rejects existing files and all
/// source aliases (same path, hard links and symlinks). Only a newly created partial
/// destination is removed on failure; the source and existing files are untouched.
pub fn extract_range(
    source: &Path,
    destination: &Path,
    offset: u64,
    length: u64,
) -> Result<u64, OperationError> {
    let result = (|| {
        let mut input = open_regular(source)?;
        ByteRange {
            offset,
            length: Some(length),
        }
        .resolve(input.metadata()?.len(), MAX_COPY_BYTES)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        let result = copy_range(&mut input, &mut output, offset, length);
        drop(output);
        if result.is_err() {
            let _cleanup = fs::remove_file(destination);
        }
        result
    })();
    result.map_err(|e: OperationError| e.operation("file.range.extract"))
}

pub(super) const OFFSET_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "offset",
    description: "Absolute zero-based byte offset.",
    required: false,
    default: Some(crate::operations::ParameterValue::Unsigned(0)),
    constraints: crate::operations::ParameterType::Unsigned {
        min: 0,
        max: u64::MAX,
    },
};
pub(super) const LENGTH_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "length",
    description: "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
    required: false,
    default: None,
    constraints: crate::operations::ParameterType::Unsigned {
        min: 0,
        max: u64::MAX,
    },
};
pub(super) const RESULT_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "max_results",
    description: "Maximum returned records; incomplete listings are explicit.",
    required: false,
    default: Some(crate::operations::ParameterValue::Unsigned(1000)),
    constraints: crate::operations::ParameterType::Unsigned {
        min: 1,
        max: MAX_RESULTS as u64,
    },
};
const EXTRACTION_PARAMETERS: &[crate::operations::Parameter] = &[
    crate::operations::Parameter {
        required: true,
        default: None,
        ..OFFSET_PARAMETER
    },
    crate::operations::Parameter {
        required: true,
        description: "Exact byte count; streaming supports at most 1 GiB and byte-returning APIs 16 MiB.",
        constraints: crate::operations::ParameterType::Unsigned {
            min: 0,
            max: 1073741824,
        },
        ..LENGTH_PARAMETER
    },
];
/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.range.extract",
    category: "file",
    description: "Copy an explicit byte range to a new destination; never overwrite.",
    input_type: "byte_reader",
    output_type: "bytes",
    command: "file extract",
    example: "cube file extract sample.bin --offset 4 --length 8 --output bytes.bin",
    parameters: EXTRACTION_PARAMETERS,
    deterministic: true,
    reversible: None,
}];
