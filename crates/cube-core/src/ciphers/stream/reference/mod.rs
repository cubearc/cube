//! Explicit historical stream profiles. Raw output has no authentication or
//! constant-time guarantee; RC4 is broken. No state is retained between calls.
mod catalog;
mod fcsr;
mod grain_isaac;
mod hc128;
mod panama;
mod permutation;
mod seal;
mod snow;
mod sober;
mod sosemanuk;
mod strumok;
mod trivium;
mod word_field;
mod zuc;
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
    ($(($variant:ident, $name:literal, $min:literal, $max:literal, $iv_min:literal, $iv_max:literal)),+ $(,)?) => {
        /// Exact named construction and initialization; there is no default.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
        pub enum Profile { $(#[serde(rename=$name)] $variant,)+ }
        impl Profile {
            pub const ALL: &[Self] = &[$(Self::$variant,)+];
            pub const NAMES: &[&str] = &[$($name,)+];
            fn key_range(self) -> (usize, usize) { match self { $(Self::$variant => ($min,$max),)+ } }
            fn nonce_range(self) -> (usize,usize) { match self { $(Self::$variant => ($iv_min,$iv_max),)+ } }
        }
    }
}
profiles! {
    (Rc4, "rc4_rfc6229_v1", 1, 256, 0, 0),
    (Rc4Drop, "rc4_drop_v1", 1, 256, 0, 0),
    (Hc256, "hc256_wu2004_v1", 32, 32, 32, 32),
    (Rabbit, "rabbit_rfc4503_iv_le_v1", 16, 16, 8, 8),
    (RabbitKeyOnly, "rabbit_rfc4503_key_only_le_v1", 16, 16, 0, 0),
    (Rc4a, "rc4a_fse2004_rc4_k2_same_length_v1", 1, 256, 0, 0),
    (Vmpc, "vmpc_zoltak2004_iv_v1", 16, 64, 16, 64),
    (VmpcKsa3, "vmpc_ksa3_iv_v1", 16, 64, 16, 64),
    (Spritz, "spritz_rs2014_add_key_only_v1", 1, 256, 0, 0),
    (SpritzIv, "spritz_rs2014_add_iv_v1", 1, 256, 0, 256),
    (Hc128, "hc128_wu2008_le_v1", 16, 16, 16, 16),
    (Trivium, "trivium_estream80_v1", 10, 10, 10, 10),
    (Sosemanuk, "sosemanuk_estream_le_v1", 16, 32, 16, 16),
    (GrainV1, "grain_v1_estream_le_v1", 10, 10, 8, 8),
    (Isaac32, "isaac32_bc_le_seed_be_v1", 32, 32, 0, 0),
    (Strumok256, "strumok256_dstu8845_le_v1", 32, 32, 32, 32),
    (Strumok512, "strumok512_dstu8845_le_v1", 64, 64, 32, 32),
    (PanamaBeKeyOnly, "panama_1998_be_key_only_v1", 32, 32, 0, 0),
    (PanamaLeKeyOnly, "panama_1998_le_key_only_v1", 32, 32, 0, 0),
    (PanamaBeIv, "panama_1998_be_iv256_v1", 32, 32, 32, 32),
    (PanamaLeIv, "panama_1998_le_iv256_v1", 32, 32, 32, 32),
    (Snow20_128, "snow20_128_be_v1", 16, 16, 16, 16),
    (Snow20_256, "snow20_256_be_v1", 32, 32, 16, 16),
    (Snow3g, "snow3g_sage11_words_be_v1", 16, 16, 16, 16),
    (Zuc128, "zuc128_3gpp16_be_v1", 16, 16, 16, 16),
    (Sober128, "sober128_rose2003_le_iv32_v1", 16, 16, 4, 4),
    (SealBe, "seal3_be_l32768_v1", 20, 20, 4, 4),
    (SealLe, "seal3_le_l32768_v1", 20, 20, 4, 4),
    (Fcsr, "ffcsr_h_2005_iv80_be_v1", 10, 10, 10, 10),
    (Lex, "lex_v2_estream128_le_iv_words_v1", 16, 16, 16, 16),
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
        let (iv_min, iv_max) = p.profile.nonce_range();
        if !(iv_min..=iv_max).contains(&nonce.len()) {
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
    transform(input, settings, false)
}
/// Decrypt the same exact raw profile. Successful byte production does not
/// establish authenticity or correct key selection.
pub fn decrypt(input: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    transform(input, settings, true)
}
fn transform(input: &[u8], s: &Settings, decrypting: bool) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_INPUT {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "stream input exceeds 64 KiB",
        ));
    }
    let mut output = input.to_vec();
    match s.profile {
        Profile::SealBe | Profile::SealLe => {
            seal::apply(&mut output, &s.key, &s.nonce, s.profile == Profile::SealLe).map_err(
                |_| {
                    OperationError::new(ErrorCode::ResourceLimit, "SEAL position counter exhausted")
                },
            )?
        }
        Profile::Fcsr => fcsr::apply(&mut output, &s.key, &s.nonce),
        Profile::Lex => {
            crate::ciphers::block_profiles::rijndael::lex128(&mut output, &s.key, &s.nonce)
        }
        Profile::Snow20_128 | Profile::Snow20_256 | Profile::Snow3g => {
            snow::apply(&mut output, &s.key, &s.nonce, s.profile == Profile::Snow3g)
        }
        Profile::Zuc128 => zuc::apply(&mut output, &s.key, &s.nonce),
        Profile::Sober128 => sober::apply(&mut output, &s.key, &s.nonce).map_err(|_| {
            OperationError::new(
                ErrorCode::ResourceLimit,
                "SOBER-128 initialization work limit exceeded",
            )
        })?,
        Profile::Sosemanuk => sosemanuk::apply(&mut output, &s.key, &s.nonce),
        Profile::GrainV1 => grain_isaac::grain(&mut output, &s.key, &s.nonce),
        Profile::Isaac32 => grain_isaac::isaac(&mut output, &s.key),
        Profile::Strumok256 | Profile::Strumok512 => strumok::apply(&mut output, &s.key, &s.nonce),
        Profile::PanamaBeKeyOnly
        | Profile::PanamaLeKeyOnly
        | Profile::PanamaBeIv
        | Profile::PanamaLeIv => panama::apply(
            &mut output,
            &s.key,
            &s.nonce,
            matches!(s.profile, Profile::PanamaLeKeyOnly | Profile::PanamaLeIv),
        ),
        Profile::Rc4a => permutation::rc4a(&mut output, &s.key),
        Profile::Vmpc | Profile::VmpcKsa3 => permutation::vmpc(
            &mut output,
            &s.key,
            &s.nonce,
            s.profile == Profile::VmpcKsa3,
        ),
        Profile::Spritz | Profile::SpritzIv => permutation::spritz(
            &mut output,
            &s.key,
            (s.profile == Profile::SpritzIv).then_some(&s.nonce),
            decrypting,
        ),
        Profile::Hc128 => hc128::apply(&mut output, &s.key, &s.nonce),
        Profile::Trivium => trivium::apply(&mut output, &s.key, &s.nonce),
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
