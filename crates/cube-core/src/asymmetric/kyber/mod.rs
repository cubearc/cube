//! Historical CRYSTALS-Kyber 3.0 SHAKE decapsulation compatibility.
mod catalog;

use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use libcrux_ml_kem::{MlKemCiphertext, MlKemPrivateKey, mlkem512, mlkem768, mlkem1024};
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::Zeroizing;

pub const DECAPSULATE: &str = "crypto.kyber.decapsulate";
pub const MAX_PARAMETERS: usize = 256;
pub const MAX_CIPHERTEXT: usize = 1568;
pub const MAX_PRIVATE_KEY: usize = 3168;
pub const SHARED_SECRET_BYTES: usize = 32;

/// Explicit SHAKE parameter set of CRYSTALS-Kyber 3.0 Round 3.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "kyber512_round3_shake")]
    Kyber512,
    #[serde(rename = "kyber768_round3_shake")]
    Kyber768,
    #[serde(rename = "kyber1024_round3_shake")]
    Kyber1024,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
}

impl Profile {
    pub const NAMES: &[&str] = &[
        "kyber512_round3_shake",
        "kyber768_round3_shake",
        "kyber1024_round3_shake",
    ];

    /// Parse the required historical profile from a bounded JSON object.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        serde_json::from_str::<Parameters>(document)
            .map(|p| p.profile)
            .map_err(|_| {
                OperationError::new(
                    ErrorCode::InvalidParameter,
                    "use an explicit Kyber Round 3 SHAKE profile",
                )
            })
    }
}

/// Decapsulate a raw Round 3 ciphertext with an exact-width expanded key.
/// A correct-width ciphertext returns a deterministic secret with implicit
/// rejection. Use this operation to recover data from a historical exchange.
pub fn decapsulate(
    ciphertext: &[u8],
    private_key: &[u8],
    profile: Profile,
) -> Result<Zeroizing<[u8; SHARED_SECRET_BYTES]>, OperationError> {
    if ciphertext.len() > MAX_CIPHERTEXT || private_key.len() > MAX_PRIVATE_KEY {
        return Err(limit());
    }
    match profile {
        Profile::Kyber512 => decapsulate_exact(
            ciphertext,
            private_key,
            mlkem512::portable::validate_private_key,
            mlkem512::portable::kyber_decapsulate,
        ),
        Profile::Kyber768 => decapsulate_exact(
            ciphertext,
            private_key,
            mlkem768::portable::validate_private_key,
            mlkem768::portable::kyber_decapsulate,
        ),
        Profile::Kyber1024 => decapsulate_exact(
            ciphertext,
            private_key,
            mlkem1024::portable::validate_private_key,
            mlkem1024::portable::kyber_decapsulate,
        ),
    }
}

fn decapsulate_exact<const KEY: usize, const CT: usize>(
    ciphertext: &[u8],
    private_key: &[u8],
    validate: impl FnOnce(&MlKemPrivateKey<KEY>, &MlKemCiphertext<CT>) -> bool,
    decapsulate: impl FnOnce(&MlKemPrivateKey<KEY>, &MlKemCiphertext<CT>) -> [u8; SHARED_SECRET_BYTES],
) -> Result<Zeroizing<[u8; SHARED_SECRET_BYTES]>, OperationError> {
    let ciphertext = ciphertext.try_into().map_err(|_| invalid())?;
    let private_key = private_key.try_into().map_err(|_| invalid())?;
    if !validate(&private_key, &ciphertext) {
        return Err(invalid());
    }
    Ok(Zeroizing::new(decapsulate(&private_key, &ciphertext)))
}

fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "invalid Kyber Round 3 key or ciphertext encoding",
    )
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "Kyber Round 3 input exceeds its transport bound",
    )
}
