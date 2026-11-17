//! Exact raw ElGamal and Cramer-Shoup encryption in the RFC7919 prime-order group.
mod catalog;
mod constants;
mod field;
#[cfg(test)]
mod tests;
use super::entropy::SystemRng;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use crypto_bigint::{BoxedUint, CtEq, modular::BoxedMontyForm};
use field::{BITS, WIDTH, encode, encode_tuple, group};
use rand_core::TryCryptoRng;
use schemars::JsonSchema;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;
pub const ELGAMAL_ENCRYPT: &str = "cipher.elgamal.encrypt";
pub const ELGAMAL_DECRYPT: &str = "cipher.elgamal.decrypt";
pub const CRAMER_SHOUP_ENCRYPT: &str = "cipher.cramer_shoup.encrypt";
pub const CRAMER_SHOUP_DECRYPT: &str = "cipher.cramer_shoup.decrypt";
pub const MAX_PARAMETERS: usize = 1024;
pub const MAX_KEY: usize = 1280;
pub const MAX_CIPHERTEXT: usize = 1024;
pub const MAX_LABEL: usize = 256;
#[derive(Deserialize, JsonSchema)]
pub enum ElGamalProfile {
    #[serde(rename = "elgamal_ffdhe2048_raw_v1")]
    Raw,
}
#[derive(Deserialize, JsonSchema)]
pub enum CramerShoupProfile {
    #[serde(rename = "cramer_shoup_ffdhe2048_sha256_raw_v1")]
    Raw,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ElGamalParameters {
    profile: ElGamalProfile,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CramerShoupParameters {
    profile: CramerShoupProfile,
    #[serde(default)]
    #[schemars(length(max = 512), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    label_hex: String,
}
fn parameters<T: serde::de::DeserializeOwned>(
    document: &str,
    key: &[u8],
) -> Result<T, OperationError> {
    if document.len() > MAX_PARAMETERS || key.len() > MAX_KEY {
        return Err(limit());
    }
    serde_json::from_str(document).map_err(|_| invalid())
}
fn label(p: &CramerShoupParameters) -> Result<Vec<u8>, OperationError> {
    if p.label_hex.len() > MAX_LABEL * 2 {
        return Err(limit());
    }
    hex::decode(&p.label_hex).map_err(|_| invalid())
}
/// One validated nonidentity prime-order ElGamal recipient public value.
pub struct ElGamalEncryptionSettings {
    key: BoxedMontyForm,
}
/// Exact256-byte ElGamal private scalar in1..q-1.
pub struct ElGamalDecryptionSettings {
    key: Zeroizing<BoxedUint>,
}
/// Three validated nonidentity recipient group values c,d,h and explicit label.
pub struct CramerShoupEncryptionSettings {
    key: [BoxedMontyForm; 3],
    label: Vec<u8>,
}
/// Five private scalars and their checked public counterpart; diagnostics redacted.
pub struct CramerShoupDecryptionSettings {
    key: [Zeroizing<BoxedUint>; 5],
    public: [BoxedMontyForm; 3],
    label: Vec<u8>,
}
macro_rules! redact {($($t:ty),*)=>{$(impl std::fmt::Debug for $t{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("GroupEncryptionSettings { [REDACTED] }")}})*};}
redact!(
    ElGamalEncryptionSettings,
    ElGamalDecryptionSettings,
    CramerShoupEncryptionSettings,
    CramerShoupDecryptionSettings
);
impl ElGamalEncryptionSettings {
    /// Require the exact raw profile and a256-byte nonidentity subgroup element.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: ElGamalParameters = parameters(document, key)?;
        let _ = p.profile;
        Ok(Self {
            key: group().element(key, false).map_err(|_| invalid())?,
        })
    }
}
impl ElGamalDecryptionSettings {
    /// Require the exact profile and a256-byte big-endian scalar in1..q-1.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: ElGamalParameters = parameters(document, key)?;
        let _ = p.profile;
        Ok(Self {
            key: group().scalar(key)?,
        })
    }
    /// Return the corresponding256-byte public value, without generating a key.
    pub fn public_key(&self) -> Vec<u8> {
        encode(&group().g1.pow(&self.key))
    }
}
impl CramerShoupEncryptionSettings {
    /// Parse exactly c||d||h as three256-byte nonidentity subgroup elements.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: CramerShoupParameters = parameters(document, key)?;
        let _ = p.profile;
        if key.len() != 3 * WIDTH {
            return Err(invalid());
        }
        let values = key
            .as_chunks::<WIDTH>()
            .0
            .iter()
            .map(|v| group().element(v, false).map_err(|_| invalid()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            key: values.try_into().map_err(|_| invalid())?,
            label: label(&p)?,
        })
    }
}
impl CramerShoupDecryptionSettings {
    /// Parse x1||x2||y1||y2||z as five256-byte scalars in1..q-1, refusing
    /// a degenerate public counterpart. No key generation or custom group occurs.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let p: CramerShoupParameters = parameters(document, key)?;
        let _ = p.profile;
        if key.len() != 5 * WIDTH {
            return Err(invalid());
        }
        let values = key
            .as_chunks::<WIDTH>()
            .0
            .iter()
            .map(|v| group().scalar(v))
            .collect::<Result<Vec<_>, _>>()?;
        let key: [Zeroizing<BoxedUint>; 5] = values.try_into().map_err(|_| invalid())?;
        let g = group();
        let public = [
            &g.g1.pow(&key[0]) * &g.g2.pow(&key[1]),
            &g.g1.pow(&key[2]) * &g.g2.pow(&key[3]),
            g.g1.pow(&key[4]),
        ];
        let one = BoxedMontyForm::one(public[0].params());
        if public.iter().any(|p| p == &one) {
            return Err(invalid());
        }
        Ok(Self {
            key,
            public,
            label: label(&p)?,
        })
    }
    /// Return c||d||h as three fixed-width public values; no private bytes.
    pub fn public_key(&self) -> Vec<u8> {
        encode_tuple(&self.public)
    }
}
fn input(input: &[u8], width: usize) -> Result<(), OperationError> {
    if input.len() > MAX_CIPHERTEXT {
        return Err(limit());
    }
    if input.len() != width {
        return Err(failed());
    }
    Ok(())
}
/// Encrypt one exact256-byte subgroup message with fresh uniform r in1..q-1.
/// Output is u||e (512 bytes). Raw ElGamal is malleable and unauthenticated.
pub fn elgamal_encrypt(
    message: &[u8],
    s: &ElGamalEncryptionSettings,
) -> Result<Vec<u8>, OperationError> {
    elgamal_with_rng(message, s, &mut SystemRng::bounded(2048))
}
fn elgamal_with_rng(
    message: &[u8],
    s: &ElGamalEncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    input(message, WIDTH)?;
    let g = group();
    let message = Zeroizing::new(g.element(message, true)?);
    let r = g.random_scalar(rng)?;
    let u = g.g1.pow(&r);
    let mask = Zeroizing::new(s.key.pow(&r));
    let e = &*mask * &*message;
    Ok(encode_tuple(&[u, e]))
}
/// Validate a raw512-byte pair and return one fixed-width subgroup message.
/// No padding, authenticity or sender identity is inferred from successful output.
pub fn elgamal_decrypt(
    ciphertext: &[u8],
    s: &ElGamalDecryptionSettings,
) -> Result<Vec<u8>, OperationError> {
    input(ciphertext, 2 * WIDTH)?;
    let g = group();
    let u = g.element(&ciphertext[..WIDTH], false)?;
    let e = g.element(&ciphertext[WIDTH..], true)?;
    let inverse = g.inverse_exponent(&s.key);
    let mask = Zeroizing::new(u.pow(&inverse));
    Ok(encode(&Zeroizing::new(&e * &*mask)))
}
fn alpha(prefix: &[u8], label: &[u8]) -> BoxedUint {
    let mut hash = Sha256::new();
    hash.update(b"CubeArc Cramer-Shoup ffdhe2048 SHA256 v1\0");
    hash.update(prefix);
    hash.update(label);
    BoxedUint::from_be_slice(hash.finalize().as_slice(), BITS)
        .expect("public SHA256 fits scalar precision")
}
/// Encrypt one subgroup message as u1||u2||e||v, using the profile's pinned
/// second generator and fixed-width domain-bound SHA256 transcript and label.
pub fn cramer_shoup_encrypt(
    message: &[u8],
    s: &CramerShoupEncryptionSettings,
) -> Result<Vec<u8>, OperationError> {
    cramer_shoup_with_rng(message, s, &mut SystemRng::bounded(2048))
}
fn cramer_shoup_with_rng(
    message: &[u8],
    s: &CramerShoupEncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    input(message, WIDTH)?;
    let g = group();
    let message = Zeroizing::new(g.element(message, true)?);
    let r = g.random_scalar(rng)?;
    let u1 = g.g1.pow(&r);
    let u2 = g.g2.pow(&r);
    let mask = Zeroizing::new(s.key[2].pow(&r));
    let e = &*mask * &*message;
    let mut output = encode_tuple(&[u1, u2, e]);
    let a = alpha(&output, &s.label);
    let ra = g.product(&r, &a);
    let tag = &s.key[0].pow(&r) * &s.key[1].pow(&ra);
    output.extend_from_slice(&encode(&tag));
    Ok(output)
}
/// Check all four encoded subgroup values and the complete private verification
/// equation before releasing one raw group message. Invalid label or ciphertext
/// returns a uniform error with no partial output. No sender identity is proven.
pub fn cramer_shoup_decrypt(
    ciphertext: &[u8],
    s: &CramerShoupDecryptionSettings,
) -> Result<Vec<u8>, OperationError> {
    input(ciphertext, 4 * WIDTH)?;
    let g = group();
    let u1 = g.element(&ciphertext[..WIDTH], false)?;
    let u2 = g.element(&ciphertext[WIDTH..2 * WIDTH], false)?;
    let e = g.element(&ciphertext[2 * WIDTH..3 * WIDTH], true)?;
    let tag = g.element(&ciphertext[3 * WIDTH..], true)?;
    let a = alpha(&ciphertext[..3 * WIDTH], &s.label);
    let first = g.linear(&s.key[0], &s.key[2], &a);
    let second = g.linear(&s.key[1], &s.key[3], &a);
    let expected = Zeroizing::new(&u1.pow(&first) * &u2.pow(&second));
    if !bool::from(expected.ct_eq(&tag)) {
        return Err(failed());
    }
    let inverse = g.inverse_exponent(&s.key[4]);
    let mask = Zeroizing::new(u1.pow(&inverse));
    Ok(encode(&Zeroizing::new(&e * &*mask)))
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid raw group-encryption profile or exact key encoding",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "group encryption failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "group-encryption input exceeds its declared bounds",
    )
}
