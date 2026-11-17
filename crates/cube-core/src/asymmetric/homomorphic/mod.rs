//! Public operations on fixed homomorphic ciphertext representations.
mod catalog;

use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use serde::Deserialize;

pub const BFV_ADD: &str = "cipher.bfv.add";
pub const BGV_ADD: &str = "cipher.bgv.add";
pub const CKKS_ADD: &str = "cipher.ckks.add";
pub const TFHE_NOT: &str = "cipher.tfhe.not";
pub const MAX_PARAMETERS: usize = 256;
pub(crate) const SCRATCH_BYTES: usize = 256 * 1024;

/// Fixed interoperability profiles, with format tags before the raw words.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Profile {
    Bfv,
    Bgv,
    Ckks,
    Tfhe,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters<'a> {
    profile: &'a str,
}

impl Profile {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::Bfv => BFV_ADD,
            Self::Bgv => BGV_ADD,
            Self::Ckks => CKKS_ADD,
            Self::Tfhe => TFHE_NOT,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Bfv => "bfv_seal412_degree1024_t17_raw_v1",
            Self::Bgv => "bgv_seal412_degree1024_t17_ntt_raw_v1",
            Self::Ckks => "ckks_seal412_degree2048_scale32_ntt_raw_v1",
            Self::Tfhe => "tfhe_rs182_boolean_default_lwe805_raw_v1",
        }
    }

    pub(crate) fn from_operation(operation: &str) -> Option<Self> {
        match operation {
            BFV_ADD => Some(Self::Bfv),
            BGV_ADD => Some(Self::Bgv),
            CKKS_ADD => Some(Self::Ckks),
            TFHE_NOT => Some(Self::Tfhe),
            _ => None,
        }
    }

    pub const fn bytes(self) -> usize {
        match self {
            Self::Bfv | Self::Bgv => 4 + 2 * 1024 * 8,
            Self::Ckks => 4 + 2 * 2048 * 8,
            Self::Tfhe => 4 + 806 * 4,
        }
    }

    const fn tag(self) -> &'static [u8; 4] {
        match self {
            Self::Bfv => b"CBF1",
            Self::Bgv => b"CBG1",
            Self::Ckks => b"CCK1",
            Self::Tfhe => b"CTF1",
        }
    }

    /// Validate that the settings name this exact profile, without defaults.
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
            "use the required homomorphic ciphertext profile",
        ))
    }

    fn payload(self, input: &[u8]) -> Result<&[u8], OperationError> {
        if input.len() > self.bytes() {
            return Err(limit());
        }
        if input.len() != self.bytes() || !input.starts_with(self.tag()) {
            return Err(invalid());
        }
        Ok(&input[4..])
    }
}

/// Add two canonical ciphertexts at the same fixed level and scale. Inputs
/// must use the same recipient key and the profile's coefficient/NTT basis.
pub fn add(left: &[u8], right: &[u8], profile: Profile) -> Result<Vec<u8>, OperationError> {
    let modulus = match profile {
        Profile::Bfv | Profile::Bgv => 132_120_577,
        Profile::Ckks => 18_014_398_492_704_769,
        Profile::Tfhe => return Err(invalid()),
    };
    let (left, _) = profile.payload(left)?.as_chunks::<8>();
    let (right, _) = profile.payload(right)?.as_chunks::<8>();
    let mut output = Vec::with_capacity(profile.bytes());
    output.extend_from_slice(profile.tag());
    for (a, b) in left.iter().zip(right) {
        let a = u64::from_le_bytes(*a);
        let b = u64::from_le_bytes(*b);
        if a >= modulus || b >= modulus {
            return Err(invalid());
        }
        output.extend_from_slice(&((a + b) % modulus).to_le_bytes());
    }
    Ok(output)
}

/// Negate the complete fixed-profile Torus32 LWE ciphertext to evaluate a
/// Boolean NOT. This linear gate preserves the magnitude of the input noise.
pub fn tfhe_not(ciphertext: &[u8]) -> Result<Vec<u8>, OperationError> {
    let profile = Profile::Tfhe;
    let (words, _) = profile.payload(ciphertext)?.as_chunks::<4>();
    let mut output = Vec::with_capacity(profile.bytes());
    output.extend_from_slice(profile.tag());
    for word in words {
        output.extend_from_slice(&u32::from_le_bytes(*word).wrapping_neg().to_le_bytes());
    }
    Ok(output)
}

fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "invalid homomorphic ciphertext encoding",
    )
}

fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "homomorphic input exceeds its profile bound",
    )
}
