//! Detached authentication for the designers' SKEW2011 Grain128a revision.
//! Core owns both stream and MAC; failed verification releases no plaintext.
mod catalog;
use super::{MAX_INPUT, MAX_PARAMETERS, bits, decode_hex, grain128a};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};
pub const MAX_TAG_BYTES: usize = 4;
pub const ENCRYPT: &str = "cipher.stream.auth.encrypt";
pub const DECRYPT: &str = "cipher.stream.auth.decrypt";
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
enum Profile {
    #[serde(rename = "grain128a_skew2011_msb_v1")]
    Grain128a,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{2}){16}$"))]
    key_hex: String,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{2}){12}$"))]
    nonce_hex: String,
    #[schemars(range(min = 0, max = 524288))]
    data_bits: usize,
    #[schemars(range(min = 1, max = 32))]
    tag_bits: u8,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
        self.nonce_hex.zeroize();
    }
}
/// Validated private settings. Exact packed bit lengths are mandatory.
pub struct Settings {
    profile: Profile,
    key: Zeroizing<Vec<u8>>,
    nonce: Zeroizing<Vec<u8>>,
    data_bits: usize,
    tag_bits: u8,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("profile", &self.profile)
            .field("key", &"[REDACTED]")
            .field("nonce", &"[REDACTED]")
            .finish()
    }
}
fn invalid(parameter: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact Grain128a profile, key, nonce, data_bits and tag_bits",
    )
    .parameter(parameter)
}
impl Settings {
    /// Parse at most4096 UTF8 bytes of private JSON. No defaults, duplicate/null/
    /// unknown fields, implicit padding or automatic nonce/key generation.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "stream parameters exceed 4096 bytes",
            ));
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid("parameters"))?;
        let key = decode_hex(&p.key_hex, "key_hex")?;
        let nonce = decode_hex(&p.nonce_hex, "nonce_hex")?;
        if key.len() != 16 {
            return Err(invalid("key_hex"));
        }
        if nonce.len() != 12 {
            return Err(invalid("nonce_hex"));
        }
        if p.data_bits > MAX_INPUT * 8 {
            return Err(invalid("data_bits"));
        }
        if !(1..=32).contains(&p.tag_bits) {
            return Err(invalid("tag_bits"));
        }
        Ok(Self {
            profile: p.profile,
            key,
            nonce,
            data_bits: p.data_bits,
            tag_bits: p.tag_bits,
        })
    }
    fn validate(&self, input: &[u8]) -> Result<(), OperationError> {
        if input.len() > MAX_INPUT {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "stream input exceeds 64 KiB",
            ));
        }
        if !bits::packed(input, self.data_bits, bits::Packing::Msb) {
            return Err(OperationError::new(
                ErrorCode::InvalidInput,
                "packed input must match data_bits with zero unused bits",
            ));
        }
        Ok(())
    }
}
/// Exact packed ciphertext and detached packed tag. Both start at the MSB;
/// unused tail bits are zero. No encrypted padding or associated data is added.
pub struct Encrypted {
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}
fn tag(full: &[u8; 4], width: u8) -> Vec<u8> {
    let packed = Zeroizing::new((u32::from_be_bytes(*full) << (32 - width)).to_be_bytes());
    packed[..usize::from(width).div_ceil(8)].to_vec()
}
/// Encrypt at most64KiB using the original optional MAC. tag_bits is1..32;
/// this historical construction is unaudited and not a modern AEAD profile.
pub fn encrypt(input: &[u8], settings: &Settings) -> Result<Encrypted, OperationError> {
    settings.validate(input)?;
    let mut output = Zeroizing::new(input.to_vec());
    let full = grain128a::apply(
        &mut output,
        &settings.key,
        &settings.nonce,
        settings.data_bits,
        false,
    );
    Ok(Encrypted {
        ciphertext: std::mem::take(&mut *output),
        tag: tag(&full, settings.tag_bits),
    })
}
fn authentication_failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "authentication failed")
}
/// Verify the complete detached tag before returning any plaintext. Failed
/// verification wipes the owned tentative plaintext and returns a static error.
pub fn decrypt(
    input: &[u8],
    supplied_tag: &[u8],
    settings: &Settings,
) -> Result<Vec<u8>, OperationError> {
    settings.validate(input)?;
    if !bits::packed(
        supplied_tag,
        usize::from(settings.tag_bits),
        bits::Packing::Msb,
    ) {
        return Err(authentication_failed());
    }
    let mut output = Zeroizing::new(input.to_vec());
    let full = grain128a::apply(
        &mut output,
        &settings.key,
        &settings.nonce,
        settings.data_bits,
        true,
    );
    let expected = Zeroizing::new(tag(&full, settings.tag_bits));
    let difference = expected
        .iter()
        .zip(supplied_tag)
        .fold(0u8, |d, (a, b)| d | (a ^ b));
    if difference != 0 {
        return Err(authentication_failed());
    }
    Ok(std::mem::take(&mut *output))
}
