//! Raw FIPS 197 AES blocks, without padding, authentication or a message format.
//! This low-level interoperability interface is not a data-encryption scheme.
mod catalog;
use crate::errors::{ErrorCode, OperationError};
use aes::{
    Aes128, Aes192, Aes256,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::Deserialize;
use zeroize::{Zeroize, Zeroizing};

pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub const MAX_PARAMETERS: usize = 1024;
pub const ENCRYPT: &str = "cipher.aes.encrypt_blocks";
pub const DECRYPT: &str = "cipher.aes.decrypt_blocks";

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Parameters {
    #[schemars(regex(pattern = "^(?:[0-9A-Fa-f]{32}|[0-9A-Fa-f]{48}|[0-9A-Fa-f]{64})$"))]
    key_hex: String,
}
impl Drop for Parameters {
    fn drop(&mut self) {
        self.key_hex.zeroize();
    }
}

/// Parse the exact typed transport, without echoing malformed secret JSON.
/// The decoded key and deserialized hexadecimal copy are wiped on drop. The
/// caller owns the source document; this cannot erase its copies or OS buffers.
pub fn parse_key(document: &str) -> Result<ParsedKey, OperationError> {
    let invalid = || {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "use only key_hex with exactly 32, 48 or 64 hexadecimal digits",
        )
        .parameter("key_hex")
    };
    if document.len() > MAX_PARAMETERS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "AES parameters exceed 1024 bytes",
        ));
    }
    let p: Parameters = serde_json::from_str(document).map_err(|_| invalid())?;
    if !matches!(p.key_hex.len(), 32 | 48 | 64) {
        return Err(invalid());
    }
    let mut key = Zeroizing::new(vec![0; p.key_hex.len() / 2]);
    hex::decode_to_slice(&p.key_hex, &mut key[..]).map_err(|_| invalid())?;
    Ok(ParsedKey(key))
}

/// Owned decoded AES transport key, wiped on drop and redacted in Debug.
pub struct ParsedKey(Zeroizing<Vec<u8>>);
impl AsRef<[u8]> for ParsedKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl std::fmt::Debug for ParsedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ParsedKey { [REDACTED] }")
    }
}

/// Encrypt independent 16-byte blocks using a 16/24/32-byte key.
///
/// Empty input is an identity after key validation. Inputs above 16 MiB or with
/// partial blocks fail before output allocation. Equal blocks produce equal
/// output; there is no nonce, padding, authentication, KDF or random generation.
/// RustCrypto's expanded keys are zeroized on drop. Borrowed keys/input and the
/// returned bytes belong to the caller and are not erased by this function.
pub fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    transform(data, key, false)
}

/// Decrypt raw AES blocks under the same rules as [`encrypt`].
/// Every valid-length key produces bytes; successful decryption does not verify
/// the key, plaintext integrity or authenticity. No padding is removed.
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    transform(data, key, true)
}

fn transform(data: &[u8], key: &[u8], decrypt: bool) -> Result<Vec<u8>, OperationError> {
    let id = if decrypt { DECRYPT } else { ENCRYPT };
    if !matches!(key.len(), 16 | 24 | 32) {
        return Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "AES key must contain 16, 24 or 32 bytes",
        )
        .operation(id)
        .parameter("key_hex"));
    }
    if data.len() > MAX_INPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "AES input exceeds 16 MiB").operation(id),
        );
    }
    if !data.len().is_multiple_of(16) {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "AES input must contain whole 16-byte blocks; padding is not supported",
        )
        .operation(id));
    }
    Ok(match key.len() {
        16 => blocks(
            &Aes128::new_from_slice(key).expect("validated key length"),
            data,
            decrypt,
        ),
        24 => blocks(
            &Aes192::new_from_slice(key).expect("validated key length"),
            data,
            decrypt,
        ),
        _ => blocks(
            &Aes256::new_from_slice(key).expect("validated key length"),
            data,
            decrypt,
        ),
    })
}

fn blocks<C>(cipher: &C, data: &[u8], decrypt: bool) -> Vec<u8>
where
    C: BlockCipherEncrypt<BlockSize = aes::cipher::consts::U16> + BlockCipherDecrypt,
{
    let mut output = data.to_vec();
    for bytes in output.as_chunks_mut::<16>().0 {
        let block: &mut aes::Block = bytes
            .as_mut_slice()
            .try_into()
            .expect("exact 16-byte chunk");
        if decrypt {
            cipher.decrypt_block(block);
        } else {
            cipher.encrypt_block(block);
        }
    }
    output
}
