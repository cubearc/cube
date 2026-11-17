//! Explicit SP800-38A byte modes. These modes do not authenticate messages.
mod backend;
mod catalog;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub const MAX_PARAMETERS: usize = 2048;
pub const ENCRYPT: &str = "cipher.aes.mode.encrypt";
pub const DECRYPT: &str = "cipher.aes.mode.decrypt";
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Padding {
    None,
    Pkcs7,
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum Parameters {
    Ecb {
        #[schemars(schema_with = "key_schema")]
        key_hex: String,
        padding: Padding,
    },
    Cbc {
        #[schemars(schema_with = "key_schema")]
        key_hex: String,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
        padding: Padding,
    },
    Cfb128 {
        #[schemars(schema_with = "key_schema")]
        key_hex: String,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    Ofb {
        #[schemars(schema_with = "key_schema")]
        key_hex: String,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    Ctr128Be {
        #[schemars(schema_with = "key_schema")]
        key_hex: String,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
}
impl Drop for Parameters {
    fn drop(&mut self) {
        match self {
            Self::Ecb { key_hex, .. }
            | Self::Cbc { key_hex, .. }
            | Self::Cfb128 { key_hex, .. }
            | Self::Ofb { key_hex, .. }
            | Self::Ctr128Be { key_hex, .. } => key_hex.zeroize(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum Mode {
    Ecb(Padding),
    Cbc(Padding),
    Cfb128,
    Ofb,
    Ctr128Be,
}
/// Validated caller settings, with owned key wiped on drop and redacted Debug.
pub struct Settings {
    mode: Mode,
    key: Zeroizing<Vec<u8>>,
    iv: [u8; 16],
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AesModeSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Parse required explicit mode, key, and mode-specific IV/padding.
    /// No secret input is included in an error. JSON is limited to 2048 bytes.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit("AES mode settings exceed 2048 bytes"));
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        let (mode, key, iv) = match &p {
            Parameters::Ecb { key_hex, padding } => (Mode::Ecb(*padding), key_hex, None),
            Parameters::Cbc {
                key_hex,
                iv_hex,
                padding,
            } => (Mode::Cbc(*padding), key_hex, Some(iv_hex)),
            Parameters::Cfb128 { key_hex, iv_hex } => (Mode::Cfb128, key_hex, Some(iv_hex)),
            Parameters::Ofb { key_hex, iv_hex } => (Mode::Ofb, key_hex, Some(iv_hex)),
            Parameters::Ctr128Be { key_hex, iv_hex } => (Mode::Ctr128Be, key_hex, Some(iv_hex)),
        };
        if !matches!(key.len(), 32 | 48 | 64) {
            return Err(invalid());
        }
        let mut decoded = Zeroizing::new(vec![0; key.len() / 2]);
        hex::decode_to_slice(key, &mut decoded[..]).map_err(|_| invalid())?;
        let mut nonce = [0; 16];
        if let Some(iv) = iv {
            hex::decode_to_slice(iv, &mut nonce).map_err(|_| invalid())?;
        }
        Ok(Self {
            mode,
            key: decoded,
            iv: nonce,
        })
    }
}
/// Encrypt exact bytes with an explicit unauthenticated AES mode.
///
/// ECB/CBC require explicit `none` or PKCS#7 padding. Feedback modes preserve
/// length, including a partial final block. CTR increments the entire 128-bit
/// big-endian IV and refuses wrap before allocation. Caller manages IV safety.
pub fn encrypt(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, false)
}
/// Decrypt only after whole-buffer padding validation; padding is not integrity.
/// All padding failures return the same static error and no partial plaintext.
pub fn decrypt(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, true)
}
fn transform(data: &[u8], s: &Settings, inverse: bool) -> Result<Vec<u8>, OperationError> {
    let padded = matches!(
        s.mode,
        Mode::Ecb(Padding::Pkcs7) | Mode::Cbc(Padding::Pkcs7)
    );
    if data.len() > MAX_INPUT + if inverse && padded { 16 } else { 0 } {
        return Err(limit("AES mode input exceeds its 16 MiB body bound"));
    }
    if matches!(s.mode, Mode::Ecb(_) | Mode::Cbc(_))
        && (inverse || !padded)
        && !data.len().is_multiple_of(16)
    {
        return Err(failed());
    }
    if inverse && padded && data.is_empty() {
        return Err(failed());
    }
    if matches!(s.mode, Mode::Ctr128Be)
        && !data.is_empty()
        && u128::from_be_bytes(s.iv)
            .checked_add(data.len().div_ceil(16) as u128 - 1)
            .is_none()
    {
        return Err(limit("AES counter would wrap"));
    }
    backend::transform(data, s, inverse)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use an exact AES mode, 16/24/32-byte hexadecimal key and required mode-specific IV/padding",
    )
}
fn limit(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}
fn failed() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "invalid AES mode ciphertext or padding",
    )
}

fn key_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    hex_schema("^(?:[0-9A-Fa-f]{32}|[0-9A-Fa-f]{48}|[0-9A-Fa-f]{64})$")
}
fn iv_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    hex_schema("^[0-9A-Fa-f]{32}$")
}
fn hex_schema(pattern: &str) -> schemars::Schema {
    schemars::Schema::try_from(serde_json::json!({"type":"string","pattern":pattern}))
        .expect("hexadecimal string schema")
}
