//! Bounded RFC7919 prime-order finite-field key agreement, without a KDF.
mod catalog;
mod groups;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use crypto_bigint::{
    BoxedUint, Odd,
    modular::{BoxedMontyForm, BoxedMontyParams},
};
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::Zeroizing;
pub const AGREE: &str = "crypto.dh.agree";
pub const MAX_PARAMETERS: usize = 256;
pub const MAX_PUBLIC_KEY: usize = 1024;
pub const MAX_PRIVATE_KEY: usize = 64;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "ffdhe2048_rfc7919_raw")]
    Ffdhe2048,
    #[serde(rename = "ffdhe3072_rfc7919_raw")]
    Ffdhe3072,
    #[serde(rename = "ffdhe4096_rfc7919_raw")]
    Ffdhe4096,
    #[serde(rename = "ffdhe6144_rfc7919_raw")]
    Ffdhe6144,
    #[serde(rename = "ffdhe8192_rfc7919_raw")]
    Ffdhe8192,
}
impl Profile {
    pub const NAMES: &[&str] = &[
        "ffdhe2048_rfc7919_raw",
        "ffdhe3072_rfc7919_raw",
        "ffdhe4096_rfc7919_raw",
        "ffdhe6144_rfc7919_raw",
        "ffdhe8192_rfc7919_raw",
    ];
    fn sizes(self) -> (u32, usize) {
        match self {
            Self::Ffdhe2048 => (2048, 32),
            Self::Ffdhe3072 => (3072, 40),
            Self::Ffdhe4096 => (4096, 48),
            Self::Ffdhe6144 => (6144, 56),
            Self::Ffdhe8192 => (8192, 64),
        }
    }
    fn modulus(self) -> BoxedUint {
        let text = match self {
            Self::Ffdhe2048 => groups::P2048,
            Self::Ffdhe3072 => groups::P3072,
            Self::Ffdhe4096 => groups::P4096,
            Self::Ffdhe6144 => groups::P6144,
            Self::Ffdhe8192 => groups::P8192,
        };
        BoxedUint::from_be_hex(text, self.sizes().0).expect("reviewed RFC7919 modulus")
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
}
/// A named public group and a private fixed-width big-endian exponent.
/// Diagnostics redact the private exponent; owned private arithmetic is wiped.
pub struct Settings {
    profile: Profile,
    secret: Zeroizing<BoxedUint>,
    params: BoxedMontyParams,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DhSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Parse a named RFC7919 profile and an exact32/40/48/56/64-byte private
    /// exponent respectively. Zero and one are refused. The caller must supply
    /// a fresh uniformly random secret; this interface does not generate keys.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS || key.len() > MAX_PRIVATE_KEY {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        Self::from_private_key(p.profile, key)
    }
    /// Validate the exact private exponent encoding for a typed named profile.
    /// Key generation, authentication and KDFs remain outside raw agreement.
    pub fn from_private_key(profile: Profile, key: &[u8]) -> Result<Self, OperationError> {
        if key.len() > MAX_PRIVATE_KEY {
            return Err(limit());
        }
        let (bits, length) = profile.sizes();
        if key.len() != length {
            return Err(invalid());
        }
        let secret = Zeroizing::new(
            BoxedUint::from_be_slice(key, (length * 8) as u32).map_err(|_| invalid())?,
        );
        if *secret <= BoxedUint::one_with_precision((length * 8) as u32) {
            return Err(invalid());
        }
        let modulus = profile.modulus();
        debug_assert_eq!(modulus.bits_precision(), bits);
        Ok(Self {
            profile,
            secret,
            params: BoxedMontyParams::new_vartime(Odd::new(modulus).expect("odd safe prime")),
        })
    }
    /// Return g^x as a fixed-width big-endian public value, where g=2.
    pub fn public_key(&self) -> Vec<u8> {
        let mut two = vec![0; self.profile.sizes().0 as usize / 8];
        *two.last_mut().expect("named group width") = 2;
        let base = BoxedMontyForm::new(
            BoxedUint::from_be_slice(&two, self.params.bits_precision()).expect("fixed width"),
            &self.params,
        );
        Zeroizing::new(base.pow(&self.secret))
            .retrieve()
            .to_be_bytes()
            .into_vec()
    }
}
/// Validate 2 <= peer <= p-2 and membership in the prime-order q=(p-1)/2
/// subgroup, then compute one shared secret padded to the named modulus width.
/// Secret exponentiation uses crypto-bigint's fixed-window constant-time path
/// at a public per-profile exponent width. No authentication, RNG or KDF occurs.
pub fn agree(peer: &[u8], s: &Settings) -> Result<Vec<u8>, OperationError> {
    let base = peer_form(peer, &s.params)?;
    let shared = Zeroizing::new(base.pow(&s.secret));
    Ok(Zeroizing::new(shared.retrieve()).to_be_bytes().into_vec())
}
/// Check exact public encoding, range and prime-order subgroup membership
/// without a private operation. Only the five typed named groups are accepted.
pub fn validate_peer(peer: &[u8], profile: Profile) -> Result<(), OperationError> {
    let params =
        BoxedMontyParams::new_vartime(Odd::new(profile.modulus()).expect("odd safe prime"));
    peer_form(peer, &params).map(|_| ())
}
fn peer_form(peer: &[u8], params: &BoxedMontyParams) -> Result<BoxedMontyForm, OperationError> {
    if peer.len() > MAX_PUBLIC_KEY {
        return Err(limit());
    }
    let bits = params.bits_precision();
    if peer.len() != bits as usize / 8 {
        return Err(failed());
    }
    let public = BoxedUint::from_be_slice(peer, bits).map_err(|_| failed())?;
    let one = BoxedUint::one_with_precision(bits);
    let p_minus_one = params.modulus().as_ref().wrapping_sub(&one);
    if public <= one || public >= p_minus_one {
        return Err(failed());
    }
    let base = BoxedMontyForm::new(public, params);
    let order = p_minus_one.unbounded_shr_vartime(1);
    if base.pow(&order).retrieve() != one {
        return Err(failed());
    }
    Ok(base)
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use a named RFC7919 profile and its exact-width private key",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "key agreement failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "DH input exceeds its declared bounds",
    )
}
