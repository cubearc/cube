//! Explicit raw-block reference profiles for legacy interoperability.
//!
//! These non-AES backends are unaudited and do not promise constant-time
//! execution. They are not a message-encryption scheme. No mode, padding,
//! authentication, key derivation or random generation is performed.
mod anubis;
mod backend;
mod clefia;
mod crypton;
mod e2;
mod finite_field;
mod khazad;
mod mars;
mod safer;
mod seed;
mod simon;
mod skinny;
mod square;

mod catalog;
mod hight;
mod kasumi;
mod lea;
mod misty1;
mod noekeon;
mod present;
mod shacal2;
mod tea;
mod twine;

mod profiles;
mod rijndael;
mod skipjack;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub use profiles::Profile;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_PARAMETERS: usize = 4096;
pub const ENCRYPT: &str = "cipher.block.encrypt_blocks";
pub const DECRYPT: &str = "cipher.block.decrypt_blocks";

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{2}){1,128}$"))]
    key_hex: String,
    #[serde(default, deserialize_with = "effective_bits")]
    #[schemars(range(min = 1, max = 1024))]
    effective_key_bits: Option<u16>,
}
fn effective_bits<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u16>, D::Error> {
    u16::deserialize(d).map(Some)
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}

/// Validated owned key and exact profile. Debug never reveals key material.
/// Owned key bytes are wiped on drop; borrowed, parser, interpreter and OS
/// copies are outside this guarantee. Backend temporary schedules can persist.
pub struct Settings {
    profile: Profile,
    key: Zeroizing<Vec<u8>>,
    effective_key_bits: Option<u16>,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("profile", &self.profile)
            .field("key", &"[REDACTED]")
            .finish()
    }
}
impl Settings {
    /// Validate an exact key and RC2 effective-bit count before copying the key.
    /// RC2 requires 1..=1024 effective bits; other profiles reject that setting.
    pub fn new(
        profile: Profile,
        key: &[u8],
        effective_key_bits: Option<u16>,
    ) -> Result<Self, OperationError> {
        if !profile.accepts_key(key.len()) {
            return Err(invalid(
                "key_hex",
                "key length does not match the explicit block profile",
            ));
        }
        let valid_bits = if profile == Profile::Rc2 {
            matches!(effective_key_bits, Some(1..=1024))
        } else {
            effective_key_bits.is_none()
        };
        if !valid_bits {
            return Err(invalid(
                "effective_key_bits",
                "RC2 requires 1..1024 effective bits; other profiles forbid this field",
            ));
        }
        Ok(Self {
            profile,
            key: Zeroizing::new(key.to_vec()),
            effective_key_bits,
        })
    }
    /// Parse bounded private JSON, rejecting duplicates, unknown fields and null
    /// effective-bit settings. Error messages never include parser excerpts.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "block parameters exceed 4096 bytes",
            ));
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| {
            invalid(
                "parameters",
                "use the exact profile, hexadecimal key and profile-specific settings",
            )
        })?;
        if !p.key_hex.len().is_multiple_of(2) || p.key_hex.len() > 256 {
            return Err(invalid(
                "key_hex",
                "key must contain exact hexadecimal byte pairs",
            ));
        }
        let mut key = Zeroizing::new(vec![0; p.key_hex.len() / 2]);
        hex::decode_to_slice(&p.key_hex, &mut key[..])
            .map_err(|_| invalid("key_hex", "key must contain exact hexadecimal byte pairs"))?;
        Self::new(p.profile, &key, p.effective_key_bits)
    }
}
fn invalid(parameter: &'static str, message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).parameter(parameter)
}
/// Encrypt exact independent whole blocks. Empty input still requires valid
/// settings. Equal input blocks produce equal output blocks. At most 1 MiB of
/// input (64 KiB for educational reference implementations) and one equally
/// sized output allocation are accepted.
pub fn encrypt(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, false)
}
/// Decrypt exact independent blocks. A wrong valid key still produces bytes;
/// success does not authenticate the key, plaintext or message integrity.
pub fn decrypt(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, true)
}
fn transform(
    data: &[u8],
    settings: &Settings,
    decrypting: bool,
) -> Result<Vec<u8>, OperationError> {
    let operation = if decrypting { DECRYPT } else { ENCRYPT };
    if data.len() > settings.profile.max_input_bytes() {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "block input exceeds the selected profile byte limit",
        )
        .operation(operation));
    }
    if !data.len().is_multiple_of(settings.profile.block_bytes()) {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "input must contain whole blocks; padding is not supported",
        )
        .operation(operation));
    }
    Ok(backend::transform(data, settings, decrypting))
}
