//! Typed structural inspectors for bounded, inert byte buffers.
pub mod bmp;
pub mod executable;
pub mod mpeg;
pub mod ogg;
pub mod tiff;
pub mod webp;

use super::{limit, malformed};
use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Serialize;

const MIB: usize = 1 << 20;
pub const MAX_SOURCE: usize = 32 * MIB;
const MAX_REPORT: usize = 4 * MIB;

#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Extent {
    pub offset: u64,
    pub length: u64,
}

impl Extent {
    fn new(offset: usize, length: usize) -> Self {
        Self {
            offset: offset as u64,
            length: length as u64,
        }
    }
}

fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message)
}

fn check_source(bytes: &[u8]) -> Result<(), OperationError> {
    if bytes.len() > MAX_SOURCE {
        return Err(limit("file source exceeds 32 MiB"));
    }
    Ok(())
}

fn bounded_report<T: Serialize>(report: T, maximum: usize) -> Result<T, OperationError> {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(data.len())
                .ok_or(std::io::ErrorKind::FileTooLarge)?;
            Ok(data.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(maximum), &report)
        .map_err(|_| limit("file report exceeds its bound"))?;
    Ok(report)
}

fn slice(data: &[u8], offset: usize, length: usize) -> Result<&[u8], OperationError> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| malformed("file range overflows"))?;
    data.get(offset..end)
        .ok_or_else(|| malformed("file range exceeds its source").offset(offset as u64))
}

fn le16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(data[offset..offset + 2].try_into().expect("checked header"))
}

fn le32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().expect("checked header"))
}

fn le64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().expect("checked header"))
}
