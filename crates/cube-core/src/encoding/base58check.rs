//! Bounded single-version-byte Base58Check, independent of wallet/network policy.
use crate::{
    errors::{ErrorCode, OperationError},
    hashing,
    operations::{Descriptor, Parameter, ParameterType},
};
use serde::Serialize;

pub const MAX_PAYLOAD: usize = 1024;
pub const MAX_FRAME: usize = MAX_PAYLOAD + 5;
/// Conservative 1.38*frame conversion bound, including leading zero bytes.
pub const MAX_ENCODED: usize = MAX_FRAME * 138 / 100 + 1;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Decoded {
    pub version: u8,
    #[serde(rename = "payload_hex", serialize_with = "super::serialize_hex")]
    pub payload: Vec<u8>,
    pub checksum_verified: bool,
}
fn error(code: ErrorCode, message: &str, operation: &'static str) -> OperationError {
    OperationError::new(code, message).operation(operation)
}
fn checksum(body: &[u8]) -> [u8; 4] {
    let digest = hashing::sha256_bytes(&hashing::sha256_bytes(body));
    [digest[0], digest[1], digest[2], digest[3]]
}
/// Encode one version byte and up to 1024 payload bytes, including empty payloads.
/// Checksum is the first four raw bytes of double SHA-256, not digest text.
/// Allocation/work are bounded before calling the existing Bitcoin Base58 codec.
/// Leading zeros are preserved. The checksum detects corruption, not authenticity.
pub fn encode(version: u8, payload: &[u8]) -> Result<String, OperationError> {
    const OP: &str = "encoding.base58check.encode";
    if payload.len() > MAX_PAYLOAD {
        return Err(error(
            ErrorCode::ResourceLimit,
            "Base58Check payload exceeds 1024 bytes",
            OP,
        ));
    }
    let mut frame = Vec::with_capacity(payload.len() + 5);
    frame.push(version);
    frame.extend_from_slice(payload);
    let check = checksum(&frame);
    frame.extend_from_slice(&check);
    Ok(super::base58_encode(&frame))
}
/// Strictly decode bounded Bitcoin-alphabet text and verify its checksum.
/// Empty text, whitespace/nonalphabet input and frames below five bytes fail.
/// Working decoded storage is bounded by 1421 input characters; accepted frames
/// are at most 1029 bytes and returned payloads at most 1024. No input is echoed.
pub fn decode(text: &str) -> Result<Decoded, OperationError> {
    const OP: &str = "encoding.base58check.decode";
    if text.len() > MAX_ENCODED {
        return Err(error(
            ErrorCode::ResourceLimit,
            "Base58Check text exceeds 1421 bytes",
            OP,
        ));
    }
    let frame = super::base58_decode(text).map_err(|_| {
        error(
            ErrorCode::InvalidInput,
            "invalid Base58Check alphabet; whitespace is not accepted",
            OP,
        )
    })?;
    if frame.len() < 5 {
        return Err(error(
            ErrorCode::InvalidInput,
            "Base58Check requires a version byte and four checksum bytes",
            OP,
        ));
    }
    if frame.len() > MAX_FRAME {
        return Err(error(
            ErrorCode::ResourceLimit,
            "Base58Check frame exceeds 1029 bytes",
            OP,
        ));
    }
    let body = frame.len() - 4;
    if frame[body..] != checksum(&frame[..body]) {
        return Err(error(
            ErrorCode::InvalidInput,
            "Base58Check checksum mismatch",
            OP,
        ));
    }
    Ok(Decoded {
        version: frame[0],
        payload: frame[1..body].to_vec(),
        checksum_verified: true,
    })
}

const PROFILED: [crate::operations::metadata::Entry; 2] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "encoding.base58check.encode",
            category: "encoding",
            description: "Single-byte version plus <=1024 payload bytes, raw double-SHA256 four-byte checksum and Bitcoin Base58. Empty payload allowed; <=1421 encoded bytes, bounded quadratic conversion. Corruption check, not authentication.",
            input_type: "bytes",
            output_type: "text",
            deterministic: true,
            reversible: Some(true),
            command: "encode base58check",
            example: "cube encode base58check --version-byte 0 --text abc",
            parameters: &[Parameter {
                name: "version",
                description: "Exact single-byte data prefix, separate from any pipeline adapter-contract version.",
                required: true,
                default: None,
                constraints: ParameterType::Unsigned { min: 0, max: 255 },
            }],
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Base58Check",
                family: "base_encodings",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::Encoding,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Bitcoin Base58 plus one version byte and first four bytes of double SHA-256; checksum verified.",
                    key: "Required single-byte version for Base58Check encoding.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "payload_bytes",
                        maximum: crate::encoding::base58check::MAX_PAYLOAD as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "encoded_bytes",
                        maximum: crate::encoding::base58check::MAX_ENCODED as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "frame_bytes",
                        maximum: crate::encoding::base58check::MAX_FRAME as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Decode is structured and direct-only; checksum detects corruption, not authenticity. Network/wallet interpretation is outside this codec.",
                rust_api: "cube_core::encoding::base58check::encode",
                python_api: "base58check_encode",
                references: &["https://docs.rs/bs58/0.5.1/bs58/alphabet/struct.Alphabet.html"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "encoding.base58check.decode",
            category: "encoding",
            description: "Verify bounded Bitcoin Base58Check text and return exact version/payload. <=1421 encoded/working bytes, 5..1029 accepted frame bytes, <=1024 payload bytes; no trimming, network inference or authentication claim. Structured direct result, no pipeline adapter.",
            input_type: "text",
            output_type: "encoding.base58check.decoded",
            deterministic: true,
            reversible: Some(true),
            command: "decode base58check",
            example: "cube decode base58check --text 1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo --json",
            parameters: &[],
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Base58Check",
                family: "base_encodings",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::Encoding,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Bitcoin Base58 plus one version byte and first four bytes of double SHA-256; checksum verified.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "payload_bytes",
                        maximum: crate::encoding::base58check::MAX_PAYLOAD as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "encoded_bytes",
                        maximum: crate::encoding::base58check::MAX_ENCODED as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "frame_bytes",
                        maximum: crate::encoding::base58check::MAX_FRAME as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Decode is structured and direct-only; checksum detects corruption, not authenticity. Network/wallet interpretation is outside this codec.",
                rust_api: "cube_core::encoding::base58check::decode",
                python_api: "base58check_decode",
                references: &["https://docs.rs/bs58/0.5.1/bs58/alphabet/struct.Alphabet.html"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
