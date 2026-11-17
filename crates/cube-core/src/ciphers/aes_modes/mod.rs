//! Explicit AES byte modes, ciphertext stealing and storage data units.
//! These transformations do not authenticate messages.
mod backend;
mod catalog;
mod stealing;
mod storage;
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
#[serde(transparent)]
struct SecretHex(String);
impl Drop for SecretHex {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum Parameters {
    Ecb {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        padding: Padding,
    },
    Cbc {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
        padding: Padding,
    },
    Cfb128 {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    Ofb {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    Ctr128Be {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    CbcCs1 {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    CbcCs2 {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    CbcCs3 {
        #[schemars(schema_with = "key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        iv_hex: String,
    },
    #[serde(rename = "xts_aes128")]
    XtsAes128 {
        #[schemars(schema_with = "storage128_key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        tweak_hex: String,
    },
    #[serde(rename = "xts_aes256")]
    XtsAes256 {
        #[schemars(schema_with = "storage256_key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        tweak_hex: String,
    },
    #[serde(rename = "xex_aes128_ieee1619_full_blocks")]
    XexAes128 {
        #[schemars(schema_with = "storage128_key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        tweak_hex: String,
    },
    #[serde(rename = "xex_aes256_ieee1619_full_blocks")]
    XexAes256 {
        #[schemars(schema_with = "storage256_key_schema")]
        key_hex: SecretHex,
        #[schemars(schema_with = "iv_schema")]
        tweak_hex: String,
    },
}
#[derive(Clone, Copy, Debug)]
enum Mode {
    Ecb(Padding),
    Cbc(Padding),
    Cfb128,
    Ofb,
    Ctr128Be,
    Stealing(Stealing),
    Storage(Storage),
}
#[derive(Clone, Copy, Debug)]
enum Stealing {
    Cs1,
    Cs2,
    Cs3,
}
#[derive(Clone, Copy, Debug)]
enum Storage {
    Xts128,
    Xts256,
    Xex128,
    Xex256,
}
impl Storage {
    fn key_size(self) -> usize {
        match self {
            Self::Xts128 | Self::Xex128 => 32,
            Self::Xts256 | Self::Xex256 => 64,
        }
    }
    fn full_blocks(self) -> bool {
        matches!(self, Self::Xex128 | Self::Xex256)
    }
}
/// Validated caller settings, with owned key wiped on drop and redacted Debug.
pub struct Settings {
    mode: Mode,
    key: Zeroizing<Vec<u8>>,
    initial_block: [u8; 16],
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AesModeSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Parse required explicit mode, key, and mode-specific IV/tweak/padding.
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
            Parameters::CbcCs1 { key_hex, iv_hex } => {
                (Mode::Stealing(Stealing::Cs1), key_hex, Some(iv_hex))
            }
            Parameters::CbcCs2 { key_hex, iv_hex } => {
                (Mode::Stealing(Stealing::Cs2), key_hex, Some(iv_hex))
            }
            Parameters::CbcCs3 { key_hex, iv_hex } => {
                (Mode::Stealing(Stealing::Cs3), key_hex, Some(iv_hex))
            }
            Parameters::XtsAes128 { key_hex, tweak_hex } => {
                (Mode::Storage(Storage::Xts128), key_hex, Some(tweak_hex))
            }
            Parameters::XtsAes256 { key_hex, tweak_hex } => {
                (Mode::Storage(Storage::Xts256), key_hex, Some(tweak_hex))
            }
            Parameters::XexAes128 { key_hex, tweak_hex } => {
                (Mode::Storage(Storage::Xex128), key_hex, Some(tweak_hex))
            }
            Parameters::XexAes256 { key_hex, tweak_hex } => {
                (Mode::Storage(Storage::Xex256), key_hex, Some(tweak_hex))
            }
        };
        let key = key.0.as_str();
        let valid = match mode {
            Mode::Storage(kind) => key.len() == kind.key_size() * 2,
            _ => matches!(key.len(), 32 | 48 | 64),
        };
        if !valid {
            return Err(invalid());
        }
        let mut decoded = Zeroizing::new(vec![0; key.len() / 2]);
        hex::decode_to_slice(key, &mut decoded[..]).map_err(|_| invalid())?;
        if let Mode::Storage(_) = mode {
            let half = decoded.len() / 2;
            if decoded[..half] == decoded[half..] {
                return Err(invalid());
            }
        }
        let mut nonce = [0; 16];
        if let Some(iv) = iv {
            hex::decode_to_slice(iv, &mut nonce).map_err(|_| invalid())?;
        }
        Ok(Self {
            mode,
            key: decoded,
            initial_block: nonce,
        })
    }
}
/// Encrypt exact bytes with an explicit unauthenticated AES mode.
///
/// ECB/CBC require explicit `none` or PKCS#7 padding. Feedback modes preserve
/// length, including a partial final block. CTR increments the entire 128-bit
/// big-endian IV and refuses wrap before allocation. CTS and XTS accept at
/// least16bytes; the named IEEE1619 XEX profiles require whole blocks. Storage
/// keys contain two distinct16/32-byte halves. Caller manages IV/tweak safety.
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
        && u128::from_be_bytes(s.initial_block)
            .checked_add(data.len().div_ceil(16) as u128 - 1)
            .is_none()
    {
        return Err(limit("AES counter would wrap"));
    }
    if matches!(s.mode, Mode::Stealing(_) | Mode::Storage(_)) && data.len() < 16 {
        return Err(failed());
    }
    if let Mode::Storage(kind) = s.mode
        && kind.full_blocks()
        && !data.len().is_multiple_of(16)
    {
        return Err(failed());
    }
    backend::transform(data, s, inverse)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use an exact AES mode with its required key, IV/tweak and padding; storage key halves must differ",
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

fn storage128_key_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    hex_schema("^[0-9A-Fa-f]{64}$")
}
fn storage256_key_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    hex_schema("^[0-9A-Fa-f]{128}$")
}
fn common_key_schema() -> schemars::Schema {
    hex_schema("^(?:[0-9A-Fa-f]{32}|[0-9A-Fa-f]{48}|[0-9A-Fa-f]{64}|[0-9A-Fa-f]{128})$")
}
