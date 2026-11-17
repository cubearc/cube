//! X25519 and P-256 key agreement; no encryption or implicit KDF.
mod catalog;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use p256::elliptic_curve::sec1::ToSec1Point;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const AGREE: &str = "crypto.ecdh.agree";
pub const MAX_PARAMETERS: usize = 1024;
pub const MAX_PUBLIC_KEY: usize = 65;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum Profile {
    #[serde(rename = "x25519_rfc7748")]
    X25519,
    #[serde(rename = "p256_sec1_raw")]
    P256,
}
impl Profile {
    pub const NAMES: &[&str] = &["x25519_rfc7748", "p256_sec1_raw"];
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(regex(pattern = "^[0-9A-Fa-f]{64}$"))]
    private_key_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.private_key_hex.zeroize();
    }
}
enum Key {
    X25519(x25519_dalek::StaticSecret),
    P256(p256::SecretKey),
}
/// Validated curve-specific private key, redacted in diagnostics.
pub struct Settings {
    key: Key,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EcdhSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Parse an explicit curve profile and exactly32 private bytes in hexadecimal.
    /// X25519 applies RFC7748 clamping; P-256 requires a scalar in1..n-1.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        if p.private_key_hex.len() != 64 {
            return Err(invalid());
        }
        let mut secret = Zeroizing::new([0u8; 32]);
        hex::decode_to_slice(&p.private_key_hex, &mut secret[..]).map_err(|_| invalid())?;
        Self::from_private_key(p.profile, &secret[..])
    }
    /// Validate a typed profile and exactly32 raw private bytes without a JSON
    /// serialization round trip. This does not generate keys or apply a KDF.
    pub fn from_private_key(profile: Profile, key: &[u8]) -> Result<Self, OperationError> {
        let secret = Zeroizing::new(<[u8; 32]>::try_from(key).map_err(|_| invalid())?);
        let key = match profile {
            Profile::X25519 => Key::X25519(x25519_dalek::StaticSecret::from(*secret)),
            Profile::P256 => {
                Key::P256(p256::SecretKey::from_slice(&secret[..]).map_err(|_| invalid())?)
            }
        };
        Ok(Self { key })
    }
    /// Return the matching public key: raw32-byte X25519 or compressed SEC1 P-256.
    /// This helper performs no generation and does not return private material.
    pub fn public_key(&self) -> Vec<u8> {
        self.public_encoding(true)
    }
    /// Return65-byte uncompressed SEC1 P-256, or unchanged raw32-byte X25519.
    pub fn public_key_uncompressed(&self) -> Vec<u8> {
        self.public_encoding(false)
    }
    fn public_encoding(&self, compressed: bool) -> Vec<u8> {
        match &self.key {
            Key::X25519(s) => x25519_dalek::PublicKey::from(s).as_bytes().to_vec(),
            Key::P256(s) => s.public_key().to_sec1_point(compressed).as_bytes().to_vec(),
        }
    }
}
/// Compute one32-byte raw shared secret using the explicit private curve profile.
/// X25519 accepts RFC7748's masked/reduced input encodings but rejects all-zero
/// output. P-256 accepts only canonical compressed/uncompressed SEC1 points.
/// The result is neither a session key nor authenticated; apply an explicit KDF
/// binding both participants and protocol context before using it as key material.
pub fn agree(peer: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    if peer.len() > MAX_PUBLIC_KEY {
        return Err(limit());
    }
    match &settings.key {
        Key::X25519(s) => {
            let peer: [u8; 32] = peer.try_into().map_err(|_| failed())?;
            let shared = s.diffie_hellman(&x25519_dalek::PublicKey::from(peer));
            if !shared.was_contributory() {
                return Err(failed());
            }
            Ok(shared.as_bytes().to_vec())
        }
        Key::P256(s) => {
            if !matches!(
                (peer.len(), peer.first()),
                (33, Some(2 | 3)) | (65, Some(4))
            ) {
                return Err(failed());
            }
            let public = p256::PublicKey::from_sec1_bytes(peer).map_err(|_| failed())?;
            let shared = p256::ecdh::diffie_hellman(s.to_nonzero_scalar(), public.as_affine());
            Ok(shared.raw_secret_bytes().to_vec())
        }
    }
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use an exact ECDH profile and valid32-byte private_key_hex",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "key agreement failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "ECDH public key or private settings exceed their declared bounds",
    )
}
