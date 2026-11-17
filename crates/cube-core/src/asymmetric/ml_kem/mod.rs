//! FIPS 203 key encapsulation over the portable Libcrux implementation.
mod catalog;
#[cfg(test)]
mod tests;

use super::entropy::SystemRng;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use libcrux_ml_kem::{mlkem512, mlkem768, mlkem1024};
use rand_core::TryCryptoRng;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::Zeroizing;

pub const KEYGEN: &str = "crypto.ml_kem.keygen";
pub const ENCAPSULATE: &str = "crypto.ml_kem.encapsulate";
pub const DECAPSULATE: &str = "crypto.ml_kem.decapsulate";
pub const MAX_PARAMETERS: usize = 256;
pub const MAX_PUBLIC_KEY: usize = 1568;
pub const MAX_PRIVATE_KEY: usize = 3168;
pub const MAX_CIPHERTEXT: usize = 1568;
pub const SHARED_SECRET_BYTES: usize = 32;
// A fixed reservation for one portable backend call, including packing and
// stack scratch. The worker retains its separate process memory limit.
pub(crate) const MEMORY_RESERVATION: usize = 1024 * 1024;

/// Explicit FIPS 203 parameter set for raw wire encodings.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[serde(rename = "ml_kem_512_fips203")]
    MlKem512,
    #[serde(rename = "ml_kem_768_fips203")]
    MlKem768,
    #[serde(rename = "ml_kem_1024_fips203")]
    MlKem1024,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
}

impl Profile {
    pub const NAMES: &[&str] = &[
        "ml_kem_512_fips203",
        "ml_kem_768_fips203",
        "ml_kem_1024_fips203",
    ];

    /// Parse a bounded JSON object containing exactly one explicit `profile`.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        serde_json::from_str::<Parameters>(document)
            .map(|p| p.profile)
            .map_err(|_| {
                OperationError::new(
                    ErrorCode::InvalidParameter,
                    "use an explicit ML-KEM profile",
                )
            })
    }
}

/// Generated raw public key and expanded private key. The owned private bytes
/// are wiped on drop. Caller-created copies and backend temporaries may persist.
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Zeroizing<Vec<u8>>,
}

/// Encapsulated ciphertext and its 32-byte shared secret.
pub struct Encapsulation {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Zeroizing<[u8; SHARED_SECRET_BYTES]>,
}

macro_rules! portable_profile {
    ($profile:expr, |$backend:ident| $body:block) => {
        match $profile {
            Profile::MlKem512 => {
                use mlkem512::portable as $backend;
                $body
            }
            Profile::MlKem768 => {
                use mlkem768::portable as $backend;
                $body
            }
            Profile::MlKem1024 => {
                use mlkem1024::portable as $backend;
                $body
            }
        }
    };
}

/// Generate one fresh ML-KEM key pair using system entropy.
pub fn keygen(profile: Profile) -> Result<KeyPair, OperationError> {
    generate(profile, &mut SystemRng::bounded(64))
}

fn generate(profile: Profile, rng: &mut impl TryCryptoRng) -> Result<KeyPair, OperationError> {
    let seed = random(rng)?;
    Ok(portable_profile!(profile, |backend| {
        let keys = backend::generate_key_pair(*seed);
        KeyPair {
            public_key: keys.pk().to_vec(),
            private_key: Zeroizing::new(keys.sk().to_vec()),
        }
    }))
}

/// Encapsulate a fresh shared secret to one canonical raw public key.
pub fn encapsulate(public_key: &[u8], profile: Profile) -> Result<Encapsulation, OperationError> {
    encapsulate_with_rng(public_key, profile, &mut SystemRng::bounded(32))
}

fn encapsulate_with_rng(
    public_key: &[u8],
    profile: Profile,
    rng: &mut impl TryCryptoRng,
) -> Result<Encapsulation, OperationError> {
    if public_key.len() > MAX_PUBLIC_KEY {
        return Err(limit());
    }
    portable_profile!(profile, |backend| {
        let key = public_key.try_into().map_err(|_| invalid_input())?;
        if !backend::validate_public_key(&key) {
            return Err(invalid_input());
        }
        let randomness = random(rng)?;
        let (ciphertext, shared) = backend::encapsulate(&key, *randomness);
        Ok(Encapsulation {
            ciphertext: ciphertext.as_ref().to_vec(),
            shared_secret: Zeroizing::new(shared),
        })
    })
}

/// Decapsulate one exact-width ciphertext with a FIPS 203 expanded key.
/// Correct-width ciphertexts use implicit rejection and always return 32 bytes
/// after the key checks succeed. Callers receive no ciphertext validity flag.
pub fn decapsulate(
    ciphertext: &[u8],
    private_key: &[u8],
    profile: Profile,
) -> Result<Zeroizing<[u8; SHARED_SECRET_BYTES]>, OperationError> {
    if ciphertext.len() > MAX_CIPHERTEXT || private_key.len() > MAX_PRIVATE_KEY {
        return Err(limit());
    }
    portable_profile!(profile, |backend| {
        let key = private_key.try_into().map_err(|_| invalid_input())?;
        let ciphertext = ciphertext.try_into().map_err(|_| invalid_input())?;
        if !backend::validate_private_key(&key, &ciphertext) {
            return Err(invalid_input());
        }
        Ok(Zeroizing::new(backend::decapsulate(&key, &ciphertext)))
    })
}

fn random<const N: usize>(
    rng: &mut impl TryCryptoRng,
) -> Result<Zeroizing<[u8; N]>, OperationError> {
    let mut random = Zeroizing::new([0; N]);
    rng.try_fill_bytes(&mut random[..])
        .map_err(|_| OperationError::new(ErrorCode::Io, "ML-KEM system entropy unavailable"))?;
    Ok(random)
}

fn invalid_input() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "invalid ML-KEM key or ciphertext encoding",
    )
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "ML-KEM input exceeds its transport bound",
    )
}
