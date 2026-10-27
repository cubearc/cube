//! Filesystem facts and header evidence, without EXIF disclosure or pixel decoding.

use super::{open_regular, signatures};
use crate::errors::OperationError;
use serde::Serialize;
use std::{
    io::Read,
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

/// Read basic metadata and up to 12 header bytes. Symlinks, directories and special
/// files are rejected; no recursive traversal or sensitive EXIF inspection occurs.
pub fn file_info(path: &Path) -> Result<FileInfo, OperationError> {
    let result = (|| {
        let mut file = open_regular(path)?;
        let metadata = file.metadata()?;
        let mut header = Vec::new();
        (&mut file).take(12).read_to_end(&mut header)?;
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
    description: "Inspect basic filesystem facts and bounded header evidence; no EXIF.",
    input_type: "file_path",
    output_type: "file.info",
    command: "inspect",
    example: "cube inspect sample.bin",
    parameters: &[],
    deterministic: false,
    reversible: None,
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
