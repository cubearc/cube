//! WIF structural validation; secret bytes never enter the returned metadata.
use super::{MAX_MATERIAL_BYTES, Network, PrivateKey, invalid, limit, malformed};
use crate::{
    encoding::base58check,
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType},
};
use serde::Serialize;
#[derive(Clone, Copy, Debug, Serialize)]
pub struct WifInfo {
    pub valid: bool,
    pub version: u8,
    pub network_family: Network,
    pub compressed: bool,
    pub scalar_validated: bool,
    pub test_networks_indistinguishable: bool,
}
fn check_network(
    network: Network,
    expected_network: Option<Network>,
) -> Result<(), OperationError> {
    if expected_network.is_some_and(|n| n != network) {
        return Err(invalid(
            "network",
            "WIF version does not match the expected network family",
        ));
    }
    Ok(())
}
fn check_compression(
    compressed: bool,
    expected_compressed: Option<bool>,
) -> Result<(), OperationError> {
    if expected_compressed.is_some_and(|c| c != compressed) {
        return Err(invalid(
            "compressed",
            "requested public-key encoding conflicts with WIF intent",
        ));
    }
    Ok(())
}
pub(crate) fn check_intent(
    info: &WifInfo,
    network: Option<Network>,
    compressed: Option<bool>,
) -> Result<(), OperationError> {
    check_network(info.network_family, network)?;
    check_compression(info.compressed, compressed)
}
pub(super) fn decode(
    text: &str,
    expected_network: Option<Network>,
    expected_compressed: Option<bool>,
) -> Result<(PrivateKey, WifInfo), OperationError> {
    let result = (|| {
        if text.len() > MAX_MATERIAL_BYTES {
            return Err(limit("WIF text exceeds 128 bytes"));
        }
        let decoded = base58check::decode(text)?;
        let network = match decoded.version {
            0x80 => Network::Mainnet,
            0xef => Network::Testnet,
            _ => {
                return Err(OperationError::new(
                    ErrorCode::UnsupportedFormat,
                    "WIF version must be 0x80 or 0xef",
                ));
            }
        };
        check_network(network, expected_network)?;
        let compressed = match decoded.payload.len() {
            32 => false,
            33 if decoded.payload[32] == 1 => true,
            _ => {
                return Err(malformed(
                    "WIF payload must be 32 scalar bytes or 32 bytes followed by 0x01",
                ));
            }
        };
        check_compression(compressed, expected_compressed)?;
        let key = PrivateKey::from_bytes(&decoded.payload[..32])?;
        let info = WifInfo {
            valid: true,
            version: decoded.version,
            network_family: network,
            compressed,
            scalar_validated: true,
            test_networks_indistinguishable: network == Network::Testnet,
        };
        Ok((key, info))
    })();
    result.map_err(|e: OperationError| e.operation("blockchain.wif.validate"))
}
/// Validate mainnet/test-family WIF, including checksum, exact shape and scalar.
/// No trimming, secret export, derived identifiers, I/O or automatic capture.
/// A 0xef version cannot distinguish testnet, signet and regtest.
pub fn validate(text: &str, expected_network: Option<Network>) -> Result<WifInfo, OperationError> {
    decode(text, expected_network, None).map(|(_, info)| info)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "blockchain.wif.validate",
    category: "blockchain",
    description: "Validate <=128 WIF bytes via Base58Check, supported 0x80/0xef version, 32-byte nonzero secp256k1 scalar < n and optional 0x01 compression marker. Return safe version/network-family/compression metadata, never private bytes, WIF or fingerprints. Shared test versions cannot identify a chain; no I/O or pipeline adapter.",
    input_type: "private.wif",
    output_type: "key.validation",
    parameters: &[Parameter {
        name: "expected_network",
        description: "Optional encoded network-family expectation; testnet means shared testnet/signet/regtest versions.",
        required: false,
        default: None,
        constraints: ParameterType::Text {
            choices: &["mainnet", "testnet"],
        },
    }],
    deterministic: true,
    reversible: None,
    command: "blockchain wif",
    example: "cube blockchain wif --expected-network mainnet --json < public-test-key.wif",
}];
