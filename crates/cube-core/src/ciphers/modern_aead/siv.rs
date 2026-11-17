//! RFC5297 nonce-free SIV with ordered, independently delimited AAD components.
use super::*;
use aes_siv::{
    KeyInit,
    siv::{Aes128Siv, Aes256Siv},
};
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum SivProfile {
    #[serde(rename = "aes128_siv_cmac_nonce_free")]
    Aes128,
    #[serde(rename = "aes256_siv_cmac_nonce_free")]
    Aes256,
}
impl SivProfile {
    pub const NAMES: &[&str] = &["aes128_siv_cmac_nonce_free", "aes256_siv_cmac_nonce_free"];
    pub(super) fn key_size(self) -> usize {
        match self {
            Self::Aes128 => 32,
            Self::Aes256 => 64,
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct Parameters {
    profile: SivProfile,
    key_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}
/// Exact nonce-free SIV key, erased on drop; component boundaries are caller data.
pub struct SivSettings {
    profile: SivProfile,
    key: Zeroizing<Vec<u8>>,
}
impl std::fmt::Debug for SivSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SivSettings { [REDACTED] }")
    }
}
impl SivSettings {
    /// Require a 32/64-byte total SIV-CMAC key; nonce fields are forbidden.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        document_bound(document)?;
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        Ok(Self {
            profile: p.profile,
            key: decode(&p.key_hex, p.profile.key_size())?,
        })
    }
}
fn preflight_components(data: &[u8], components: &[Vec<u8>]) -> Result<(), OperationError> {
    if components.len() > MAX_AAD_COMPONENTS {
        return Err(limit("SIV accepts at most 126 associated-data components"));
    }
    let size = components
        .iter()
        .try_fold(0usize, |sum, c| sum.checked_add(c.len()))
        .ok_or_else(|| limit("SIV associated-data length overflows"))?;
    preflight(data, size, MAX_INPUT)
}
/// Deterministic RFC5297 encryption with ordered AAD components and no nonce.
/// Empty component lists differ from a single empty component. Combined input
/// is at most16MiB and at most126 AAD components; the detached SIV is16bytes.
pub fn siv_encrypt(
    data: &[u8],
    components: &[Vec<u8>],
    s: &SivSettings,
) -> Result<Detached, OperationError> {
    preflight_components(data, components)?;
    let mut buffer = Zeroizing::new(data.to_vec());
    let tag = match s.profile {
        SivProfile::Aes128 => Aes128Siv::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .encrypt_inout_detached(components.iter(), buffer.as_mut_slice().into()),
        SivProfile::Aes256 => Aes256Siv::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .encrypt_inout_detached(components.iter(), buffer.as_mut_slice().into()),
    }
    .map_err(|_| authentication_failed())?;
    Ok(Detached {
        ciphertext: buffer.to_vec(),
        tag: tag.to_vec(),
    })
}
/// Verify the complete SIV and all component boundaries before plaintext release.
pub fn siv_decrypt(
    data: &[u8],
    components: &[Vec<u8>],
    tag: &[u8],
    s: &SivSettings,
) -> Result<Vec<u8>, OperationError> {
    preflight_components(data, components)?;
    let tag: &aes_siv::Tag = tag.try_into().map_err(|_| authentication_failed())?;
    let mut buffer = Zeroizing::new(data.to_vec());
    match s.profile {
        SivProfile::Aes128 => Aes128Siv::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .decrypt_inout_detached(components.iter(), buffer.as_mut_slice().into(), tag),
        SivProfile::Aes256 => Aes256Siv::new_from_slice(&s.key)
            .map_err(|_| invalid())?
            .decrypt_inout_detached(components.iter(), buffer.as_mut_slice().into(), tag),
    }
    .map_err(|_| authentication_failed())?;
    Ok(buffer.to_vec())
}
