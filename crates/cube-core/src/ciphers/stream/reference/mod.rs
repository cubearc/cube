//! Explicit historical stream profiles. Raw output has no authentication or
//! constant-time guarantee; RC4 is broken. No state is retained between calls.
mod catalog;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use chacha20::cipher::{KeyInit, KeyIvInit, StreamCipher};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

pub const MAX_INPUT: usize = 64 * 1024;
pub const MAX_PARAMETERS: usize = 4096;
pub const MAX_DISCARD: usize = 65536;
pub const ENCRYPT: &str = "cipher.stream.encrypt";
pub const DECRYPT: &str = "cipher.stream.decrypt";

macro_rules! profiles {
    ($(($variant:ident, $name:literal, $min:literal, $max:literal, $iv:literal)),+ $(,)?) => {
        /// Exact named construction and initialization; there is no default.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
        pub enum Profile { $(#[serde(rename=$name)] $variant,)+ }
        impl Profile {
            pub const ALL: &[Self] = &[$(Self::$variant,)+];
            pub const NAMES: &[&str] = &[$($name,)+];
            fn key_range(self) -> (usize, usize) { match self { $(Self::$variant => ($min,$max),)+ } }
            fn nonce_bytes(self) -> usize { match self { $(Self::$variant => $iv,)+ } }
        }
    }
}
profiles! {
    (Rc4, "rc4_rfc6229_v1", 1, 256, 0),
    (Rc4Drop, "rc4_drop_v1", 1, 256, 0),
    (Hc256, "hc256_wu2004_v1", 32, 32, 32),
    (Rabbit, "rabbit_rfc4503_iv_le_v1", 16, 16, 8),
    (RabbitKeyOnly, "rabbit_rfc4503_key_only_le_v1", 16, 16, 0),
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{2}){1,256}$"))]
    key_hex: String,
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{2}){0,256}$"))]
    nonce_hex: String,
    #[serde(default, deserialize_with = "discard")]
    #[schemars(range(min = 0, max = 65536))]
    discard_bytes: Option<usize>,
}
fn discard<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<usize>, D::Error> {
    usize::deserialize(d).map(Some)
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
        self.nonce_hex.zeroize();
    }
}

/// Validated settings. Debug redacts private material; owned key/nonce copies
/// and supported backend schedules are wiped on drop. Parser/caller copies are
/// outside this guarantee.
pub struct Settings {
    profile: Profile,
    key: Zeroizing<Vec<u8>>,
    nonce: Zeroizing<Vec<u8>>,
    discard_bytes: usize,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("profile", &self.profile)
            .field("key", &"[REDACTED]")
            .field("nonce", &"[REDACTED]")
            .finish()
    }
}
impl Settings {
    /// Parse at most 4096 bytes of private JSON. Duplicate/unknown fields, null,
    /// wrong key/nonce widths and unrelated drop settings are rejected without
    /// echoing parser excerpts. Nonce is explicit, including an empty nonce for
    /// constructions with no IV. RC4-drop requires its explicit discard count.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "stream parameters exceed 4096 bytes",
            ));
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid("parameters"))?;
        let key = decode_hex(&p.key_hex, "key_hex")?;
        let nonce = decode_hex(&p.nonce_hex, "nonce_hex")?;
        let (min, max) = p.profile.key_range();
        if !(min..=max).contains(&key.len()) {
            return Err(invalid("key_hex"));
        }
        if nonce.len() != p.profile.nonce_bytes() {
            return Err(invalid("nonce_hex"));
        }
        let discard_bytes = match (p.profile, p.discard_bytes) {
            (Profile::Rc4Drop, Some(n)) if n <= MAX_DISCARD => n,
            (Profile::Rc4Drop, _) => return Err(invalid("discard_bytes")),
            (_, None) => 0,
            _ => return Err(invalid("discard_bytes")),
        };
        Ok(Self {
            profile: p.profile,
            key,
            nonce,
            discard_bytes,
        })
    }
}
fn decode_hex(value: &str, name: &'static str) -> Result<Zeroizing<Vec<u8>>, OperationError> {
    if value.len() > 512 || !value.len().is_multiple_of(2) {
        return Err(invalid(name));
    }
    let mut bytes = Zeroizing::new(vec![0; value.len() / 2]);
    hex::decode_to_slice(value, &mut bytes[..]).map_err(|_| invalid(name))?;
    Ok(bytes)
}
fn invalid(parameter: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact profile, key, nonce and profile-specific settings",
    )
    .parameter(parameter)
}
/// Encrypt at most 64 KiB with exact profile settings. No nonce generation,
/// padding, KDF, authentication or persistent key/position tracking occurs.
pub fn encrypt(input: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(input, settings)
}
/// Decrypt the same exact raw profile. Successful byte production does not
/// establish authenticity or correct key selection.
pub fn decrypt(input: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(input, settings)
}
fn transform(input: &[u8], s: &Settings) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_INPUT {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "stream input exceeds 64 KiB",
        ));
    }
    let mut output = input.to_vec();
    match s.profile {
        Profile::Rc4 | Profile::Rc4Drop => {
            let mut c = rc4::Rc4::new_from_slice(&s.key).expect("validated key");
            let mut scratch = Zeroizing::new([0u8; 64]);
            let mut remaining = s.discard_bytes;
            while remaining > 0 {
                let n = remaining.min(scratch.len());
                scratch.fill(0);
                c.apply_keystream(&mut scratch[..n]);
                remaining -= n;
            }
            c.apply_keystream(&mut output);
        }
        Profile::Hc256 => hc_256::Hc256::new_from_slices(&s.key, &s.nonce)
            .expect("validated key and IV")
            .apply_keystream(&mut output),
        Profile::Rabbit => rabbit::Rabbit::new_from_slices(&s.key, &s.nonce)
            .expect("validated key and IV")
            .apply_keystream(&mut output),
        Profile::RabbitKeyOnly => rabbit::RabbitKeyOnly::new_from_slice(&s.key)
            .expect("validated key")
            .apply_keystream(&mut output),
    }
    Ok(output)
}
