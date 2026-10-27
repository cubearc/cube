//! Explicit, non-destructive recovery using the existing bounded byte-range copy.
use super::{ByteRange, MAX_COPY_BYTES, boundaries, copy_range, invalid, malformed, open_regular};
use crate::errors::OperationError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{Metadata, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};
pub const DEFAULT_MAX_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub enum Selection {
    /// Exact caller-selected bytes, without a format-validation claim.
    Range { offset: u64, length: u64 },
    /// Revalidate supported framing at this explicit origin before copying.
    Candidate { offset: u64 },
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Extraction {
    pub source_size: u64,
    pub offset: u64,
    pub length: u64,
    pub bytes_written: u64,
    pub selection: &'static str,
    pub format: Option<&'static str>,
    pub evidence: &'static str,
    /// SHA-256 of the exact bytes written, not a trust/format-validity assertion.
    pub sha256: String,
}

struct DigestOutput {
    file: std::fs::File,
    digest: Sha256,
}
impl Write for DigestOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let count = self.file.write(bytes)?;
        self.digest.update(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// Reader coordinates relative to an explicitly selected origin; no allocation,
/// path resolution or extraction capability is introduced into pipeline values.
struct Origin<'a, R> {
    reader: &'a mut R,
    base: u64,
    length: u64,
    pos: u64,
}
impl<R: Read + Seek> Read for Origin<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let n = (self.length - self.pos).min(bytes.len() as u64) as usize;
        let count = self.reader.read(&mut bytes[..n])?;
        if count > n {
            return Err(std::io::Error::other(
                "reader exceeded selected origin bounds",
            ));
        }
        self.pos += count as u64;
        Ok(count)
    }
}
impl<R: Seek> Seek for Origin<'_, R> {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        let p = match from {
            SeekFrom::Start(n) => n as i128,
            SeekFrom::End(n) => self.length as i128 + n as i128,
            SeekFrom::Current(n) => self.pos as i128 + n as i128,
        };
        if p < 0 || p > self.length as i128 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek exceeds selected origin bounds",
            ));
        }
        let p = p as u64;
        let absolute = self
            .base
            .checked_add(p)
            .ok_or_else(|| std::io::Error::other("origin offset overflow"))?;
        self.reader.seek(SeekFrom::Start(absolute))?;
        self.pos = p;
        Ok(p)
    }
}
fn unchanged(a: &Metadata, b: &Metadata) -> bool {
    a.len() == b.len() && a.modified().ok() == b.modified().ok()
}

