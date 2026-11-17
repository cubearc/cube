//! Deterministic RFC3394/5649 wrapping with integrity-before-key-release.
mod catalog;
use crate::errors::{ErrorCode, OperationError};
use aes::{
    Aes128, Aes192, Aes256,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};
pub const MAX_INPUT: usize = 65536;
pub const MAX_WRAPPED: usize = MAX_INPUT + 8;
pub const MAX_PARAMETERS: usize = 1024;
pub const WRAP: &str = "cipher.aes.key_wrap.wrap";
pub const UNWRAP: &str = "cipher.aes.key_wrap.unwrap";
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Profile {
    #[serde(rename = "aes_kw_rfc3394")]
    Kw,
    #[serde(rename = "aes_kwp_rfc5649")]
    Kwp,
}
impl Profile {
    pub const NAMES: &[&str] = &["aes_kw_rfc3394", "aes_kwp_rfc5649"];
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{32}|[0-9A-Fa-f]{48}|[0-9A-Fa-f]{64})$"))]
    key_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}
/// Validated KEK, wiped on drop and redacted in Debug.
pub struct Settings {
    profile: Profile,
    key: Zeroizing<Vec<u8>>,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyWrapSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Require the exact KW/KWP profile and a16/24/32-byte hexadecimal KEK.
    /// JSON is limited to1024 bytes; errors never include its secret contents.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        if !matches!(p.key_hex.len(), 32 | 48 | 64) {
            return Err(invalid());
        }
        let mut key = Zeroizing::new(vec![0; p.key_hex.len() / 2]);
        hex::decode_to_slice(&p.key_hex, &mut key[..]).map_err(|_| invalid())?;
        Ok(Self {
            profile: p.profile,
            key,
        })
    }
}
/// Wrap16..65536 whole eight-byte units under KW, or1..65536 bytes under KWP.
/// Fixed RFC IV/AIV, padding and output bounds are computed before allocation.
/// No external IV, random generation, KDF or envelope is accepted.
pub fn wrap(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, false)
}
/// Verify the complete IV/AIV/length/padding integrity checks before returning
/// any key data. Wrong KEK or malformed wrapping fails uniformly without output.
pub fn unwrap(data: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(data, settings, true)
}
fn transform(data: &[u8], s: &Settings, inverse: bool) -> Result<Vec<u8>, OperationError> {
    if data.len() > if inverse { MAX_WRAPPED } else { MAX_INPUT } {
        return Err(limit());
    }
    if inverse {
        let minimum = if s.profile == Profile::Kw { 24 } else { 16 };
        if data.len() < minimum || !data.len().is_multiple_of(8) {
            return Err(failed());
        }
    } else if data.is_empty()
        || (s.profile == Profile::Kw && (data.len() < 16 || !data.len().is_multiple_of(8)))
    {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "invalid key data length for the selected wrap profile",
        ));
    }
    let length = if inverse {
        data.len() - 8
    } else {
        data.len().div_ceil(8) * 8 + 8
    };
    let mut buffer = Zeroizing::new(vec![0; length]);
    let size = match s.key.len() {
        16 => apply::<Aes128>(data, &mut buffer, s, inverse),
        24 => apply::<Aes192>(data, &mut buffer, s, inverse),
        _ => apply::<Aes256>(data, &mut buffer, s, inverse),
    }?;
    Ok(buffer[..size].to_vec())
}
fn apply<C>(
    data: &[u8],
    buffer: &mut [u8],
    s: &Settings,
    inverse: bool,
) -> Result<usize, OperationError>
where
    C: BlockCipherEncrypt<BlockSize = aes::cipher::consts::U16> + BlockCipherDecrypt + KeyInit,
{
    let result = match (s.profile, inverse) {
        (Profile::Kw, false) => aes_kw::AesKw::<C>::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .wrap_key(data, buffer),
        (Profile::Kw, true) => aes_kw::AesKw::<C>::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .unwrap_key(data, buffer),
        (Profile::Kwp, false) => aes_kw::AesKwp::<C>::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .wrap_key(data, buffer),
        (Profile::Kwp, true) => aes_kw::AesKwp::<C>::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .unwrap_key(data, buffer),
    };
    result.map(|output| output.len()).map_err(|_| failed())
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use an exact AES key-wrap profile and16/24/32-byte hexadecimal key_hex",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "key unwrap integrity check failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "key-wrap input or private settings exceed their declared bounds",
    )
}
