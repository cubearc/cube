//! Explicit, reproducible non-cryptographic and keyed digest profiles.
use crate::{
    errors::OperationError,
    formal_work::{parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use twox_hash::{XxHash32, XxHash64};
use zeroize::Zeroizing;

const MAX_DATA: usize = 16 << 20;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Xxh32,
    Xxh64,
    Murmur3X86_32,
    Murmur3X64_128,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub profile: Profile,
    #[serde(default)]
    pub seed: u64,
}

#[derive(Serialize, JsonSchema)]
pub struct Digest {
    pub hex: String,
}

fn check(data: &[u8]) -> Result<(), OperationError> {
    if data.len() > MAX_DATA {
        return Err(resource("digest input exceeds 16 MiB"));
    }
    Ok(())
}

impl Options {
    pub(crate) fn seed32(self) -> Result<u32, OperationError> {
        u32::try_from(self.seed)
            .map_err(|_| parameter("seed must fit an unsigned 32-bit integer").parameter("seed"))
    }

    pub(crate) fn validate(self) -> Result<Self, OperationError> {
        if !matches!(self.profile, Profile::Xxh64) {
            self.seed32()?;
        }
        Ok(self)
    }

    pub(crate) fn hex_length(self) -> usize {
        match self.profile {
            Profile::Xxh32 | Profile::Murmur3X86_32 => 8,
            Profile::Xxh64 => 16,
            Profile::Murmur3X64_128 => 32,
        }
    }
}

/// Hash data with canonical digest byte order for the selected reference profile.
pub fn noncrypto(data: &[u8], options: Options) -> Result<Digest, OperationError> {
    check(data)?;
    let hex = match options.profile {
        Profile::Xxh32 => hex::encode(XxHash32::oneshot(options.seed32()?, data).to_be_bytes()),
        Profile::Xxh64 => hex::encode(XxHash64::oneshot(options.seed, data).to_be_bytes()),
        Profile::Murmur3X86_32 => {
            hex::encode(hashcrew::murmur::murmur3_x86_32(data, options.seed32()?).to_le_bytes())
        }
        Profile::Murmur3X64_128 => {
            hex::encode(hashcrew::murmur::murmur3_x64_128(data, options.seed32()?).to_le_bytes())
        }
    };
    Ok(Digest { hex })
}

/// Calculate SipHash-2-4 with a caller-provided exact 16-byte key.
pub fn siphash24(data: &[u8], key: &[u8]) -> Result<Digest, OperationError> {
    check(data)?;
    let key: Zeroizing<[u8; 16]> =
        Zeroizing::new(key.try_into().map_err(|_| {
            parameter("SipHash-2-4 requires an exact 16-byte key").parameter("key")
        })?);
    let mut state = siphasher::sip::SipHasher24::new_with_key(&key);
    state.write(data);
    Ok(Digest {
        hex: hex::encode(state.finish().to_le_bytes()),
    })
}
