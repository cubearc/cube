//! Bounded standard digests reused by interoperable protocol formats.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::Descriptor,
};
use ripemd::Digest;
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub(crate) fn ripemd160_bytes(data: &[u8]) -> [u8; 20] {
    ripemd::Ripemd160::digest(data).into()
}
/// Exact RIPEMD-160 bytes rendered as 40 lowercase hex digits; input <=16 MiB.
/// Format-compatibility digest, not password hashing or message authentication.
pub fn ripemd160(data: &[u8]) -> Result<String, OperationError> {
    if data.len() > MAX_INPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "digest input exceeds 16 MiB")
                .operation("hash.ripemd160"),
        );
    }
    Ok(crate::encoding::hex_encode(&ripemd160_bytes(data)))
}
pub(crate) fn keccak256_bytes(data: &[u8]) -> [u8; 32] {
    sha3::Keccak256::digest(data).into()
}
/// Ethereum Keccak-256 as 64 lowercase hex digits; distinct from SHA3-256.
/// Exact input bytes, <=16 MiB; no normalization or protocol-specific prefixes.
pub fn keccak256(data: &[u8]) -> Result<String, OperationError> {
    if data.len() > MAX_INPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "digest input exceeds 16 MiB")
                .operation("hash.keccak256"),
        );
    }
    Ok(crate::encoding::hex_encode(&keccak256_bytes(data)))
}
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "hash.keccak256",
        category: "hashing",
        description: "Ethereum Keccak-256 (not standardized SHA3-256), exact bytes as 64 lowercase hex digits; <=16 MiB. No protocol-specific prefixes or normalization.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash keccak256",
        example: "cube hash keccak256 --text abc",
    },
    Descriptor {
        id: "hash.ripemd160",
        category: "hashing",
        description: "RIPEMD-160 exact-byte digest as 40 lowercase hex digits for protocol compatibility; <=16 MiB input, not password hashing/authentication.",
        input_type: "bytes",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "hash ripemd160",
        example: "cube hash ripemd160 --text abc",
    },
];
