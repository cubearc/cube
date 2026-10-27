//! Small explicit signature rules. Matches are candidates, never validity proofs.

use super::{ByteRange, MAX_SCAN_BYTES, check_results, invalid, size};
use crate::errors::OperationError;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};

struct Rule {
    format: &'static str,
    mime: &'static str,
    prefix: &'static [u8],
    suffix: &'static [u8],
    suffix_offset: usize,
}
const RULES: &[Rule] = &[
    Rule {
        format: "png",
        mime: "image/png",
        prefix: b"\x89PNG\r\n\x1a\n",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "jpeg",
        mime: "image/jpeg",
        prefix: b"\xff\xd8\xff",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "gif",
        mime: "image/gif",
        prefix: b"GIF87a",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "gif",
        mime: "image/gif",
        prefix: b"GIF89a",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "pdf",
        mime: "application/pdf",
        prefix: b"%PDF-",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "zip",
        mime: "application/zip",
        prefix: b"PK\x03\x04",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "zip",
        mime: "application/zip",
        prefix: b"PK\x05\x06",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "gzip",
        mime: "application/gzip",
        prefix: b"\x1f\x8b\x08",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "wave",
        mime: "audio/wav",
        prefix: b"RIFF",
        suffix: b"WAVE",
        suffix_offset: 8,
    },
    Rule {
        format: "tiff",
        mime: "image/tiff",
        prefix: b"II\x2a\x00",
        suffix: b"",
        suffix_offset: 0,
    },
    Rule {
        format: "tiff",
        mime: "image/tiff",
        prefix: b"MM\x00\x2a",
        suffix: b"",
        suffix_offset: 0,
    },
];

/// Matched public signature rule and its exact evidence; this does not validate a file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct Signature {
    pub format: &'static str,
    pub mime: &'static str,
    pub offset: u64,
    pub evidence_hex: String,
    pub rule: &'static str,
}

/// Identify header candidates from at most the first 12 bytes. Unknown/truncated
/// headers produce an empty list; filenames and extensions are never consulted.
pub fn detect(input: &[u8]) -> Vec<Signature> {
    at(input, 0)
}

fn at(input: &[u8], offset: u64) -> Vec<Signature> {
    RULES
        .iter()
        .filter(|rule| {
            input.starts_with(rule.prefix)
                && input.get(rule.suffix_offset..rule.suffix_offset + rule.suffix.len())
                    == Some(rule.suffix)
        })
        .map(|rule| Signature {
            format: rule.format,
            mime: rule.mime,
            offset,
            evidence_hex: hex::encode(
                &input[..rule
                    .prefix
                    .len()
                    .max(rule.suffix_offset + rule.suffix.len())],
            ),
            rule: if rule.format == "wave" {
                "RIFF at +0 and WAVE at +8"
            } else {
                "fixed prefix at +0"
            },
        })
        .collect()
}

/// Bounded embedded-candidate scan with absolute offsets and explicit completeness.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Scan {
    pub candidates: Vec<Signature>,
    pub scanned_bytes: u64,
    pub complete: bool,
}

/// Scan a selected range, including overlapping matches, in offset/rule order.
/// One MiB blocks retain an 11-byte overlap. Format filters are exact names from
/// `detect`; unknown names fail. Reaching max_results returns an incomplete scan.
pub fn scan(
    reader: &mut (impl Read + Seek),
    range: ByteRange,
    format: Option<&str>,
    max_results: usize,
) -> Result<Scan, OperationError> {
    let result = (|| {
        check_results(max_results)?;
        if format.is_some_and(|format| !RULES.iter().any(|rule| rule.format == format)) {
            return Err(invalid("unknown signature format filter").parameter("format"));
        }
        let total = size(reader)?;
        let (offset, length) = range.resolve(total, u64::MAX)?;
        let selected = length.min(MAX_SCAN_BYTES);
        reader.seek(SeekFrom::Start(offset))?;
        let mut candidates = Vec::new();
        let mut pending = Vec::new();
        let mut consumed = 0;
        let mut base = offset;
        let mut buffer = [0; 64 * 1024];
        while consumed < selected {
            let count = (selected - consumed).min(buffer.len() as u64) as usize;
            reader.read_exact(&mut buffer[..count])?;
            consumed += count as u64;
            pending.extend_from_slice(&buffer[..count]);
            let inspect = if consumed == selected {
                pending.len()
            } else {
                pending.len().saturating_sub(11)
            };
            for index in 0..inspect {
                for candidate in at(&pending[index..], base + index as u64) {
                    if format.is_none_or(|format| candidate.format == format) {
                        if candidates.len() == max_results {
                            return Ok(Scan {
                                candidates,
                                scanned_bytes: base + index as u64 - offset,
                                complete: false,
                            });
                        }
                        candidates.push(candidate);
                    }
                }
            }
            pending.drain(..inspect);
            base += inspect as u64;
        }
        Ok(Scan {
            candidates,
            scanned_bytes: selected,
            complete: selected == length,
        })
    })();
    result.map_err(|e: OperationError| e.operation("file.signature.scan"))
}

const SCAN_PARAMETERS: &[crate::operations::Parameter] = &[
    super::OFFSET_PARAMETER,
    super::LENGTH_PARAMETER,
    super::RESULT_PARAMETER,
    crate::operations::Parameter {
        name: "format",
        description: "Exact signature format filter; omitted selects all supported rules.",
        required: false,
        default: None,
        constraints: crate::operations::ParameterType::Text {
            choices: &[
                RULES[0].format,
                RULES[1].format,
                RULES[2].format,
                RULES[4].format,
                RULES[5].format,
                RULES[7].format,
                RULES[8].format,
                RULES[9].format,
            ],
        },
    },
];
/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "file.signature.detect",
        category: "file",
        description: "Detect header signatures as evidence, not structural validity.",
        input_type: "bytes",
        output_type: "file.signatures",
        command: "file signature",
        example: "cube file signature sample.bin",
        parameters: &[],
        deterministic: true,
        reversible: None,
    },
    crate::operations::Descriptor {
        id: "file.signature.scan",
        category: "file",
        description: "Locate bounded embedded signature candidates with absolute offsets.",
        input_type: "byte_reader",
        output_type: "file.signature_scan",
        command: "file embedded",
        example: "cube file embedded sample.bin",
        parameters: SCAN_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
];
