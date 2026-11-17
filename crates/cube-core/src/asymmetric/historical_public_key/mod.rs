//! Executable educational reference schemes, never protection for live secrets.
//! Big integer operations are variable-time and their private heaps are not
//! guaranteed erased. Interfaces must retain this explicit security policy.
use crate::errors::{ErrorCode, OperationError};
use schemars::JsonSchema;
use serde::Deserialize;
pub const MAX_BITS: u64 = 4096;
pub const MAX_TOKEN_BYTES: usize = 1234;
pub const MAX_TOKENS: usize = 4096;
pub const MAX_KEY_BYTES: usize = 65536;
pub const MAX_PARAMETERS: usize = 1024;
pub const MAX_REPORT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_WORK: u64 = 4 * 1024 * 1024;
pub const MAX_ENTROPY: usize = 16 * 1024 * 1024;
fn default_work() -> u64 {
    MAX_WORK
}
fn default_memory() -> u64 {
    MAX_MEMORY_BYTES
}
macro_rules! parameters {
    ($name:literal) => {
        #[derive(Deserialize, JsonSchema)]
        enum Profile {
            #[serde(rename=$name)]
            Reference,
        }
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        pub(super) struct Parameters {
            profile: Profile,
            #[serde(default = "super::default_work")]
            #[schemars(range(min = 1, max = 4194304))]
            max_work: u64,
            #[serde(default = "super::default_memory")]
            #[schemars(range(min = 1, max = 67108864))]
            max_memory_bytes: u64,
        }
    };
}
macro_rules! redact {($($t:ty),*)=>{$(impl std::fmt::Debug for $t{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("HistoricalReferenceSettings { [REDACTED] }")}})*};}
pub mod benaloh;
mod catalog;
pub mod goldwasser_micali;
mod integer;
pub mod merkle_hellman;
pub mod rabin;
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "invalid educational reference profile or exact key",
    )
}
fn failed() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "historical public-key reference operation failed",
    )
}
fn limit() -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "historical reference call exceeds its declared bound",
    )
}
pub const RABIN_ENCRYPT: &str = "cipher.rabin.encrypt";
pub const RABIN_DECRYPT: &str = "cipher.rabin.decrypt";
pub const GOLDWASSER_MICALI_ENCRYPT: &str = "cipher.goldwasser_micali.encrypt";
pub const GOLDWASSER_MICALI_DECRYPT: &str = "cipher.goldwasser_micali.decrypt";
pub const BENALOH_ENCRYPT: &str = "cipher.benaloh.encrypt";
pub const BENALOH_DECRYPT: &str = "cipher.benaloh.decrypt";
pub const MERKLE_HELLMAN_ENCRYPT: &str = "cipher.merkle_hellman.encrypt";
pub const MERKLE_HELLMAN_DECRYPT: &str = "cipher.merkle_hellman.decrypt";
pub(crate) fn supports(op: &str) -> bool {
    matches!(
        op,
        RABIN_ENCRYPT
            | RABIN_DECRYPT
            | GOLDWASSER_MICALI_ENCRYPT
            | GOLDWASSER_MICALI_DECRYPT
            | BENALOH_ENCRYPT
            | BENALOH_DECRYPT
            | MERKLE_HELLMAN_ENCRYPT
            | MERKLE_HELLMAN_DECRYPT
    )
}
pub(crate) fn bounded_parameters(
    mut p: serde_json::Value,
    budget: crate::operations::execution::Budget,
) -> Result<serde_json::Value, OperationError> {
    let object = p.as_object_mut().ok_or_else(invalid)?;
    for (field, hard, host) in [
        ("max_work", MAX_WORK, budget.instructions),
        (
            "max_memory_bytes",
            MAX_MEMORY_BYTES,
            budget.memory_bytes as u64,
        ),
    ] {
        let supplied = match object.get(field) {
            None => hard,
            Some(v) => v
                .as_u64()
                .filter(|n| *n > 0 && *n <= hard)
                .ok_or_else(invalid)?,
        };
        let actual = supplied.min(host);
        if actual == 0 {
            return Err(limit());
        }
        object.insert(field.into(), serde_json::json!(actual));
    }
    Ok(p)
}
pub(crate) fn execute(
    op: &str,
    tokens: &[String],
    key: &[u8],
    document: &str,
) -> Result<serde_json::Value, OperationError> {
    Ok(match op {
        RABIN_ENCRYPT => {
            serde_json::json!({"tokens":rabin::encrypt(tokens,&rabin::EncryptionSettings::parse(document,key)?)?})
        }
        RABIN_DECRYPT => {
            serde_json::json!({"candidates":rabin::decrypt(tokens,&rabin::DecryptionSettings::parse(document,key)?)?})
        }
        GOLDWASSER_MICALI_ENCRYPT => {
            serde_json::json!({"tokens":goldwasser_micali::encrypt(tokens,&goldwasser_micali::EncryptionSettings::parse(document,key)?)?})
        }
        GOLDWASSER_MICALI_DECRYPT => {
            serde_json::json!({"tokens":goldwasser_micali::decrypt(tokens,&goldwasser_micali::DecryptionSettings::parse(document,key)?)?})
        }
        BENALOH_ENCRYPT => {
            serde_json::json!({"tokens":benaloh::encrypt(tokens,&benaloh::EncryptionSettings::parse(document,key)?)?})
        }
        BENALOH_DECRYPT => {
            serde_json::json!({"tokens":benaloh::decrypt(tokens,&benaloh::DecryptionSettings::parse(document,key)?)?})
        }
        MERKLE_HELLMAN_ENCRYPT => {
            serde_json::json!({"tokens":merkle_hellman::encrypt(tokens,&merkle_hellman::EncryptionSettings::parse(document,key)?)?})
        }
        MERKLE_HELLMAN_DECRYPT => {
            serde_json::json!({"tokens":merkle_hellman::decrypt(tokens,&merkle_hellman::DecryptionSettings::parse(document,key)?)?})
        }
        _ => return Err(invalid()),
    })
}
