//! Exact SM2/SM3 encryption with explicit ciphertext representation.
mod catalog;
#[cfg(test)]
mod tests;
use super::entropy::SystemRng;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use rand_core::TryCryptoRng;
use rsa::pkcs8::der::{Decode, Encode};
use schemars::JsonSchema;
use serde::Deserialize;
use sm2::{
    elliptic_curve::{sec1::ToSec1Point, subtle::ConstantTimeEq},
    pke::{Cipher, EncryptingKey, Mode},
};
use sm3::{Digest, Sm3};
use zeroize::{Zeroize, Zeroizing};
pub const ENCRYPT: &str = "cipher.sm2.encrypt";
pub const DECRYPT: &str = "cipher.sm2.decrypt";
pub const MAX_BODY: usize = 16 * 1024 * 1024;
pub const MAX_CIPHERTEXT: usize = MAX_BODY + 128;
pub const MAX_KEY: usize = 65;
pub const MAX_PARAMETERS: usize = 256;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "sm2_sm3_c1c3c2")]
    C1C3C2,
    #[serde(rename = "sm2_sm3_c1c2c3")]
    C1C2C3,
    #[serde(rename = "sm2_sm3_der")]
    Der,
}
impl Profile {
    pub const NAMES: &[&str] = &["sm2_sm3_c1c3c2", "sm2_sm3_c1c2c3", "sm2_sm3_der"];
    fn mode(self) -> Mode {
        match self {
            Self::C1C2C3 => Mode::C1C2C3,
            _ => Mode::C1C3C2,
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
}
fn parameters(document: &str, key: &[u8]) -> Result<Profile, OperationError> {
    if document.len() > MAX_PARAMETERS || key.len() > MAX_KEY {
        return Err(limit());
    }
    let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
    Ok(p.profile)
}
/// Validated uncompressed SEC1 public key on the standard SM2 curve.
pub struct EncryptionSettings {
    profile: Profile,
    key: EncryptingKey,
}
/// Validated32-byte private scalar; redacted diagnostics and backend erasure.
pub struct DecryptionSettings {
    profile: Profile,
    key: sm2::SecretKey,
}
macro_rules! redact {($($t:ty),*)=>{$(impl std::fmt::Debug for $t {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Sm2Settings { [REDACTED] }")}})*};}
redact!(EncryptionSettings, DecryptionSettings);
impl EncryptionSettings {
    /// Require one explicit SM2/SM3 encoding profile and exactly65 uncompressed
    /// SEC1 public bytes. Infinity, noncanonical coordinates and other curves fail.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let profile = parameters(document, key)?;
        if key.len() != 65 || key[0] != 4 {
            return Err(invalid());
        }
        let public = sm2::PublicKey::from_sec1_bytes(key).map_err(|_| invalid())?;
        Ok(Self {
            profile,
            key: EncryptingKey::new_with_mode(public, profile.mode()),
        })
    }
}
impl DecryptionSettings {
    /// Require the selected representation and an exact32-byte big-endian scalar
    /// in1..n-1 on SM2. No key generation or format guessing occurs.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let profile = parameters(document, key)?;
        if key.len() != 32 {
            return Err(invalid());
        }
        Ok(Self {
            profile,
            key: sm2::SecretKey::from_slice(key).map_err(|_| invalid())?,
        })
    }
}
/// Encrypt nonempty bytes using SM2, SM3, the standard counter KDF and fresh
/// bounded OS/browser entropy. Raw C1 is always65 uncompressed SEC1 bytes.
/// No caller-controlled production coins or sender authentication is provided.
pub fn encrypt(input: &[u8], s: &EncryptionSettings) -> Result<Vec<u8>, OperationError> {
    encrypt_with_rng(input, s, &mut SystemRng::bounded(256))
}
fn encrypt_with_rng(
    input: &[u8],
    s: &EncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_BODY {
        return Err(limit());
    }
    if input.is_empty() {
        return Err(failed());
    }
    match s.profile {
        Profile::Der => s.key.encrypt_der(rng, input),
        _ => s.key.encrypt(rng, input),
    }
    .map_err(|_| failed())
}
fn parse_cipher(input: &[u8], profile: Profile) -> Result<Cipher<'_>, OperationError> {
    if input.len() > MAX_CIPHERTEXT {
        return Err(limit());
    }
    let cipher = match profile {
        Profile::Der => {
            let c = Cipher::from_der(input).map_err(|_| failed())?;
            if c.to_der().map_err(|_| failed())? != input {
                return Err(failed());
            }
            c
        }
        _ => {
            if input.len() < 98 || input[0] != 4 {
                return Err(failed());
            }
            Cipher::from_slice(input, profile.mode()).map_err(|_| failed())?
        }
    };
    if cipher.c2().is_empty() || cipher.c2().len() > MAX_BODY {
        return Err(failed());
    }
    sm2::PublicKey::from_affine(*cipher.c1()).map_err(|_| failed())?;
    Ok(cipher)
}
/// Parse exactly the selected ciphertext representation and authenticate the
/// entire recovered message before returning any bytes. SM3 comparison is
/// constant-time and an all-zero KDF is rejected. No sender identity is proven.
pub fn decrypt(input: &[u8], s: &DecryptionSettings) -> Result<Vec<u8>, OperationError> {
    let cipher = parse_cipher(input, s.profile)?;
    let shared = Zeroizing::new(
        (sm2::ProjectivePoint::from(*cipher.c1()) * s.key.to_nonzero_scalar().as_ref()).to_affine(),
    );
    let point = Zeroizing::new(shared.to_sec1_point(false));
    let coordinates = point.as_bytes();
    let x = &coordinates[1..33];
    let y = &coordinates[33..65];
    let mut plaintext = Zeroizing::new(vec![0; cipher.c2().len()]);
    let nonzero = kdf_xor(cipher.c2(), &mut plaintext, |counter| {
        let mut hash = Sm3::new();
        hash.update(x);
        hash.update(y);
        hash.update(counter.to_be_bytes());
        hash.finalize().into()
    });
    let mut hash = Sm3::new();
    hash.update(x);
    hash.update(&*plaintext);
    hash.update(y);
    let mut digest = hash.finalize();
    let authenticated = bool::from(digest.as_slice().ct_eq(cipher.c3().as_slice()));
    digest.as_mut_slice().zeroize();
    if !nonzero || !authenticated {
        return Err(failed());
    }
    Ok(std::mem::take(&mut *plaintext))
}
// Compose the specified SM3 counter KDF over maintained primitives. This narrow
// Core composition avoids upstream0.14's ordinary tag comparison and missing
// all-zero KDF decryption check; SDKs never implement the scheme themselves.
fn kdf_xor(input: &[u8], output: &mut [u8], mut block: impl FnMut(u32) -> [u8; 32]) -> bool {
    let mut nonzero = 0;
    for (index, (source, dest)) in input.chunks(32).zip(output.chunks_mut(32)).enumerate() {
        let key = Zeroizing::new(block(index as u32 + 1));
        for (i, byte) in source.iter().enumerate() {
            nonzero |= key[i];
            dest[i] = byte ^ key[i];
        }
    }
    nonzero != 0
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid SM2 profile or exact raw key encoding",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "SM2 operation failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "SM2 input exceeds its declared bounds",
    )
}