/// Recover a selected range or freshly validated PNG/JPEG/GIF/ZIP candidate to a
/// new destination. Validate range and caller output budget before creation;
/// never overwrite a destination, decompress archives or execute content. Working
/// copying memory is 64 KiB; validation has the owning format's separate bounds.
/// The hard output limit is 1 GiB. Only a newly created partial output is removed
/// on failure. Metadata changes during recovery are rejected; this is not an
/// atomic filesystem snapshot, so callers must keep the source stable.
pub fn carve(
    source: &Path,
    destination: &Path,
    selection: Selection,
    max_bytes: u64,
) -> Result<Extraction, OperationError> {
    let result = (|| {
        if max_bytes > MAX_COPY_BYTES {
            return Err(
                invalid("carving budget exceeds the 1 GiB hard output limit")
                    .parameter("max_bytes"),
            );
        }
        let mut input = open_regular(source)?;
        let before = input.metadata()?;
        let source_size = before.len();
        let SelectedRange {
            offset,
            length,
            mode,
            format,
            evidence,
        } = selected(&mut input, selection, source_size)?;
        ByteRange {
            offset,
            length: Some(length),
        }
        .resolve(source_size, max_bytes)?;
        if !unchanged(&before, &input.metadata()?) {
            return Err(malformed("source changed during candidate validation"));
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        let mut output = DigestOutput {
            file,
            digest: Sha256::new(),
        };
        let copied = (|| {
            let bytes_written = copy_range(&mut input, &mut output, offset, length)?;
            if !unchanged(&before, &input.metadata()?) {
                return Err(malformed("source changed during recovery"));
            }
            Ok(Extraction {
                source_size,
                offset,
                length,
                bytes_written,
                selection: mode,
                format,
                evidence,
                sha256: crate::encoding::hex_encode(&output.digest.clone().finalize()),
            })
        })();
        drop(output);
        if copied.is_err() {
            let _cleanup = std::fs::remove_file(destination);
        }
        copied
    })();
    result.map_err(|e: OperationError| e.operation("file.carve"))
}

struct SelectedRange {
    offset: u64,
    length: u64,
    mode: &'static str,
    format: Option<&'static str>,
    evidence: &'static str,
}
fn selected(
    input: &mut (impl Read + Seek),
    selection: Selection,
    source_size: u64,
) -> Result<SelectedRange, OperationError> {
    Ok(match selection {
        Selection::Range { offset, length } => SelectedRange {
            offset,
            length,
            mode: "range",
            format: None,
            evidence: "Caller-selected exact byte range; no file-structure validation",
        },
        Selection::Candidate { offset } => {
            let length = source_size.checked_sub(offset).ok_or_else(|| {
                invalid("candidate origin extends beyond EOF").parameter("offset")
            })?;
            input.seek(SeekFrom::Start(offset))?;
            let mut origin = Origin {
                reader: input,
                base: offset,
                length,
                pos: 0,
            };
            let report = boundaries::inspect(&mut origin).map_err(|mut error| {
                error.offset = error
                    .offset
                    .and_then(|relative| offset.checked_add(relative));
                error
            })?;
            SelectedRange {
                offset,
                length: report.logical_end,
                mode: "candidate",
                format: Some(report.format),
                evidence: report.evidence,
            }
        }
    })
}

/// Recover exact bytes from a caller-authorized stable reader, with the same
/// candidate validation and provenance as `carve`. No filesystem write occurs.
/// The in-memory API is capped at 16 MiB; callers may lower the output budget.
pub fn carve_reader(
    input: &mut (impl Read + Seek),
    selection: Selection,
    max_bytes: u64,
) -> Result<(Extraction, Vec<u8>), OperationError> {
    let result = (|| {
        if max_bytes > super::MAX_OUTPUT_BYTES {
            return Err(invalid("in-memory carving budget exceeds 16 MiB").parameter("max_bytes"));
        }
        let source_size = input.seek(SeekFrom::End(0))?;
        let SelectedRange {
            offset,
            length,
            mode,
            format,
            evidence,
        } = selected(input, selection, source_size)?;
        ByteRange {
            offset,
            length: Some(length),
        }
        .resolve(source_size, max_bytes)?;
        let data = super::read_range(
            input,
            ByteRange {
                offset,
                length: Some(length),
            },
        )?;
        let report = Extraction {
            source_size,
            offset,
            length,
            bytes_written: data.len() as u64,
            selection: mode,
            format,
            evidence,
            sha256: crate::hashing::sha256(&data),
        };
        Ok((report, data))
    })();
    result.map_err(|e: OperationError| e.operation("file.carve"))
}

pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.carve",
    category: "file",
    description: "Recover an explicit range or freshly validated framing candidate to a new file with exact range provenance; no decompression.",
    input_type: "file.carve_request",
    output_type: "file.carving",
    deterministic: true,
    reversible: None,
    command: "file carve",
    example: "cube file carve sample.bin --offset 128 --output recovered.zip --json",
    parameters: &[
        crate::operations::Parameter {
            required: true,
            default: None,
            ..super::OFFSET_PARAMETER
        },
        crate::operations::Parameter {
            description: "Exact caller-selected length; omit to validate a supported candidate at offset.",
            ..super::LENGTH_PARAMETER
        },
        crate::operations::Parameter {
            name: "max_bytes",
            description: "Maximum output bytes, checked before destination creation.",
            required: false,
            default: Some(crate::operations::ParameterValue::Unsigned(
                DEFAULT_MAX_BYTES,
            )),
            constraints: crate::operations::ParameterType::Unsigned {
                min: 0,
                max: MAX_COPY_BYTES,
            },
        },
    ],
}];

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn relative_reader_seeks_and_reads_cannot_escape_origin() {
        let mut reader = Cursor::new(b"hostabc");
        let mut origin = Origin {
            reader: &mut reader,
            base: 4,
            length: 3,
            pos: 0,
        };
        origin.seek(SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        origin.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"abc");
        assert!(origin.seek(SeekFrom::Current(-4)).is_err());
        assert!(origin.seek(SeekFrom::End(1)).is_err());
        assert!(origin.seek(SeekFrom::Start(u64::MAX)).is_err());
        origin.seek(SeekFrom::End(-1)).unwrap();
        let mut b = [0];
        origin.read_exact(&mut b).unwrap();
        assert_eq!(b, b"c"[..]);
        let mut invalid = Origin {
            reader: &mut reader,
            base: u64::MAX,
            length: 1,
            pos: 0,
        };
        assert!(invalid.seek(SeekFrom::Start(1)).is_err());
    }
}
