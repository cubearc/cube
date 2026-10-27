//! Bounded hexadecimal, ASCII-string and exact binary-difference inspection.

use super::{ByteRange, MAX_OUTPUT_BYTES, MAX_SCAN_BYTES, check_results, invalid, limit, size};
use crate::errors::OperationError;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

/// Hex row with original bytes and a printable ASCII preview. JSON uses hex.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct HexRow {
    pub offset: u64,
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub hex: String,
    pub ascii: String,
}

/// Read selected rows under a 16 MiB output allocation budget. Row width is 1–256;
/// previews replace bytes outside 0x20–0x7e with '.', never Unicode replacement.
pub fn hex_rows(
    reader: &mut (impl Read + Seek),
    range: ByteRange,
    width: usize,
) -> Result<Vec<HexRow>, OperationError> {
    let result = (|| {
        if !(1..=256).contains(&width) {
            return Err(invalid("width must be between 1 and 256").parameter("width"));
        }
        // Hex, ASCII and retained bytes require at least four bytes per input byte;
        // row allocation overhead is conservatively budgeted for narrow widths.
        let budget = MAX_OUTPUT_BYTES / (4 + 128 / width as u64);
        let (offset, length) = range.resolve(size(reader)?, budget)?;
        reader.seek(SeekFrom::Start(offset))?;
        let mut rows = Vec::new();
        let mut consumed = 0;
        while consumed < length {
            let count = (length - consumed).min(width as u64) as usize;
            let mut bytes = vec![0; count];
            reader.read_exact(&mut bytes)?;
            let ascii = bytes
                .iter()
                .map(|byte| {
                    if (0x20..=0x7e).contains(byte) {
                        *byte as char
                    } else {
                        '.'
                    }
                })
                .collect();
            rows.push(HexRow {
                offset: offset + consumed,
                hex: hex::encode(&bytes),
                ascii,
                bytes,
            });
            consumed += count as u64;
        }
        Ok(rows)
    })();
    result.map_err(|e: OperationError| e.operation("file.hex.inspect"))
}

/// One complete ASCII printable run (spaces included).
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StringRun {
    pub offset: u64,
    pub length: u64,
    pub text: String,
}
/// String results and whether the entire requested range was scanned.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Strings {
    pub strings: Vec<StringRun>,
    pub scanned_bytes: u64,
    pub complete: bool,
}

/// Extract runs of bytes 0x20–0x7e, joining across read buffers and including EOF.
/// Run allocations are limited to 1 MiB and accumulated text to 16 MiB. Scanning
/// stops explicitly at the 64 MiB or result cap; no run is silently clipped.
pub fn strings(
    reader: &mut (impl Read + Seek),
    range: ByteRange,
    min_length: usize,
    max_results: usize,
) -> Result<Strings, OperationError> {
    let result = (|| {
        check_results(max_results)?;
        if min_length == 0 || min_length > 1024 * 1024 {
            return Err(invalid("min_length must be between 1 and 1048576").parameter("min_length"));
        }
        let (offset, length) = range.resolve(size(reader)?, u64::MAX)?;
        let selected = length.min(MAX_SCAN_BYTES);
        reader.seek(SeekFrom::Start(offset))?;
        let mut output = Vec::new();
        let mut run = String::new();
        let mut start = offset;
        let mut consumed = 0;
        let mut output_size = 0;
        let mut buffer = [0; 64 * 1024];
        while consumed < selected {
            let count = (selected - consumed).min(buffer.len() as u64) as usize;
            reader.read_exact(&mut buffer[..count])?;
            for byte in &buffer[..count] {
                if (0x20..=0x7e).contains(byte) {
                    if run.is_empty() {
                        start = offset + consumed;
                    }
                    if run.len() == 1024 * 1024 {
                        return Err(limit("printable run exceeds 1 MiB; select a smaller range")
                            .offset(start));
                    }
                    run.push(*byte as char);
                } else if !run.is_empty() {
                    if run.len() >= min_length {
                        if output.len() == max_results
                            || output_size + run.len() as u64 > MAX_OUTPUT_BYTES
                        {
                            return Ok(Strings {
                                strings: output,
                                scanned_bytes: consumed,
                                complete: false,
                            });
                        }
                        output_size += run.len() as u64;
                        output.push(StringRun {
                            offset: start,
                            length: run.len() as u64,
                            text: std::mem::take(&mut run),
                        });
                    }
                    run.clear();
                }
                consumed += 1;
            }
        }
        let mut complete = selected == length;
        // Do not publish an incomplete run at a scan-budget boundary as complete.
        if complete && run.len() >= min_length {
            if output.len() == max_results || output_size + run.len() as u64 > MAX_OUTPUT_BYTES {
                complete = false;
            } else {
                output.push(StringRun {
                    offset: start,
                    length: run.len() as u64,
                    text: run,
                });
            }
        }
        Ok(Strings {
            strings: output,
            scanned_bytes: consumed,
            complete,
        })
    })();
    result.map_err(|e: OperationError| e.operation("file.strings.extract"))
}

