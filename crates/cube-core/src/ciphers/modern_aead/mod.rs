//! Explicit authenticated byte profiles, detached tags and whole-message release.
mod backend;
mod catalog;
mod profiles;
mod siv;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub use profiles::Profile;
use schemars::JsonSchema;
use serde::Deserialize;
pub use siv::{SivProfile, SivSettings, siv_decrypt, siv_encrypt};
use zeroize::{Zeroize, Zeroizing};
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub const MAX_PARAMETERS: usize = 2048;
pub const MAX_AAD_COMPONENTS: usize = 126;
pub const ENCRYPT: &str = "cipher.aead.encrypt";
pub const DECRYPT: &str = "cipher.aead.decrypt";
pub const SIV_ENCRYPT: &str = "cipher.aes.siv.encrypt";
pub const SIV_DECRYPT: &str = "cipher.aes.siv.decrypt";
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    key_hex: String,
    nonce_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}
/// Owned validated key material, erased on drop and redacted in Debug.
pub struct Settings {
    profile: Profile,
    key: Zeroizing<Vec<u8>>,
    nonce: Vec<u8>,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AeadSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Require an exact profile, hexadecimal key and nonce. No implicit RNG,
    /// key generation, nonce default, AAD conversion or secret error excerpt.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        document_bound(document)?;
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        let spec = p.profile.spec();
        Ok(Self {
            profile: p.profile,
            key: decode(&p.key_hex, spec.key)?,
            nonce: decode(&p.nonce_hex, spec.nonce)?.to_vec(),
        })
    }
}
/// Ciphertext and its exact detached tag. No serialization or envelope is implied.
pub struct Detached {
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}
/// Encrypt binary plaintext with exact associated data and caller-managed nonce.
/// Combined plaintext/AAD is at most16MiB; CCM message bounds may be smaller.
/// All profiles require explicit settings and a full profile-specific tag.
pub fn encrypt(data: &[u8], aad: &[u8], s: &Settings) -> Result<Detached, OperationError> {
    preflight(data, aad.len(), s.profile.spec().body_limit)?;
    let mut buffer = Zeroizing::new(data.to_vec());
    let tag = (s.profile.spec().run)(s, aad, &mut buffer, None)?;
    Ok(Detached {
        ciphertext: buffer.to_vec(),
        tag,
    })
}
/// Authenticate the complete ciphertext, AAD and detached tag before returning
/// any plaintext. Failed tentative plaintext is wiped; failures disclose no key.
pub fn decrypt(
    data: &[u8],
    aad: &[u8],
    tag: &[u8],
    s: &Settings,
) -> Result<Vec<u8>, OperationError> {
    preflight(data, aad.len(), s.profile.spec().body_limit)?;
    if tag.len() != s.profile.spec().tag {
        return Err(authentication_failed());
    }
    let mut buffer = Zeroizing::new(data.to_vec());
    (s.profile.spec().run)(s, aad, &mut buffer, Some(tag))?;
    Ok(buffer.to_vec())
}
fn preflight(data: &[u8], aad: usize, body_limit: usize) -> Result<(), OperationError> {
    if data.len() > body_limit
        || aad > MAX_INPUT
        || data.len().checked_add(aad).is_none_or(|n| n > MAX_INPUT)
    {
        return Err(limit(
            "authenticated data exceeds its body or combined 16 MiB bound",
        ));
    }
    Ok(())
}
fn document_bound(document: &str) -> Result<(), OperationError> {
    if document.len() > MAX_PARAMETERS {
        Err(limit("authenticated settings exceed 2048 bytes"))
    } else {
        Ok(())
    }
}
fn decode(text: &str, size: usize) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    if text.len() != size * 2 {
        return Err(invalid());
    }
    let mut bytes = Zeroizing::new(vec![0; size]);
    hex::decode_to_slice(text, &mut bytes[..]).map_err(|_| invalid())?;
    Ok(bytes)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact authenticated profile and its required hexadecimal key/nonce fields",
    )
}
fn authentication_failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "authentication failed")
}
fn limit(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}
