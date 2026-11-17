//! Small educational examples of the original NTRUEncrypt and Regev PKE constructions.
mod catalog;
mod ntru;
mod regev;

use super::entropy::SystemRng;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use rand_core::TryRng;
use serde::Deserialize;

pub const NTRU_ENCRYPT: &str = "cipher.ntru_encrypt.encrypt";
pub const NTRU_DECRYPT: &str = "cipher.ntru_encrypt.decrypt";
pub const REGEV_ENCRYPT: &str = "cipher.regev.encrypt";
pub const REGEV_DECRYPT: &str = "cipher.regev.decrypt";
pub const MAX_PARAMETERS: usize = 256;
pub(crate) const SCRATCH_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum Profile {
    NtruEncrypt,
    Regev,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters<'a> {
    profile: &'a str,
}

impl Profile {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NtruEncrypt => "ntru_encrypt_educational_n11_p3_q32",
            Self::Regev => "regev_educational_n4_m16_q97",
        }
    }

    pub const fn widths(self) -> (usize, usize, usize, usize) {
        // Plaintext, public key, private key and ciphertext bytes.
        match self {
            Self::NtruEncrypt => (11, 11, 22, 11),
            Self::Regev => (1, 80, 4, 5),
        }
    }

    pub(crate) fn from_operation(operation: &str) -> Option<(Self, bool)> {
        match operation {
            NTRU_ENCRYPT => Some((Self::NtruEncrypt, true)),
            NTRU_DECRYPT => Some((Self::NtruEncrypt, false)),
            REGEV_ENCRYPT => Some((Self::Regev, true)),
            REGEV_DECRYPT => Some((Self::Regev, false)),
            _ => None,
        }
    }

    pub fn validate_parameters(self, document: &str) -> Result<(), OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        if serde_json::from_str::<Parameters<'_>>(document)
            .is_ok_and(|parameters| parameters.profile == self.name())
        {
            return Ok(());
        }
        Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "use the exact educational lattice profile",
        ))
    }
}

/// Encrypt one small educational message using fresh system randomness.
pub fn encrypt(input: &[u8], key: &[u8], profile: Profile) -> Result<Vec<u8>, OperationError> {
    let (message, public_key, _, _) = profile.widths();
    check_lengths(input, key, message, public_key)?;
    let mut rng = SystemRng::bounded(64);
    match profile {
        Profile::NtruEncrypt => ntru::encrypt(input, key, ntru_indices(&mut rng)?),
        Profile::Regev => {
            let mut subset = [0; 2];
            rng.try_fill_bytes(&mut subset)
                .map_err(|_| entropy_error())?;
            regev::encrypt(input, key, subset)
        }
    }
}

/// Decrypt one small educational ciphertext with its matching demonstration key.
pub fn decrypt(input: &[u8], key: &[u8], profile: Profile) -> Result<Vec<u8>, OperationError> {
    let (_, _, private_key, ciphertext) = profile.widths();
    check_lengths(input, key, ciphertext, private_key)?;
    match profile {
        Profile::NtruEncrypt => ntru::decrypt(input, key),
        Profile::Regev => regev::decrypt(input, key),
    }
}

fn ntru_indices(rng: &mut impl TryRng) -> Result<[usize; 2], OperationError> {
    // 220 equally likely byte values give two representatives of each
    // ordered pair of distinct positions in Z_11.
    for _ in 0..64 {
        let mut sample = [0; 1];
        rng.try_fill_bytes(&mut sample)
            .map_err(|_| entropy_error())?;
        if sample[0] < 220 {
            let pair = usize::from(sample[0] % 110);
            let positive = pair / 10;
            let other = pair % 10;
            return Ok([positive, other + usize::from(other >= positive)]);
        }
    }
    Err(entropy_error())
}

fn check_lengths(
    input: &[u8],
    key: &[u8],
    input_bytes: usize,
    key_bytes: usize,
) -> Result<(), OperationError> {
    if input.len() > input_bytes || key.len() > key_bytes {
        return Err(limit());
    }
    if input.len() != input_bytes || key.len() != key_bytes {
        return Err(invalid());
    }
    Ok(())
}

fn entropy_error() -> OperationError {
    OperationError::new(ErrorCode::Io, "educational lattice sampling failed")
}

fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "noncanonical educational lattice message, key or ciphertext",
    )
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "input exceeds its educational lattice profile width",
    )
}
