//! Bounded binary agev1/X25519 decryption; authenticated completion before output.
mod catalog;
mod header;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use std::io::Read;
use zeroize::{Zeroize, Zeroizing};
pub const DECRYPT: &str = "cipher.age.decrypt";
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub const MAX_PLAINTEXT: usize = 16 * 1024 * 1024;
pub const MAX_HEADER: usize = 65536;
pub const MAX_STANZAS: usize = 8;
pub const MAX_PARAMETERS: usize = 2048;
#[derive(Clone, Copy, Deserialize, JsonSchema)]
enum Profile {
    #[serde(rename = "age_v1_x25519_binary")]
    X25519,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    profile: Profile,
    #[schemars(
        length(min = 74, max = 74),
        regex(pattern = "^AGE-SECRET-KEY-1[0-9A-Z]+$")
    )]
    identity: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.identity.zeroize();
    }
}
/// One caller-owned X25519 identity. Debug never formats its secret contents.
pub struct Settings {
    identity: age::x25519::Identity,
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AgeSettings { [REDACTED] }")
    }
}
impl Settings {
    /// Accept only the exact binary X25519 profile and one canonical uppercase
    /// Bech32 identity, with no identity file parsing, plugins or network lookup.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        if document.len() > MAX_PARAMETERS {
            return Err(limit());
        }
        let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
        let Profile::X25519 = p.profile;
        if p.identity.len() != 74
            || !p.identity.starts_with("AGE-SECRET-KEY-1")
            || p.identity.bytes().any(|b| b.is_ascii_lowercase())
        {
            return Err(invalid());
        }
        let identity = p.identity.parse().map_err(|_| invalid())?;
        Ok(Self { identity })
    }
}
/// Decrypt a complete bounded binary agev1 file using one X25519 identity.
/// Header/stanza caps and canonical encoding are checked before the maintained
/// parser allocates or performs any recipient cryptography. Plaintext stays in
/// a wiped quarantine until every chunk tag, final marker and EOF are verified.
/// Scrypt and armor are refused; canonical unknown stanzas are bounded and ignored.
/// No SSH, PQ or plugin identity is invoked.
pub fn decrypt(input: &[u8], settings: &Settings) -> Result<Vec<u8>, OperationError> {
    if input.len() > MAX_INPUT {
        return Err(limit());
    }
    header::check(input)?;
    let decryptor = age::Decryptor::new_buffered(input).map_err(header_error)?;
    let reader = decryptor
        .decrypt(std::iter::once(&settings.identity as &dyn age::Identity))
        .map_err(|_| authentication())?;
    let mut plaintext = Zeroizing::new(Vec::new());
    reader
        .take(MAX_PLAINTEXT as u64 + 1)
        .read_to_end(&mut plaintext)
        .map_err(|_| authentication())?;
    if plaintext.len() > MAX_PLAINTEXT {
        return Err(limit());
    }
    Ok(std::mem::take(&mut *plaintext))
}
fn header_error(error: age::DecryptError) -> OperationError {
    match error {
        age::DecryptError::UnknownFormat => unsupported(),
        age::DecryptError::InvalidHeader | age::DecryptError::Io(_) => malformed(),
        _ => authentication(),
    }
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid exact age profile or private X25519 identity",
    )
}
fn unsupported() -> OperationError {
    OperationError::new(
        ErrorCode::UnsupportedFormat,
        "unsupported age profile or stanza",
    )
}
fn malformed() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "malformed canonical age v1/X25519 header",
    )
}
fn authentication() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "age authentication or identity match failed",
    )
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "age ciphertext, plaintext, header, stanza or settings limit exceeded",
    )
}

impl Profile {
    const NAMES: &[&str] = &["age_v1_x25519_binary"];
}
