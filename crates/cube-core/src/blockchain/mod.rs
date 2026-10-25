//! Offline, bounded key-format operations; no wallets, generation or network I/O.
pub mod bitcoin;
pub mod ethereum;
pub mod private;
pub mod wif;
use crate::{
    encoding,
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
use std::fmt;

pub const MAX_MATERIAL_BYTES: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "testnet_family")]
    Testnet,
}
impl Network {
    /// `testnet` selects a version shared by test networks, signet and regtest.
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            _ => Err(invalid(
                "network",
                "network must be mainnet or testnet (shared version family)",
            )),
        }
    }
    pub fn p2pkh_version(self) -> u8 {
        match self {
            Self::Mainnet => 0x00,
            Self::Testnet => 0x6f,
        }
    }
    pub fn private_version(self) -> u8 {
        match self {
            Self::Mainnet => 0x80,
            Self::Testnet => 0xef,
        }
    }
}
fn invalid(parameter: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).parameter(parameter)
}
fn malformed(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
}

/// Caller-supplied representations. Debug intentionally never displays material.
#[derive(Clone, Copy)]
pub enum PrivateInput<'a> {
    Raw(&'a [u8]),
    Hex(&'a str),
    Wif(&'a str),
}
impl<'a> PrivateInput<'a> {
    /// Borrow exact CLI/native transport bytes without normalization or secret errors.
    pub fn parse(format: &str, data: &'a [u8]) -> Result<Self, OperationError> {
        if data.len() > MAX_MATERIAL_BYTES {
            return Err(limit("private material exceeds 128 bytes"));
        }
        if format == "raw" {
            return Ok(Self::Raw(data));
        }
        let text =
            std::str::from_utf8(data).map_err(|_| malformed("text key material must be UTF-8"))?;
        match format {
            "hex" => Ok(Self::Hex(text)),
            "wif" => Ok(Self::Wif(text)),
            _ => Err(invalid("format", "format must be raw, hex or wif")),
        }
    }
}
impl fmt::Debug for PrivateInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateInput([REDACTED])")
    }
}
/// Validated nonzero secp256k1 scalar. No Serialize or implicit private export.
/// The library owns curve/range checks. Interface buffers may retain copies;
/// this type does not promise process-wide or Python memory zeroization.
pub struct PrivateKey(pub(crate) k256::SecretKey);
impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateKey([REDACTED])")
    }
}
impl PrivateKey {
    /// Require exactly 32 big-endian bytes and 1 <= d < the secp256k1 group order.
    pub fn from_bytes(data: &[u8]) -> Result<Self, OperationError> {
        const OP: &str = "blockchain.scalar.validate";
        if data.len() != 32 {
            return Err(invalid(
                "material",
                "private scalar must contain exactly 32 big-endian bytes",
            )
            .operation(OP));
        }
        k256::SecretKey::from_slice(data)
            .map(Self)
            .map_err(|_| malformed("private scalar is outside the secp256k1 range").operation(OP))
    }
    /// Require exactly 64 ASCII hex digits, either case; no prefix/whitespace.
    pub fn from_hex(text: &str) -> Result<Self, OperationError> {
        if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid(
                "material",
                "scalar hex must contain exactly 64 ASCII hex digits without prefix or whitespace",
            )
            .operation("blockchain.scalar.validate"));
        }
        let bytes = encoding::hex_decode(text)
            .map_err(|_| malformed("invalid scalar hex").operation("blockchain.scalar.validate"))?;
        Self::from_bytes(&bytes)
    }
    pub(crate) fn sec1_public_key(&self, compressed: bool) -> Vec<u8> {
        use k256::elliptic_curve::sec1::ToSec1Point;
        self.0
            .public_key()
            .to_sec1_point(compressed)
            .as_bytes()
            .to_vec()
    }
    pub fn info(&self) -> ScalarInfo {
        ScalarInfo {
            valid: !bool::from(self.0.as_scalar_value().is_zero()),
            curve: "secp256k1",
            byte_width: 32,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ScalarInfo {
    pub valid: bool,
    pub curve: &'static str,
    pub byte_width: usize,
}
/// Validate raw/hex scalar input, returning only safe fixed structural metadata.
pub fn validate_scalar(input: PrivateInput<'_>) -> Result<ScalarInfo, OperationError> {
    match input {
        PrivateInput::Raw(bytes) => PrivateKey::from_bytes(bytes),
        PrivateInput::Hex(text) => PrivateKey::from_hex(text),
        PrivateInput::Wif(_) => Err(invalid("format", "use WIF validation for WIF material")
            .operation("blockchain.scalar.validate")),
    }
    .map(|key| key.info())
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "blockchain.scalar.validate",
    category: "blockchain",
    description: "Validate exactly 32 big-endian raw bytes or 64 strict hex digits using maintained secp256k1 scalar validation (1 <= d < n). Safe fixed structural metadata only; no private bytes/fingerprints, I/O or pipeline adapter.",
    input_type: "private.scalar",
    output_type: "key.validation",
    parameters: &[Parameter {
        name: "format",
        description: "Declared scalar representation; hex preserves all 32 bytes and accepts no prefixes/whitespace.",
        required: false,
        default: Some(ParameterValue::Text("raw")),
        constraints: ParameterType::Text {
            choices: &["raw", "hex"],
        },
    }],
    deterministic: true,
    reversible: None,
    command: "blockchain scalar",
    example: "cube blockchain scalar --format hex --json < public-test-scalar.hex",
}];

pub(crate) fn parse_private(
    input: PrivateInput<'_>,
    network: Option<Network>,
    compressed: Option<bool>,
) -> Result<(PrivateKey, Option<wif::WifInfo>), OperationError> {
    match input {
        PrivateInput::Raw(bytes) => PrivateKey::from_bytes(bytes).map(|key| (key, None)),
        PrivateInput::Hex(text) => PrivateKey::from_hex(text).map(|key| (key, None)),
        PrivateInput::Wif(text) => {
            wif::decode(text, network, compressed).map(|(key, info)| (key, Some(info)))
        }
    }
}
