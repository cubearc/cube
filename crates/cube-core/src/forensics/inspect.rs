//! Filesystem facts and header evidence, without EXIF disclosure or pixel decoding.

use super::{open_regular, signatures};
use crate::errors::OperationError;
use serde::Serialize;
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

/// Exact timestamp duration relative to the Unix epoch. Unavailable times are None.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Timestamp {
    pub seconds: u64,
    pub nanoseconds: u32,
    pub before_unix_epoch: bool,
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:09} seconds {} Unix epoch (UTC)",
            self.seconds,
            self.nanoseconds,
            if self.before_unix_epoch {
                "before"
            } else {
                "after"
            }
        )
    }
}
fn timestamp(time: std::io::Result<SystemTime>) -> Option<Timestamp> {
    time.ok().map(|time| {
        let (duration, before_unix_epoch) = match time.duration_since(UNIX_EPOCH) {
            Ok(d) => (d, false),
            Err(e) => (e.duration(), true),
        };
        Timestamp {
            seconds: duration.as_secs(),
            nanoseconds: duration.subsec_nanos(),
            before_unix_epoch,
        }
    })
}

/// Basic file facts; unavailable filesystem times are null rather than zero.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct FileInfo {
    pub size: u64,
    pub regular_file: bool,
    pub modified: Option<Timestamp>,
    pub created: Option<Timestamp>,
    pub signatures: Vec<signatures::Signature>,
}

/// Evidence from an owned byte artifact. Filesystem-only facts stay unknown.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactInfo {
    pub source_kind: ArtifactKind,
    pub size: u64,
    pub regular_file: (),
    pub modified: (),
    pub created: (),
    pub signatures: Vec<signatures::Signature>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Artifact,
}
/// Inspect supplied bytes without resolving paths or claiming OS metadata.
pub fn artifact_info(data: &[u8]) -> ArtifactInfo {
    ArtifactInfo {
        source_kind: ArtifactKind::Artifact,
        size: data.len() as u64,
        regular_file: (),
        modified: (),
        created: (),
        signatures: signatures::detect(&data[..data.len().min(12)]),
    }
}
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
pub enum FileSource {
    Reference(crate::operations::contracts::values::FileReference),
    Bytes(crate::operations::contracts::values::Bytes),
}
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
pub enum Inspection {
    File(FileInfo),
    Artifact(ArtifactInfo),
}

/// Read basic metadata and up to 12 header bytes. Symlinks, directories and special
/// files are rejected; no recursive traversal or sensitive EXIF inspection occurs.
pub fn file_info(path: &Path) -> Result<FileInfo, OperationError> {
    let result = (|| {
        let mut file = open_regular(path)?;
        file_info_reader(&mut file)
    })();
    result.map_err(|e: OperationError| e.operation("file.info.inspect"))
}

/// Inspect a regular file handle already opened by the caller's filesystem
/// capability. Reads at most 12 header bytes from offset zero; paths are never
/// resolved here. The caller owns authorization and source stability.
pub fn file_info_reader(file: &mut std::fs::File) -> Result<FileInfo, OperationError> {
    let result = (|| {
        if !file.metadata()?.is_file() {
            return Err(super::invalid("input handle must be a regular file"));
        }
        file.seek(SeekFrom::Start(0))?;
        let metadata = file.metadata()?;
        let mut header = Vec::new();
        (&mut *file).take(12).read_to_end(&mut header)?;
        Ok(FileInfo {
            size: metadata.len(),
            regular_file: true,
            modified: timestamp(metadata.modified()),
            created: timestamp(metadata.created()),
            signatures: signatures::detect(&header),
        })
    })();
    result.map_err(|e: OperationError| e.operation("file.info.inspect"))
}

/// Operation descriptors maintained beside the implementation.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.info.inspect",
    category: "file",
    description: "Inspect file or supplied byte-artifact size and bounded header evidence; artifact filesystem facts remain unknown; no EXIF.",
    input_type: "file_path",
    output_type: "file.info",
    command: "inspect",
    example: "cube inspect sample.bin",
    parameters: &[],
    deterministic: false,
    reversible: None,
}];

fn fixture() -> Result<crate::operations::contracts::Fixture, OperationError> {
    use serde_json::json;
    Ok(crate::operations::contracts::Fixture {
        inputs: json!({"input":{"base64":"AA=="}}),
        parameters: json!({}),
        outputs: json!({"output":artifact_info(&[0])}),
    })
}
pub(crate) const CONTRACTS: &[crate::operations::contracts::Entry] =
    &[crate::operations::contracts::Entry {
        operation_version: 2,
        descriptor: OPERATIONS[0],
        inputs: &[crate::operations::contracts::Port {
            name: "input",
            semantic_type: "file.source",
            required: true,
            schema: crate::operations::contracts::schema::<FileSource>,
        }],
        parameters: || crate::operations::contracts::descriptor_parameters(&OPERATIONS[0]),
        outputs: &[crate::operations::contracts::Port {
            name: "output",
            semantic_type: "file.info",
            required: true,
            schema: crate::operations::contracts::schema::<Inspection>,
        }],
        limits: crate::operations::contracts::Limits {
            max_input_bytes: None,
            max_output_bytes: None,
            max_items: None,
            max_work: Some(12),
            max_memory_bytes: None,
        },
        fixture,
    }];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timestamp_distinguishes_absence_and_pre_epoch() {
        assert!(timestamp(Err(std::io::Error::from(std::io::ErrorKind::Unsupported))).is_none());
        let result = timestamp(Ok(UNIX_EPOCH - std::time::Duration::from_millis(500))).unwrap();
        assert!(result.before_unix_epoch);
        assert_eq!(result.nanoseconds, 500_000_000);
    }
}
