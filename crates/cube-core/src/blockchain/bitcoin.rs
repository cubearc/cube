//! Public SEC1/P2PKH derivation using the shared validated scalar and standard digests.
use super::{Network, PrivateInput, PrivateKey};
use crate::{
    encoding::base58check,
    errors::OperationError,
    hashing,
    operations::{Descriptor, Parameter, ParameterType},
};
use serde::Serialize;
#[derive(Debug, Serialize)]
pub struct BitcoinAddress {
    pub network_family: Network,
    pub compressed: bool,
    pub address_version: u8,
    #[serde(
        rename = "public_key_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    pub public_key: Vec<u8>,
    #[serde(
        rename = "hash160_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    pub hash160: [u8; 20],
    pub address: String,
}
fn address_from_hash(hash: &[u8; 20], network: Network) -> Result<String, OperationError> {
    base58check::encode(network.p2pkh_version(), hash)
}
pub(crate) fn from_key(
    key: &PrivateKey,
    network: Network,
    compressed: bool,
) -> Result<BitcoinAddress, OperationError> {
    let public_key = key.sec1_public_key(compressed);
    let hash160 = hashing::protocol::ripemd160_bytes(&hashing::sha256_bytes(&public_key));
    let address = address_from_hash(&hash160, network)?;
    Ok(BitcoinAddress {
        network_family: network,
        compressed,
        address_version: network.p2pkh_version(),
        public_key,
        hash160,
        address,
    })
}
/// Derive standard compressed/uncompressed SEC1 and legacy P2PKH offline.
/// Network and encoding are explicit; WIF network/compression intent must match.
/// Returns only public material. Input transports are exact raw32/hex64/WIF and
/// bounded at 128 bytes. No network calls, private export or automatic capture.
pub fn derive(
    input: PrivateInput<'_>,
    network: Network,
    compressed: bool,
) -> Result<BitcoinAddress, OperationError> {
    let result = (|| {
        let (key, _) = super::parse_private(input, Some(network), Some(compressed))?;
        from_key(&key, network, compressed)
    })();
    result.map_err(|e: OperationError| e.operation("blockchain.bitcoin.derive"))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "blockchain.bitcoin.derive",
    category: "blockchain",
    description: "Derive SEC1 public keys and legacy P2PKH using validated secp256k1, RIPEMD160(SHA256(pubkey bytes)) and Base58Check (0x00/0x6f). Explicit network/compression, exact raw32/hex64/WIF, <=128 transport bytes. Reject conflicting WIF intent; public output only, no network/capture adapter.",
    input_type: "private.material",
    output_type: "bitcoin.public_address",
    deterministic: true,
    reversible: Some(false),
    command: "blockchain bitcoin",
    example: "cube blockchain bitcoin --format hex --network mainnet --compressed --json < public-test-scalar.hex",
    parameters: &[
        Parameter {
            name: "format",
            description: "Exact supplied representation; no inference or normalization.",
            required: false,
            default: Some(crate::operations::ParameterValue::Text("raw")),
            constraints: ParameterType::Text {
                choices: &["raw", "hex", "wif"],
            },
        },
        Parameter {
            name: "network",
            description: "Explicit P2PKH network version; testnet selects the shared test-family version.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["mainnet", "testnet"],
            },
        },
        Parameter {
            name: "compressed",
            description: "Explicit public-key SEC1 encoding, matching WIF intent when supplied.",
            required: true,
            default: None,
            constraints: ParameterType::Boolean,
        },
    ],
}];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_hash_preserves_all_leading_bytes_without_claiming_a_private_key() {
        assert_eq!(
            address_from_hash(&[0; 20], Network::Mainnet).unwrap(),
            "1111111111111111111114oLvT2"
        );
    }
}
