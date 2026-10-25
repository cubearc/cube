//! Deliberate private-material conversion, separate from safe validation reports.
use super::{Network, PrivateInput, invalid};
use crate::{
    encoding::{self, base58check},
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use std::fmt;
const OP: &str = "blockchain.private.export";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Raw,
    Hex,
    Wif,
}
impl OutputFormat {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "raw" => Ok(Self::Raw),
            "hex" => Ok(Self::Hex),
            "wif" => Ok(Self::Wif),
            _ => Err(invalid("output", "private output must be raw, hex or wif").operation(OP)),
        }
    }
}
/// Explicit plaintext result. No serialization; Debug never shows its contents.
/// Returned/interface copies are caller-owned and not guaranteed zeroized.
pub enum ExportedPrivate {
    Raw([u8; 32]),
    Hex(String),
    Wif(String),
}
impl fmt::Debug for ExportedPrivate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExportedPrivate([REDACTED])")
    }
}
impl ExportedPrivate {
    /// Deliberately take ownership of exact private bytes; text has no newline.
    pub fn into_private_bytes(self) -> Vec<u8> {
        match self {
            Self::Raw(bytes) => bytes.to_vec(),
            Self::Hex(text) | Self::Wif(text) => text.into_bytes(),
        }
    }
}
/// Export validated material explicitly. Raw output is exactly 32 bytes, hex is
/// exactly 64 lowercase digits, and WIF requires explicit network/compression.
/// WIF source intent must match supplied expectations, including WIF-to-WIF.
/// Context flags on scalar-to-scalar conversion are rejected rather than ignored.
/// No writes, network, implicit logging/capture or memory-zeroization guarantee.
pub fn export(
    input: PrivateInput<'_>,
    output: OutputFormat,
    network: Option<Network>,
    compressed: Option<bool>,
) -> Result<ExportedPrivate, OperationError> {
    let result = (|| {
        if output == OutputFormat::Wif {
            if network.is_none() {
                return Err(invalid(
                    "network",
                    "WIF export requires an explicit network",
                ));
            }
            if compressed.is_none() {
                return Err(invalid(
                    "compressed",
                    "WIF export requires explicit public-key compression intent",
                ));
            }
        } else if !matches!(input, PrivateInput::Wif(_))
            && (network.is_some() || compressed.is_some())
        {
            return Err(invalid(
                "network",
                "network/encoding expectations require WIF input or output",
            ));
        }
        let (key, _) = super::parse_private(input, network, compressed)?;
        let bytes: [u8; 32] = key.0.to_bytes().into();
        Ok(match output {
            OutputFormat::Raw => ExportedPrivate::Raw(bytes),
            OutputFormat::Hex => ExportedPrivate::Hex(encoding::hex_encode(&bytes)),
            OutputFormat::Wif => {
                let network =
                    network.ok_or_else(|| invalid("network", "WIF export requires a network"))?;
                let compressed = compressed.ok_or_else(|| {
                    invalid("compressed", "WIF export requires compression intent")
                })?;
                let mut payload = bytes.to_vec();
                if compressed {
                    payload.push(1);
                }
                ExportedPrivate::Wif(base58check::encode(network.private_version(), &payload)?)
            }
        })
    })();
    result.map_err(|e: OperationError| e.operation(OP))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "blockchain",
    description: "Explicit private export between exact raw32, hex64 and supported WIF. WIF output requires network/compression; conflicting source WIF intent fails. <=128-byte input/output; returned plaintext is caller-owned, Debug redacted/no Serialize. CLI requires --private-output and explicit stdout/new-file destination. No automatic I/O, logging, pipeline/capture adapter or memory-zeroization promise.",
    input_type: "private.material",
    output_type: "private.export",
    deterministic: true,
    reversible: Some(true),
    command: "blockchain export",
    example: "cube blockchain export --format hex --to wif --network mainnet --compressed --private-output --stdout < public-test-scalar.hex",
    parameters: &[
        Parameter {
            name: "format",
            description: "Exact declared input representation.",
            required: false,
            default: Some(ParameterValue::Text("raw")),
            constraints: ParameterType::Text {
                choices: &["raw", "hex", "wif"],
            },
        },
        Parameter {
            name: "output",
            description: "Explicit private output: raw32, lowercase hex64 or WIF; no implicit export.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["raw", "hex", "wif"],
            },
        },
        Parameter {
            name: "network",
            description: "Required for WIF output; optional source-WIF expectation for raw/hex output.",
            required: false,
            default: None,
            constraints: ParameterType::Text {
                choices: &["mainnet", "testnet"],
            },
        },
        Parameter {
            name: "compressed",
            description: "Required WIF-output intent; optional source-WIF expectation, never silently overridden.",
            required: false,
            default: None,
            constraints: ParameterType::Boolean,
        },
    ],
}];
