//! Exact numeral-string FPE profiles with integer-only domain arithmetic.
mod catalog;
mod rounds;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};
pub const ENCRYPT: &str = "cipher.fpe.encrypt";
pub const DECRYPT: &str = "cipher.fpe.decrypt";
pub const MAX_DIGITS: usize = 4096;
pub const MAX_TWEAK: usize = 65536;
pub const MAX_PARAMETERS: usize = 132096;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "ff1_sp800_38g_integer")]
    Ff1,
    #[serde(rename = "ff3_sp800_38g_historical")]
    Ff3,
    #[serde(rename = "ff3_1_2019_draft_historical")]
    Ff31,
}
impl Profile {
    pub const NAMES: &[&str] = &[
        "ff1_sp800_38g_integer",
        "ff3_sp800_38g_historical",
        "ff3_1_2019_draft_historical",
    ];
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(range(min = 2, max = 65536))]
    radix: u32,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{32}|[0-9A-Fa-f]{48}|[0-9A-Fa-f]{64})$"))]
    key_hex: String,
    #[schemars(length(max = 131072), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    tweak_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
        self.tweak_hex.zeroize();
    }
}
/// Exact validated profile, radix, AES key and tweak; Debug redacts all fields.
pub struct Settings {
    profile: Profile,
    radix: u32,
    key: Zeroizing<Vec<u8>>,
    tweak: Zeroizing<Vec<u8>>,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FpeSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Parse explicit settings, with no inferred alphabet, key, tweak or profile.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        if !(2..=65536).contains(&p.radix)
            || !matches!(p.key_hex.len(), 32 | 48 | 64)
            || p.tweak_hex.len() > MAX_TWEAK * 2
            || !p.tweak_hex.len().is_multiple_of(2)
        {
            return Err(invalid());
        }
        let key = Zeroizing::new(hex::decode(&p.key_hex).map_err(|_| invalid())?);
        let tweak = Zeroizing::new(hex::decode(&p.tweak_hex).map_err(|_| invalid())?);
        if (p.profile == Profile::Ff3 && tweak.len() != 8)
            || (p.profile == Profile::Ff31 && tweak.len() != 7)
        {
            return Err(invalid());
        }
        Ok(Self {
            profile: p.profile,
            radix: p.radix,
            key,
            tweak,
        })
    }
}
/// Encrypt exact digits without changing their length, radix or leading zeros.
/// FF3/FF3-1 are explicitly historical interoperability profiles, not modern
/// recommended encryption. No profile authenticates the data or tweak.
pub fn encrypt(input: &[u16], settings: &Settings) -> Result<Vec<u16>, OperationError> {
    transform(input, settings, false)
}
/// Invert the selected exact FPE profile. Success does not verify a key/tweak.
pub fn decrypt(input: &[u16], settings: &Settings) -> Result<Vec<u16>, OperationError> {
    transform(input, settings, true)
}
fn transform(input: &[u16], s: &Settings, inverse: bool) -> Result<Vec<u16>, OperationError> {
    if input.len() > MAX_DIGITS {
        return Err(limit());
    }
    if input.len() < 2 || input.iter().any(|&d| u32::from(d) >= s.radix) {
        return Err(invalid());
    }
    let radix = num_bigint::BigUint::from(s.radix);
    if radix.pow(input.len() as u32) < num_bigint::BigUint::from(1_000_000u32) {
        return Err(invalid());
    }
    if s.profile != Profile::Ff1
        && radix.pow(input.len().div_ceil(2) as u32) > (num_bigint::BigUint::from(1u8) << 96usize)
    {
        return Err(invalid());
    }
    rounds::transform(input, s, inverse)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid exact FPE profile, radix, digits, domain, key or tweak",
    )
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "FPE input or settings exceed the documented bound",
    )
}
