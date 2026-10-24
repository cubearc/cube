//! Small, deterministic operations shared by Cube's CLI and SDKs.
//!
//! Binary operations accept and return bytes. Text conversion belongs to the caller.
//! Classical ciphers and repeating-key XOR are puzzle tools, not secure encryption.

pub mod binary;
pub mod ciphers;
pub mod encoding;
pub mod errors;
pub mod forensics;
pub mod hashing;
pub mod image;
pub mod media;
pub mod numbers;
pub mod operations;
pub mod pipeline;
pub mod symbols;
pub mod text;

use std::fmt;

/// Invalid input to a core operation.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Input is not canonical, padded RFC 4648 standard Base32.
    InvalidBase32(data_encoding::DecodeError),
    /// Input contains a character outside the Bitcoin Base58 alphabet.
    InvalidBase58(bs58::decode::Error),
    /// Input is not canonical, padded RFC 4648 standard Base64.
    InvalidBase64(base64::DecodeError),
    /// Input is not an even number of hexadecimal digits.
    InvalidHex(hex::FromHexError),
    /// Repeating-key XOR requires at least one key byte.
    EmptyKey,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase32(error) => write!(f, "invalid Base32: {error}"),
            Self::InvalidBase58(error) => write!(f, "invalid Base58: {error}"),
            Self::InvalidBase64(error) => write!(f, "invalid Base64: {error}"),
            Self::InvalidHex(error) => write!(f, "invalid hexadecimal input: {error}"),
            Self::EmptyKey => f.write_str("XOR key must not be empty"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidBase32(error) => Some(error),
            Self::InvalidBase58(error) => Some(error),
            Self::InvalidBase64(error) => Some(error),
            Self::InvalidHex(error) => Some(error),
            Self::EmptyKey => None,
        }
    }
}

impl Error {
    /// Stable category for legacy failures; existing variants and causes are retained.
    pub fn code(&self) -> errors::ErrorCode {
        errors::ErrorCode::InvalidInput
    }

    /// Operation that rejected the input, without copying the rejected value.
    pub fn operation_id(&self) -> &'static str {
        match self {
            Self::InvalidBase32(_) => "encoding.base32.decode",
            Self::InvalidBase58(_) => "encoding.base58.decode",
            Self::InvalidBase64(_) => "encoding.base64.decode",
            Self::InvalidHex(_) => "encoding.hex.decode",
            Self::EmptyKey => "binary.xor",
        }
    }
}
