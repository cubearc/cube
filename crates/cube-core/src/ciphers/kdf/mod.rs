//! Bounded raw key derivation over exact bytes. Owned result bytes are wiped on drop.
mod catalog;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Deserialize;
use sha2::Digest;
use zeroize::{Zeroize, Zeroizing};

pub const PBKDF2: &str = "cipher.kdf.pbkdf2";
pub const HKDF: &str = "cipher.kdf.hkdf";
pub const SCRYPT: &str = "cipher.kdf.scrypt";
pub const ARGON2ID: &str = "cipher.kdf.argon2id";
pub const EVP_BYTES_TO_KEY: &str = "cipher.kdf.evp_bytes_to_key";

pub const MAX_SECRET: usize = 1 << 20;
pub const MAX_CONTEXT: usize = 64 << 10;
pub const MAX_SALT_HARD: usize = 1024;
pub const MAX_OUTPUT: usize = 1024;
pub const MAX_ITERATIONS: u32 = 10_000_000;
const MAX_MEMORY: u64 = 64 << 20;
const MAX_HARD_WORK: u64 = 16 << 20;
const MAX_EVP_WORK: u32 = 1_000_000;
const MAX_ARGON2_KIB: u32 = 63 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Sha2 {
    Sha256,
    Sha512,
}
impl Sha2 {
    fn length(self) -> usize {
        match self {
            Self::Sha256 => 32,
            Self::Sha512 => 64,
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LegacyDigest {
    Md5,
    Sha256,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pbkdf2 {
    pub profile: Sha2,
    #[schemars(range(min = 1, max = 10000000))]
    pub iterations: u32,
    #[schemars(range(min = 1, max = 1024))]
    pub output_bytes: u16,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hkdf {
    pub profile: Sha2,
    #[schemars(range(min = 1, max = 1024))]
    pub output_bytes: u16,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scrypt {
    #[schemars(range(min = 2, max = 524288))]
    pub n: u32,
    #[schemars(range(min = 1, max = 1024))]
    pub r: u32,
    #[schemars(range(min = 1, max = 1024))]
    pub p: u32,
    #[schemars(range(min = 16, max = 1024))]
    pub output_bytes: u16,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Argon2id {
    #[schemars(range(min = 8, max = 64512))]
    pub memory_kib: u32,
    #[schemars(range(min = 1, max = 255))]
    pub iterations: u32,
    #[schemars(range(min = 1, max = 32))]
    pub lanes: u32,
    #[schemars(range(min = 16, max = 1024))]
    pub output_bytes: u16,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvpBytesToKey {
    pub digest: LegacyDigest,
    #[schemars(range(min = 1, max = 1000000))]
    pub count: u32,
    #[schemars(range(min = 1, max = 1024))]
    pub key_bytes: u16,
    #[schemars(range(max = 1023))]
    pub iv_bytes: u16,
}

/// Derived bytes held in a wiped buffer and redacted in debug output.
pub struct SecretBytes(Zeroizing<Vec<u8>>);
impl SecretBytes {
    /// Borrow the result without creating a copy.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}
impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBytes { [REDACTED] }")
    }
}

/// Secret key and IV. Accessors borrow result bytes without copying.
pub struct KeyIv {
    bytes: Zeroizing<Vec<u8>>,
    key_length: usize,
}
impl KeyIv {
    pub fn key(&self) -> &[u8] {
        &self.bytes[..self.key_length]
    }
    pub fn iv(&self) -> &[u8] {
        &self.bytes[self.key_length..]
    }
}
impl std::fmt::Debug for KeyIv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeyIv { [REDACTED] }")
    }
}

fn invalid(op: &'static str, parameter: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "value violates the selected KDF profile",
    )
    .operation(op)
    .parameter(parameter)
}
fn limit(op: &'static str, param: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "KDF exceeds its declared resource budget",
    )
    .operation(op)
    .parameter(param)
}
fn bounded(
    op: &'static str,
    name: &'static str,
    data: &[u8],
    min: usize,
    max: usize,
) -> Result<(), OperationError> {
    if data.len() > max {
        return Err(limit(op, name));
    }
    if data.len() < min {
        return Err(invalid(op, name));
    }
    Ok(())
}
fn output_length(op: &'static str, length: u16, min: usize) -> Result<usize, OperationError> {
    let length = usize::from(length);
    if !(min..=MAX_OUTPUT).contains(&length) {
        return Err(invalid(op, "output_bytes"));
    }
    Ok(length)
}
fn secret_buffer(length: usize) -> Zeroizing<Vec<u8>> {
    Zeroizing::new(vec![0; length])
}

/// PBKDF2-HMAC over exact password and salt bytes with an explicit iteration count.
pub fn pbkdf2(
    password: &[u8],
    salt: &[u8],
    parameters: Pbkdf2,
) -> Result<SecretBytes, OperationError> {
    bounded(PBKDF2, "password", password, 0, MAX_SECRET)?;
    bounded(PBKDF2, "salt", salt, 0, MAX_CONTEXT)?;
    let length = output_length(PBKDF2, parameters.output_bytes, 1)?;
    if parameters.iterations == 0 {
        return Err(invalid(PBKDF2, "iterations"));
    }
    if (length.div_ceil(parameters.profile.length()) as u64) * u64::from(parameters.iterations)
        > u64::from(MAX_ITERATIONS)
    {
        return Err(limit(PBKDF2, "iterations"));
    }
    let mut out = secret_buffer(length);
    match parameters.profile {
        Sha2::Sha256 => {
            pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, parameters.iterations, &mut out)
        }
        Sha2::Sha512 => {
            pbkdf2::pbkdf2_hmac::<sha2::Sha512>(password, salt, parameters.iterations, &mut out)
        }
    }
    Ok(SecretBytes(out))
}

/// RFC 5869 extract followed by expand. Salt and info may be explicitly empty.
pub fn hkdf(
    ikm: &[u8],
    salt: &[u8],
    info: &[u8],
    parameters: Hkdf,
) -> Result<SecretBytes, OperationError> {
    bounded(HKDF, "ikm", ikm, 0, MAX_SECRET)?;
    bounded(HKDF, "salt", salt, 0, MAX_CONTEXT)?;
    bounded(HKDF, "info", info, 0, MAX_CONTEXT)?;
    let length = output_length(HKDF, parameters.output_bytes, 1)?;
    let mut out = secret_buffer(length);
    let result = match parameters.profile {
        Sha2::Sha256 => hkdf::Hkdf::<sha2::Sha256>::new(Some(salt), ikm).expand(info, &mut out),
        Sha2::Sha512 => hkdf::Hkdf::<sha2::Sha512>::new(Some(salt), ikm).expand(info, &mut out),
    };
    result.map_err(|_| invalid(HKDF, "output_bytes"))?;
    Ok(SecretBytes(out))
}

/// RFC 7914 scrypt with a power-of-two N, checked memory, and checked Salsa20/8 work.
pub fn scrypt(
    password: &[u8],
    salt: &[u8],
    parameters: Scrypt,
) -> Result<SecretBytes, OperationError> {
    bounded(SCRYPT, "password", password, 0, MAX_SECRET)?;
    bounded(SCRYPT, "salt", salt, 8, MAX_SALT_HARD)?;
    let length = output_length(SCRYPT, parameters.output_bytes, 16)?;
    let Scrypt { n, r, p, .. } = parameters;
    if !n.is_power_of_two() || !(2..=524288).contains(&n) {
        return Err(invalid(SCRYPT, "n"));
    }
    if !(1..=1024).contains(&r) {
        return Err(invalid(SCRYPT, "r"));
    }
    if !(1..=1024).contains(&p) {
        return Err(invalid(SCRYPT, "p"));
    }
    let memory = 128 * u64::from(r) * (u64::from(n) + u64::from(p) + 1);
    if memory + length as u64 + 4096 > MAX_MEMORY {
        return Err(limit(SCRYPT, "n"));
    }
    if 4 * u64::from(n) * u64::from(r) * u64::from(p) > MAX_HARD_WORK {
        return Err(limit(SCRYPT, "p"));
    }
    let params = scrypt::Params::new(n.ilog2() as u8, r, p, length.min(64))
        .map_err(|_| invalid(SCRYPT, "n"))?;
    let mut out = secret_buffer(length);
    scrypt::scrypt(password, salt, &params, &mut out)
        .map_err(|_| invalid(SCRYPT, "output_bytes"))?;
    Ok(SecretBytes(out))
}

/// Argon2id v0x13 with a caller-selected lane count and bounded owned block storage.
pub fn argon2id(
    password: &[u8],
    salt: &[u8],
    parameters: Argon2id,
) -> Result<SecretBytes, OperationError> {
    bounded(ARGON2ID, "password", password, 0, MAX_SECRET)?;
    bounded(ARGON2ID, "salt", salt, 8, MAX_SALT_HARD)?;
    let length = output_length(ARGON2ID, parameters.output_bytes, 16)?;
    let Argon2id {
        memory_kib,
        iterations,
        lanes,
        ..
    } = parameters;
    if !(1..=32).contains(&lanes) {
        return Err(invalid(ARGON2ID, "lanes"));
    }
    if !(1..=255).contains(&iterations) {
        return Err(invalid(ARGON2ID, "iterations"));
    }
    if memory_kib > MAX_ARGON2_KIB || u64::from(memory_kib) * u64::from(iterations) > MAX_HARD_WORK
    {
        return Err(limit(ARGON2ID, "memory_kib"));
    }
    let settings = argon2::Params::new(memory_kib, iterations, lanes, Some(length))
        .map_err(|_| invalid(ARGON2ID, "memory_kib"))?;
    let count = settings.block_count();
    let engine = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        settings,
    );
    let mut blocks = Zeroizing::new(Vec::new());
    blocks
        .try_reserve_exact(count)
        .map_err(|_| limit(ARGON2ID, "memory_kib"))?;
    blocks.resize(count, argon2::Block::default());
    let mut out = secret_buffer(length);
    engine
        .hash_password_into_with_memory(password, salt, &mut out, &mut blocks[..])
        .map_err(|_| invalid(ARGON2ID, "output_bytes"))?;
    Ok(SecretBytes(out))
}

/// OpenSSL EVP_BytesToKey chaining with a required eight-byte salt. This is
/// for legacy interoperability with existing encrypted artifacts.
pub fn evp_bytes_to_key(
    password: &[u8],
    salt: &[u8],
    settings: EvpBytesToKey,
) -> Result<KeyIv, OperationError> {
    let op = EVP_BYTES_TO_KEY;
    bounded(op, "password", password, 0, MAX_SECRET)?;
    if salt.len() != 8 {
        return Err(invalid(op, "salt"));
    }
    let key_length = usize::from(settings.key_bytes);
    let length = key_length + usize::from(settings.iv_bytes);
    if key_length == 0 || length > MAX_OUTPUT {
        return Err(invalid(op, "key_bytes"));
    }
    if settings.count == 0 {
        return Err(invalid(op, "count"));
    }
    let block = match settings.digest {
        LegacyDigest::Md5 => 16,
        LegacyDigest::Sha256 => 32,
    };
    if (length.div_ceil(block) as u64) * u64::from(settings.count) > u64::from(MAX_EVP_WORK) {
        return Err(limit(op, "count"));
    }
    let mut bytes = secret_buffer(length);
    evp_chain(settings.digest, password, salt, settings.count, &mut bytes);
    Ok(KeyIv { bytes, key_length })
}

pub(crate) fn evp_chain(
    digest: LegacyDigest,
    password: &[u8],
    salt: &[u8],
    count: u32,
    output: &mut [u8],
) {
    match digest {
        LegacyDigest::Md5 => chain::<md5::Md5>(password, salt, count, output),
        LegacyDigest::Sha256 => chain::<sha2::Sha256>(password, salt, count, output),
    }
}
fn chain<D: Digest>(password: &[u8], salt: &[u8], count: u32, output: &mut [u8]) {
    let mut previous = Zeroizing::new(Vec::new());
    for chunk in output.chunks_mut(<D as Digest>::output_size()) {
        let mut hash = D::new();
        hash.update(&previous[..]);
        hash.update(password);
        hash.update(salt);
        let mut next = hash.finalize();
        previous.zeroize();
        previous.extend_from_slice(&next);
        next.as_mut_slice().zeroize();
        for _ in 1..count {
            let mut repeated = D::digest(&previous[..]);
            previous.copy_from_slice(&repeated);
            repeated.as_mut_slice().zeroize();
        }
        chunk.copy_from_slice(&previous[..chunk.len()]);
    }
}

pub(crate) fn supports(operation: &str) -> bool {
    matches!(
        operation,
        PBKDF2 | HKDF | SCRYPT | ARGON2ID | EVP_BYTES_TO_KEY
    )
}
