//! Single-stream compression and bounded decompression for explicit wire formats.
mod catalog;
mod codecs;

pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::io::{self, Read, Write};

pub const ENCODE: &str = "binary.compress.encode";
pub const DECODE: &str = "binary.compress.decode";
pub const MAX_BYTES: usize = 16 << 20;
pub const MAX_RATIO: usize = 256;
pub const DEFAULT_LEVEL: u8 = 6;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Deflate,
    Zlib,
    Gzip,
    Brotli,
    Zstd,
    LzmaAlone,
}
impl Profile {
    pub const NAMES: &[&str] = &["deflate", "zlib", "gzip", "brotli", "zstd", "lzma_alone"];
    /// Parse an exact format name without guessing from input.
    pub fn parse(profile: &str, id: &'static str) -> Result<Self, OperationError> {
        match profile {
            "deflate" => Ok(Self::Deflate),
            "zlib" => Ok(Self::Zlib),
            "gzip" => Ok(Self::Gzip),
            "brotli" => Ok(Self::Brotli),
            "zstd" => Ok(Self::Zstd),
            "lzma_alone" => Ok(Self::LzmaAlone),
            _ => Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "select an exact compression format",
            )
            .operation(id)
            .parameter("profile")),
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EncodeSettings {
    pub profile: Profile,
    #[serde(default)]
    #[schemars(with = "u8", range(min = 0, max = 9))]
    pub level: Option<u8>,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecodeSettings {
    pub profile: Profile,
}

pub(crate) fn encoded_bound(length: usize) -> usize {
    (2 * length + 1024).min(MAX_BYTES)
}
pub(crate) fn decoded_bound(length: usize) -> usize {
    (MAX_RATIO * length).min(MAX_BYTES)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "invalid, incomplete, or trailing data for the selected compression format",
    )
    .operation(DECODE)
}
fn limit(id: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "compression input, output, window, or expansion ratio exceeds the profile limit",
    )
    .operation(id)
}
fn invalid_level() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "only deflate, zlib and gzip accept levels, in 0..=9",
    )
    .operation(ENCODE)
    .parameter("level")
}

struct Output {
    data: Vec<u8>,
    max: usize,
    overflowed: bool,
}
impl Output {
    fn new(max: usize) -> Self {
        Self {
            data: Vec::new(),
            max,
            overflowed: false,
        }
    }
    fn append(&mut self, bytes: &[u8]) -> bool {
        let length = self.data.len() + bytes.len();
        if self.overflowed || length > self.max {
            self.overflowed = true;
            return false;
        }
        if length > self.data.capacity() {
            let target = length
                .max(self.data.capacity().saturating_mul(2))
                .min(self.max);
            if self
                .data
                .try_reserve_exact(target - self.data.len())
                .is_err()
            {
                self.overflowed = true;
                return false;
            }
        }
        self.data.extend_from_slice(bytes);
        true
    }
}
impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // ruzstd 0.9 treats a writer failure as a panic. Discard excess
        // while the bounded encoder completes and fail before returning bytes.
        self.append(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn read_output(mut reader: impl Read, output: &mut Output) -> Result<(), OperationError> {
    let mut chunk = [0; 8192];
    loop {
        let capacity = chunk.len().min(output.max - output.data.len() + 1);
        let size = reader.read(&mut chunk[..capacity]).map_err(|_| invalid())?;
        if size == 0 {
            return Ok(());
        }
        if !output.append(&chunk[..size]) {
            return Err(limit(DECODE));
        }
    }
}

/// Compress with an exact format. The three Deflate formats default to level 6.
/// Fixed encoder settings are used by the three other formats.
pub fn compress(data: &[u8], settings: EncodeSettings) -> Result<Vec<u8>, OperationError> {
    if data.len() > MAX_BYTES {
        return Err(limit(ENCODE));
    }
    let level = match (settings.profile, settings.level) {
        (Profile::Deflate | Profile::Zlib | Profile::Gzip, None) => DEFAULT_LEVEL,
        (Profile::Deflate | Profile::Zlib | Profile::Gzip, Some(level @ 0..=9)) => level,
        (_, None) => DEFAULT_LEVEL,
        _ => return Err(invalid_level()),
    };
    let mut output = Output::new(encoded_bound(data.len()));
    codecs::encode(data, settings.profile, level, &mut output).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "compression failed for the selected profile",
        )
        .operation(ENCODE)
    })?;
    if output.overflowed || data.len() > decoded_bound(output.data.len()) {
        return Err(limit(ENCODE));
    }
    Ok(output.data)
}

/// Decode exactly one stream, capped at 16 MiB, a 16 MiB window or dictionary,
/// and a ratio of 256 decoded bytes per encoded byte.
pub fn decompress(data: &[u8], settings: DecodeSettings) -> Result<Vec<u8>, OperationError> {
    if data.len() > MAX_BYTES {
        return Err(limit(DECODE));
    }
    let mut output = Output::new(decoded_bound(data.len()));
    codecs::decode(data, settings.profile, &mut output)?;
    Ok(output.data)
}
