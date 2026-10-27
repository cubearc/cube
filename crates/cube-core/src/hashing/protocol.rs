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
const PROFILED: [crate::operations::metadata::Entry; 2] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
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
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Keccak-256",
                family: "hash_functions",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::UnkeyedDigest,
                    limitations: "Unkeyed digest, not encryption, message authentication or password hashing. No implementation certification is claimed.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact bytes; no Unicode normalization.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: Some(1088),
                    output_bits: Some(256),
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "output_hex_characters",
                        maximum: 64,
                        unit: "ASCII characters",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "input_bytes",
                        maximum: crate::hashing::protocol::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::hashing::keccak256",
                python_api: "keccak256",
                references: &["https://keccak.team/keccak_specs_summary.html"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
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
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "RIPEMD-160",
                family: "hash_functions",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::UnkeyedDigest,
                    limitations: "Unkeyed digest, not encryption, message authentication or password hashing. No implementation certification is claimed.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact bytes; no Unicode normalization.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: Some(512),
                    output_bits: Some(160),
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "output_hex_characters",
                        maximum: 40,
                        unit: "ASCII characters",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "input_bytes",
                        maximum: crate::hashing::protocol::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::hashing::ripemd160",
                python_api: "ripemd160",
                references: &["https://homes.esat.kuleuven.be/~bosselae/ripemd160.html"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
