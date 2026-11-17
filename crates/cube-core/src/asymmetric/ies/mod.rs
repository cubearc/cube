//! Complete ephemeral ECIES/DHIES suites interoperating with BC1.83 IESEngine.
mod catalog;
#[cfg(test)]
mod tests;
use super::{dh, ecdh, entropy::SystemRng};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use hmac::{Hmac, KeyInit, Mac};
use rand_core::TryCryptoRng;
use schemars::JsonSchema;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;
pub const ENCRYPT: &str = "cipher.ies.encrypt";
pub const DECRYPT: &str = "cipher.ies.decrypt";
pub const MAX_BODY: usize = 16 * 1024 * 1024;
pub const MAX_CIPHERTEXT: usize = MAX_BODY + 256 + 32;
pub const MAX_KEY: usize = 256;
pub const MAX_CONTEXT: usize = 256;
pub const MAX_PARAMETERS: usize = 2048;
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "ecies_p256_bc183_sha256_kdf2_hmac256")]
    Ecies,
    #[serde(rename = "dhies_ffdhe2048_bc183_sha256_kdf2_hmac256")]
    Dhies,
}
impl Profile {
    pub const NAMES: &[&str] = &[
        "ecies_p256_bc183_sha256_kdf2_hmac256",
        "dhies_ffdhe2048_bc183_sha256_kdf2_hmac256",
    ];
    fn public_width(self) -> usize {
        match self {
            Self::Ecies => 65,
            Self::Dhies => 256,
        }
    }
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[serde(default)]
    #[schemars(length(max = 512), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    derivation_hex: String,
    #[serde(default)]
    #[schemars(length(max = 512), regex(pattern = "^(?:[0-9A-Fa-f]{2})*$"))]
    encoding_hex: String,
}
struct Context {
    profile: Profile,
    derivation: Vec<u8>,
    encoding: Vec<u8>,
}
impl Context {
    fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS || key.len() > MAX_KEY {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        let decode = |text: &str| {
            if text.len() > MAX_CONTEXT * 2 {
                return Err(limit());
            }
            hex::decode(text).map_err(|_| invalid())
        };
        Ok(Self {
            profile: p.profile,
            derivation: decode(&p.derivation_hex)?,
            encoding: decode(&p.encoding_hex)?,
        })
    }
}
enum Agreement {
    Ecies(ecdh::Settings),
    Dhies(dh::Settings),
}
impl Agreement {
    fn parse(profile: Profile, key: &[u8]) -> Result<Self, OperationError> {
        match profile {
            Profile::Ecies => {
                ecdh::Settings::from_private_key(ecdh::Profile::P256, key).map(Self::Ecies)
            }
            Profile::Dhies => {
                dh::Settings::from_private_key(dh::Profile::Ffdhe2048, key).map(Self::Dhies)
            }
        }
    }
    fn public_key(&self) -> Vec<u8> {
        match self {
            Self::Ecies(s) => s.public_key_uncompressed(),
            Self::Dhies(s) => s.public_key(),
        }
    }
    fn agree(&self, peer: &[u8]) -> Result<Zeroizing<Vec<u8>>, OperationError> {
        let bytes = match self {
            Self::Ecies(s) => ecdh::agree(peer, s),
            Self::Dhies(s) => dh::agree(peer, s),
        }
        .map_err(|_| failed())?;
        Ok(Zeroizing::new(bytes))
    }
}
/// Validated recipient public key, complete suite and bounded KDF/MAC context.
pub struct EncryptionSettings {
    context: Context,
    key: Vec<u8>,
}
/// Validated raw private key and the same exact suite and public context.
pub struct DecryptionSettings {
    context: Context,
    key: Agreement,
}
macro_rules! redact {($($t:ty),*)=>{$(impl std::fmt::Debug for $t{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("IesSettings { [REDACTED] }")}})*};}
redact!(EncryptionSettings, DecryptionSettings);
fn public_key(profile: Profile, key: &[u8]) -> Result<(), OperationError> {
    if key.len() != profile.public_width() {
        return Err(invalid());
    }
    match profile {
        Profile::Ecies => {
            if key[0] != 4 {
                return Err(invalid());
            }
            p256::PublicKey::from_sec1_bytes(key).map_err(|_| invalid())?;
        }
        Profile::Dhies => dh::validate_peer(key, dh::Profile::Ffdhe2048).map_err(|_| invalid())?,
    }
    Ok(())
}
impl EncryptionSettings {
    /// Require a65-byte uncompressed P-256 SEC1 or256-byte subgroup-validated
    /// ffdhe2048 public key, plus an explicit suite and optional hexadecimal context.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let context = Context::parse(document, key)?;
        public_key(context.profile, key)?;
        Ok(Self {
            context,
            key: key.to_vec(),
        })
    }
}
impl DecryptionSettings {
    /// Require exactly32 private bytes for the selected suite. P-256 requires
    /// 1..n-1; the ffdhe2048 short-exponent profile refuses zero and one.
    pub fn parse(document: &str, key: &[u8]) -> Result<Self, OperationError> {
        let context = Context::parse(document, key)?;
        let key = Agreement::parse(context.profile, key).map_err(|_| invalid())?;
        Ok(Self { context, key })
    }
}
fn ephemeral(profile: Profile, rng: &mut impl TryCryptoRng) -> Result<Agreement, OperationError> {
    for _ in 0..8 {
        let mut key = Zeroizing::new([0; 32]);
        rng.try_fill_bytes(&mut key[..]).map_err(|_| failed())?;
        if let Ok(key) = Agreement::parse(profile, &key[..]) {
            return Ok(key);
        }
    }
    Err(failed())
}
/// Encrypt using a fresh ephemeral agreement key, KDF2-SHA256 streaming XOR and
/// HMAC-SHA256. Output is exactly V||C||T, matching the selected BC1.83 suite.
/// There is no static sender mode, universal envelope, IV or sender identity.
pub fn encrypt(input: &[u8], s: &EncryptionSettings) -> Result<Vec<u8>, OperationError> {
    encrypt_with_rng(input, s, &mut SystemRng::bounded(256))
}
fn encrypt_with_rng(
    input: &[u8],
    s: &EncryptionSettings,
    rng: &mut impl TryCryptoRng,
) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_BODY {
        return Err(limit());
    }
    let ephemeral = ephemeral(s.context.profile, rng)?;
    let public = ephemeral.public_key();
    let shared = ephemeral.agree(&s.key)?;
    let kdf = Kdf {
        public: &public,
        shared: &shared,
        derivation: &s.context.derivation,
    };
    let mac_key = kdf.block(1);
    let mut output = public.clone();
    output.resize(public.len() + input.len(), 0);
    kdf.xor(input, &mut output[public.len()..]);
    let tag = mac(&mac_key[..], &output[public.len()..], &s.context.encoding)
        .finalize()
        .into_bytes();
    output.extend_from_slice(&tag);
    Ok(output)
}
/// Authenticate V||C||T with the exact suite and contexts before computing or
/// returning plaintext. Invalid ephemeral keys, contexts, body or tag uniformly
/// fail with no output. HMAC comparison uses the maintained constant-time API.
pub fn decrypt(input: &[u8], s: &DecryptionSettings) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_CIPHERTEXT {
        return Err(limit());
    }
    let width = s.context.profile.public_width();
    if input.len() < width + 32 {
        return Err(failed());
    }
    let (public, tail) = input.split_at(width);
    let (body, tag) = tail.split_at(tail.len() - 32);
    if body.len() > MAX_BODY {
        return Err(limit());
    }
    public_key(s.context.profile, public).map_err(|_| failed())?;
    let shared = s.key.agree(public)?;
    let kdf = Kdf {
        public,
        shared: &shared,
        derivation: &s.context.derivation,
    };
    let mac_key = kdf.block(1);
    mac(&mac_key[..], body, &s.context.encoding)
        .verify_slice(tag)
        .map_err(|_| failed())?;
    let mut output = Zeroizing::new(vec![0; body.len()]);
    kdf.xor(body, &mut output);
    Ok(std::mem::take(&mut *output))
}
struct Kdf<'a> {
    public: &'a [u8],
    shared: &'a [u8],
    derivation: &'a [u8],
}
impl Kdf<'_> {
    fn block(&self, counter: u32) -> Zeroizing<[u8; 32]> {
        let mut hash = Sha256::new();
        hash.update(self.public);
        hash.update(self.shared);
        hash.update(counter.to_be_bytes());
        hash.update(self.derivation);
        Zeroizing::new(hash.finalize().into())
    }
    fn xor(&self, input: &[u8], output: &mut [u8]) {
        for (index, (source, dest)) in input.chunks(32).zip(output.chunks_mut(32)).enumerate() {
            let key = self.block(index as u32 + 2);
            for (i, byte) in source.iter().enumerate() {
                dest[i] = byte ^ key[i];
            }
        }
    }
}
fn mac(key: &[u8], body: &[u8], encoding: &[u8]) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("fixed32-byte HMAC key");
    mac.update(body);
    mac.update(encoding);
    mac.update(&((encoding.len() as u64) * 8).to_be_bytes());
    mac
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid exact IES suite, key encoding or context",
    )
}
fn failed() -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "IES operation failed")
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "IES input exceeds its declared bounds",
    )
}
