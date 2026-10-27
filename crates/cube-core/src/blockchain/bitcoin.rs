//! Public SEC1/P2PKH derivation using the shared validated scalar and standard digests.
use super::{Network, PrivateInput, PrivateKey};
use crate::{
    encoding::base58check,
    errors::OperationError,
    hashing,
    operations::{Descriptor, Parameter, ParameterType},
};
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BitcoinAddress {
    pub network_family: Network,
    pub compressed: bool,
    pub address_version: u8,
    #[serde(
        rename = "public_key_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    #[schemars(with = "String", regex(pattern = r"^[0-9a-f]*$"))]
    pub public_key: Vec<u8>,
    #[serde(
        rename = "hash160_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    #[schemars(with = "String", regex(pattern = r"^[0-9a-f]*$"))]
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
const PROFILED: [crate::operations::metadata::Entry; 1] = [{
    const DECLARATION: crate::operations::Descriptor = Descriptor {
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
    };
    crate::operations::metadata::Entry {
        descriptor: DECLARATION,
        metadata: Some(crate::operations::metadata::Metadata {
            schema_version: 1,
            canonical_name: "Bitcoin P2PKH public derivation",
            family: "public_address_derivation",
            aliases: &[],
            security: crate::operations::metadata::Security {
                classification: crate::operations::metadata::Classification::PublicDerivation,
                limitations: "Known private-key input; no logging/capture, hidden I/O, network validation, zeroization or certification promise. Outputs are public derivation or structural validation facts, not private material.",
                private_input: true,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: crate::operations::metadata::Profile {
                variant: DECLARATION.description,
                alphabet: "Exact declared raw32/hex64/WIF representations; no trimming, normalization or network inference.",
                key: "Nonzero 256-bit big-endian secp256k1 scalar strictly below the curve order; preserve leading zeros.",
                key_advancement: "Not applicable.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[crate::operations::metadata::ResourceBound {
                name: "material_bytes",
                maximum: crate::blockchain::MAX_MATERIAL_BYTES as u64,
                unit: "bytes",
                scope: crate::operations::metadata::Scope::Core,
            }],
            resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Direct-only secret inputs. Actual private export still requires its existing explicit intent/capability controls; annotation grants no execution permission.",
            rust_api: "cube_core::blockchain::bitcoin::derive",
            python_api: "derive_bitcoin_address",
            references: &[
                "https://www.secg.org/sec2-v2.pdf",
                "https://github.com/bitcoin/bitcoin/blob/v29.0/src/key_io.cpp",
            ],
        }),
    }
}];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
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
