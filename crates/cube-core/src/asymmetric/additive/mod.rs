//! Fixed-width public evaluation in Paillier and Damgård–Jurik ciphertext groups.
mod catalog;

use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use num_bigint::BigUint;
use num_integer::Integer;
use serde::Deserialize;

pub const PAILLIER_ADD: &str = "cipher.paillier.add";
pub const DAMGARD_JURIK_ADD: &str = "cipher.damgard_jurik.add";
pub const MAX_PARAMETERS: usize = 256;
pub const MAX_MODULUS_BYTES: usize = 512;
pub(crate) const SCRATCH_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy)]
pub enum Profile {
    Paillier,
    DamgardJurik,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters<'a> {
    profile: &'a str,
}

impl Profile {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::Paillier => PAILLIER_ADD,
            Self::DamgardJurik => DAMGARD_JURIK_ADD,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Paillier => "paillier_g_n_plus_1_be_v1",
            Self::DamgardJurik => "damgard_jurik_s2_g_n_plus_1_be_v1",
        }
    }

    pub(crate) fn from_operation(operation: &str) -> Option<Self> {
        match operation {
            PAILLIER_ADD => Some(Self::Paillier),
            DAMGARD_JURIK_ADD => Some(Self::DamgardJurik),
            _ => None,
        }
    }

    const fn power(self) -> u32 {
        match self {
            Self::Paillier => 2,
            Self::DamgardJurik => 3,
        }
    }

    pub const fn ciphertext_bytes(self, modulus_bytes: usize) -> usize {
        modulus_bytes.saturating_mul(self.power() as usize)
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
            "use the required additive ciphertext profile",
        ))
    }
}

/// Add two encrypted plaintext residues by multiplying their ciphertexts.
/// The public modulus must come from the same externally validated key.
pub fn add(
    left: &[u8],
    right: &[u8],
    modulus: &[u8],
    profile: Profile,
) -> Result<Vec<u8>, OperationError> {
    let maximum = profile.ciphertext_bytes(MAX_MODULUS_BYTES);
    if modulus.len() > MAX_MODULUS_BYTES || left.len() > maximum || right.len() > maximum {
        return Err(limit());
    }
    if !matches!(modulus.len(), 256 | 384 | 512)
        || modulus[0] & 0x80 == 0
        || modulus[modulus.len() - 1] & 1 == 0
    {
        return Err(invalid());
    }

    let width = profile.ciphertext_bytes(modulus.len());
    if left.len() != width || right.len() != width {
        return Err(invalid());
    }
    let n = BigUint::from_bytes_be(modulus);
    let ciphertext_modulus = n.pow(profile.power());
    let left = ciphertext(left, &n, &ciphertext_modulus)?;
    let right = ciphertext(right, &n, &ciphertext_modulus)?;
    let result = (left * right % ciphertext_modulus).to_bytes_be();
    let mut output = vec![0; width];
    output[width - result.len()..].copy_from_slice(&result);
    Ok(output)
}

fn ciphertext(
    value: &[u8],
    n: &BigUint,
    ciphertext_modulus: &BigUint,
) -> Result<BigUint, OperationError> {
    let value = BigUint::from_bytes_be(value);
    if &value >= ciphertext_modulus || value.gcd(n) != BigUint::from(1_u8) {
        return Err(invalid());
    }
    Ok(value)
}

fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "expected fixed-width canonical additive ciphertexts and a matching public modulus",
    )
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "additive ciphertext input exceeds its profile bound",
    )
}
