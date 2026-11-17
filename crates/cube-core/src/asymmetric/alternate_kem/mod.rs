//! Fixed alternate KEM profiles with bounded raw-key decapsulation.
mod catalog;

use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MEMORY_RESERVATION;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const FRODO: &str = "crypto.frodo_kem.decapsulate";
pub const HQC: &str = "crypto.hqc.decapsulate";
pub const MCELIECE: &str = "crypto.classic_mceliece.decapsulate";
pub const MAX_PARAMETERS: usize = 256;

/// Exact raw-key decapsulation profile.
#[derive(Clone, Copy, Debug)]
pub enum Profile {
    Frodo640Shake,
    Hqc128February2026,
    McEliece460896f,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters<'a> {
    profile: &'a str,
}

impl Profile {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::Frodo640Shake => FRODO,
            Self::Hqc128February2026 => HQC,
            Self::McEliece460896f => MCELIECE,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Frodo640Shake => "frodokem640_shake_salted",
            Self::Hqc128February2026 => "hqc128_20260210_submission",
            Self::McEliece460896f => "mceliece460896f_round4",
        }
    }

    pub(crate) fn for_operation(operation: &str) -> Option<Self> {
        match operation {
            FRODO => Some(Self::Frodo640Shake),
            HQC => Some(Self::Hqc128February2026),
            MCELIECE => Some(Self::McEliece460896f),
            _ => None,
        }
    }

    /// Exact raw widths in bytes, ordered as private key, ciphertext and secret.
    pub const fn widths(self) -> (usize, usize, usize) {
        match self {
            Self::Frodo640Shake => (19888, 9752, 16),
            Self::Hqc128February2026 => (2321, 4433, 32),
            Self::McEliece460896f => (13608, 156, 32),
        }
    }

    /// Match an explicit profile in a bounded JSON document to this operation.
    pub fn validate_parameters(self, document: &str) -> Result<(), OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let expected = serde_json::from_str::<Parameters<'_>>(document)
            .is_ok_and(|parameters| parameters.profile == self.name());
        if expected {
            Ok(())
        } else {
            Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "use the required KEM profile",
            ))
        }
    }
}

/// Decapsulate at one exact profile and return the private shared secret.
/// A correctly sized altered ciphertext goes through backend implicit rejection.
pub fn decapsulate(
    ciphertext: &[u8],
    private_key: &[u8],
    profile: Profile,
) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    let (key_bytes, ciphertext_bytes, _) = profile.widths();
    if private_key.len() > key_bytes || ciphertext.len() > ciphertext_bytes {
        return Err(limit());
    }
    if private_key.len() != key_bytes || ciphertext.len() != ciphertext_bytes {
        return Err(invalid());
    }
    match profile {
        Profile::Frodo640Shake => frodo(ciphertext, private_key),
        Profile::Hqc128February2026 => hqc(ciphertext, private_key),
        Profile::McEliece460896f => mceliece(ciphertext, private_key),
    }
}

fn frodo(ciphertext: &[u8], key: &[u8]) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    let profile = frodo_kem::Algorithm::FrodoKem640Shake;
    let ciphertext = profile
        .ciphertext_from_bytes(ciphertext)
        .map_err(|_| invalid())?;
    let key = profile
        .decryption_key_from_bytes(key)
        .map_err(|_| invalid())?;
    let (shared, mut message) = profile
        .decapsulate(&key, &ciphertext)
        .map_err(|_| invalid())?;
    message.zeroize();
    Ok(Zeroizing::new(shared.value().to_vec()))
}

fn hqc(ciphertext: &[u8], key: &[u8]) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    use hqc_kem::hqc128::{Ciphertext, DecapsulationKey};
    let key = DecapsulationKey::try_from(key).map_err(|_| invalid())?;
    let ciphertext = Ciphertext::try_from(ciphertext).map_err(|_| invalid())?;
    Ok(Zeroizing::new(
        key.decapsulate(&ciphertext).as_ref().to_vec(),
    ))
}

fn mceliece(ciphertext: &[u8], key: &[u8]) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    let profile = pq_mceliece::Algorithm::McEliece460896f;
    let ciphertext = profile
        .ciphertext_from_bytes(ciphertext)
        .map_err(|_| invalid())?;
    let key = profile
        .decapsulation_key_from_bytes(key)
        .map_err(|_| invalid())?;
    let shared = profile
        .decapsulate(&key, &ciphertext)
        .map_err(|_| invalid())?;
    Ok(Zeroizing::new(shared.value().to_vec()))
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "KEM input exceeds the profile's exact byte width",
    )
}

fn invalid() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "invalid KEM input encoding")
}
