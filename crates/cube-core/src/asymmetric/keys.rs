//! Bounded RSA SPKI/PKCS8 parsing before private arithmetic.
use crate::errors::{ErrorCode, OperationError};
use rsa::{
    pkcs8::{
        self,
        der::{Decode, Encode},
    },
    traits::PublicKeyParts,
};
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::Zeroizing;
pub const MAX_KEY: usize = 65536;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicFormat {
    SpkiDer,
    SpkiPem,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivateFormat {
    Pkcs8Der,
    Pkcs8Pem,
}
fn der(input: &[u8], pem: bool, label: &str) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    if input.len() > MAX_KEY {
        return Err(limit());
    }
    if !pem {
        return Ok(Zeroizing::new(input.to_vec()));
    }
    let (kind, decoded) = pkcs8::der::pem::decode_vec(input).map_err(|_| invalid())?;
    let decoded = Zeroizing::new(decoded);
    if kind != label {
        return Err(invalid());
    }
    let canonical = Zeroizing::new(
        pkcs8::der::pem::encode_string(label, pkcs8::der::pem::LineEnding::LF, &decoded)
            .map_err(|_| invalid())?,
    );
    let normalized = Zeroizing::new(
        std::str::from_utf8(input)
            .map_err(|_| invalid())?
            .replace("\r\n", "\n"),
    );
    if normalized.as_str() != canonical.as_str()
        && normalized.as_str() != canonical.strip_suffix('\n').unwrap_or(&canonical)
    {
        return Err(invalid());
    }
    Ok(decoded)
}
fn modulus(bytes: &[u8], e: &[u8]) -> Result<(), OperationError> {
    let first = *bytes.first().ok_or_else(invalid)?;
    let bits = bytes.len().checked_mul(8).ok_or_else(limit)? - first.leading_zeros() as usize;
    if !(2048..=8192).contains(&bits)
        || e.is_empty()
        || e.len() > 5
        || bytes.last().is_none_or(|b| b & 1 == 0)
    {
        return Err(invalid());
    }
    Ok(())
}
/// Decode exactly one canonical SPKI RSA encryption key, at most64KiB.
pub fn rsa_public(input: &[u8], format: PublicFormat) -> Result<rsa::PublicKey, OperationError> {
    let data = der(input, matches!(format, PublicFormat::SpkiPem), "PUBLIC KEY")?;
    let info = pkcs8::SubjectPublicKeyInfoRef::from_der(&data).map_err(|_| invalid())?;
    if info.algorithm != rsa::pkcs1::ALGORITHM_ID || info.to_der().map_err(|_| invalid())? != *data
    {
        return Err(invalid());
    }
    let inner = info.subject_public_key.as_bytes().ok_or_else(invalid)?;
    let components = rsa::pkcs1::RsaPublicKeyRef::from_der(inner).map_err(|_| invalid())?;
    modulus(
        components.modulus.as_bytes(),
        components.public_exponent.as_bytes(),
    )?;
    rsa::PublicKey::try_from(info).map_err(|_| invalid())
}
/// Decode a canonical two-prime PKCS8v1 RSA encryption key, at most64KiB.
/// Validate sizes before bigint construction, then backend arithmetic and CRT
/// consistency. Encrypted keys, attributes, PSS-only OIDs and trailing data fail.
pub fn rsa_private(input: &[u8], format: PrivateFormat) -> Result<rsa::PrivateKey, OperationError> {
    let data = der(
        input,
        matches!(format, PrivateFormat::Pkcs8Pem),
        "PRIVATE KEY",
    )?;
    let info = pkcs8::PrivateKeyInfoRef::from_der(&data).map_err(|_| invalid())?;
    if info.algorithm != rsa::pkcs1::ALGORITHM_ID
        || info.public_key.is_some()
        || Zeroizing::new(info.to_der().map_err(|_| invalid())?) != data
    {
        return Err(invalid());
    }
    let components = rsa::pkcs1::RsaPrivateKeyRef::from_der(info.private_key.as_bytes())
        .map_err(|_| invalid())?;
    modulus(
        components.modulus.as_bytes(),
        components.public_exponent.as_bytes(),
    )?;
    if components.version() != rsa::pkcs1::Version::TwoPrime {
        return Err(invalid());
    }
    for part in [
        components.private_exponent,
        components.prime1,
        components.prime2,
        components.exponent1,
        components.exponent2,
        components.coefficient,
    ] {
        if part.as_bytes().is_empty() || part.as_bytes().len() > 1024 {
            return Err(invalid());
        }
    }
    let key = rsa::PrivateKey::try_from(info).map_err(|_| invalid())?;
    key.validate().map_err(|_| invalid())?;
    use pkcs8::EncodePrivateKey;
    let encoded = key.to_pkcs8_der().map_err(|_| invalid())?;
    if encoded.as_bytes() != data.as_slice() {
        return Err(invalid());
    }
    Ok(key)
}
/// Checked ciphertext width in octets, including a possible partial high byte.
pub(super) fn width(key: &impl PublicKeyParts) -> usize {
    key.n().bit_width_vartime().div_ceil(8) as usize
}
pub(super) fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid RSA key encoding or profile",
    )
}
fn limit() -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, "encoded key exceeds65536 bytes")
}