/// One differing offset. A missing side at EOF is None, never a fabricated zero.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Difference {
    pub offset: u64,
    pub left: Option<u8>,
    pub right: Option<u8>,
}
/// Exact equality/count plus a bounded ordered difference listing.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Comparison {
    pub equal: bool,
    pub left_size: u64,
    pub right_size: u64,
    pub differing_bytes: u64,
    pub differences: Vec<Difference>,
    pub complete: bool,
}

/// Compare corresponding offsets in an explicit range of the larger input.
/// Each difference is a one-byte range. Both streams use 64 KiB buffers; counts
/// remain exact even when the result listing reaches max_results. Requested work
/// beyond 64 MiB is rejected. Different files are a successful analysis result.
pub fn compare(
    left: &mut (impl Read + Seek),
    right: &mut (impl Read + Seek),
    range: ByteRange,
    max_results: usize,
) -> Result<Comparison, OperationError> {
    let result = (|| {
        check_results(max_results)?;
        let left_size = size(left)?;
        let right_size = size(right)?;
        let (offset, length) = range.resolve(left_size.max(right_size), MAX_SCAN_BYTES)?;
        left.seek(SeekFrom::Start(offset))?;
        right.seek(SeekFrom::Start(offset))?;
        let mut a = [0; 64 * 1024];
        let mut b = [0; 64 * 1024];
        let mut consumed = 0;
        let mut differences = Vec::new();
        let mut differing_bytes = 0;
        while consumed < length {
            let start = offset + consumed;
            let count = (length - consumed).min(a.len() as u64) as usize;
            let ac = (left_size.saturating_sub(start).min(count as u64)) as usize;
            let bc = (right_size.saturating_sub(start).min(count as u64)) as usize;
            left.read_exact(&mut a[..ac])?;
            right.read_exact(&mut b[..bc])?;
            for index in 0..count {
                let left = (index < ac).then(|| a[index]);
                let right = (index < bc).then(|| b[index]);
                if left != right {
                    differing_bytes += 1;
                    if differences.len() < max_results {
                        differences.push(Difference {
                            offset: start + index as u64,
                            left,
                            right,
                        });
                    }
                }
            }
            consumed += count as u64;
        }
        Ok(Comparison {
            equal: differing_bytes == 0,
            left_size,
            right_size,
            complete: differing_bytes == differences.len() as u64,
            differing_bytes,
            differences,
        })
    })();
    result.map_err(|e: OperationError| e.operation("file.binary.compare"))
}

const HEX_PARAMETERS: &[crate::operations::Parameter] = &[
    super::OFFSET_PARAMETER,
    super::LENGTH_PARAMETER,
    crate::operations::Parameter {
        name: "width",
        description: "Bytes per hexadecimal row.",
        required: false,
        default: Some(crate::operations::ParameterValue::Unsigned(16)),
        constraints: crate::operations::ParameterType::Unsigned { min: 1, max: 256 },
    },
];
const STRING_PARAMETERS: &[crate::operations::Parameter] = &[
    super::OFFSET_PARAMETER,
    super::LENGTH_PARAMETER,
    super::RESULT_PARAMETER,
    crate::operations::Parameter {
        name: "min_length",
        description: "Minimum number of consecutive ASCII printable bytes.",
        required: false,
        default: Some(crate::operations::ParameterValue::Unsigned(4)),
        constraints: crate::operations::ParameterType::Unsigned {
            min: 1,
            max: 1048576,
        },
    },
];
const COMPARE_PARAMETERS: &[crate::operations::Parameter] = &[
    super::OFFSET_PARAMETER,
    super::LENGTH_PARAMETER,
    super::RESULT_PARAMETER,
];
/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "file.hex.inspect",
        category: "file",
        description: "Display exact bytes as bounded hex rows with absolute offsets.",
        input_type: "byte_reader",
        output_type: "file.hex_rows",
        command: "file hex",
        example: "cube file hex sample.bin --length 256",
        parameters: HEX_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
    crate::operations::Descriptor {
        id: "file.strings.extract",
        category: "file",
        description: "Extract ASCII printable runs and exact byte offsets.",
        input_type: "byte_reader",
        output_type: "file.strings",
        command: "file strings",
        example: "cube file strings sample.bin",
        parameters: STRING_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
    crate::operations::Descriptor {
        id: "file.binary.compare",
        category: "file",
        description: "Compare corresponding byte offsets; missing tail bytes remain distinct from zero.",
        input_type: "byte_reader_pair",
        output_type: "file.comparison",
        command: "file compare",
        example: "cube file compare first.bin second.bin",
        parameters: COMPARE_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
];
