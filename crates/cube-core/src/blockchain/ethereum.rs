//! Offline Ethereum derivation and exact ERC-55 (EIP-55) text rendering.
use super::{PrivateInput, PrivateKey};
use crate::{
    encoding,
    errors::OperationError,
    hashing::protocol::keccak256_bytes,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Checksum {
    Lowercase,
    Eip55,
}
impl Checksum {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "lowercase" => Ok(Self::Lowercase),
            "eip55" => Ok(Self::Eip55),
            _ => Err(
                super::invalid("checksum", "checksum must be lowercase or eip55")
                    .operation("blockchain.ethereum.derive"),
            ),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct EthereumAddress {
    #[serde(
        rename = "public_key_xy_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    pub public_key_xy: Vec<u8>,
    #[serde(
        rename = "address_hex",
        serialize_with = "crate::encoding::serialize_hex"
    )]
    pub address_bytes: [u8; 20],
    pub address: String,
    pub checksum: Checksum,
}
fn render(bytes: &[u8; 20], checksum: Checksum) -> String {
    let hex = encoding::hex_encode(bytes);
    if checksum == Checksum::Lowercase {
        return format!("0x{hex}");
    }
    // ERC-55 hashes lowercase ASCII hex without 0x, not the 20 address bytes.
    let hash = keccak256_bytes(hex.as_bytes());
    let mut out = String::from("0x");
    for (i, b) in hex.bytes().enumerate() {
        let nibble = if i % 2 == 0 {
            hash[i / 2] >> 4
        } else {
            hash[i / 2] & 15
        };
        out.push(if nibble >= 8 {
            b.to_ascii_uppercase()
        } else {
            b
        } as char);
    }
    out
}
pub(crate) fn from_key(
    key: &PrivateKey,
    checksum: Checksum,
) -> Result<EthereumAddress, OperationError> {
    let sec1 = key.sec1_public_key(false);
    if sec1.len() != 65 || sec1[0] != 4 {
        return Err(super::malformed(
            "unexpected uncompressed SEC1 representation",
        ));
    }
    let public_key_xy = sec1[1..].to_vec();
    let digest = keccak256_bytes(&public_key_xy);
    let mut address_bytes = [0; 20];
    address_bytes.copy_from_slice(&digest[12..]);
    Ok(EthereumAddress {
        address: render(&address_bytes, checksum),
        public_key_xy,
        address_bytes,
        checksum,
    })
}
/// Exact raw32/hex64 scalar -> uncompressed x||y -> Keccak-256 -> final20.
/// EIP-55 hashes ASCII hex; no chain/network inference, I/O or private output.
/// WIF is rejected because its Bitcoin intent has no Ethereum equivalent.
pub fn derive(
    input: PrivateInput<'_>,
    checksum: Checksum,
) -> Result<EthereumAddress, OperationError> {
    let result = (|| {
        if matches!(input, PrivateInput::Wif(_)) {
            return Err(super::invalid(
                "format",
                "Ethereum derivation accepts raw or hex scalars",
            ));
        }
        let (key, _) = super::parse_private(input, None, None)?;
        from_key(&key, checksum)
    })();
    result.map_err(|e: OperationError| e.operation("blockchain.ethereum.derive"))
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "blockchain.ethereum.derive",
    category: "blockchain",
    description: "Offline validated secp256k1 raw32/hex64 scalar -> 64 uncompressed x||y bytes (no 04 prefix) -> Ethereum Keccak-256 (not SHA3) -> final20 bytes. Exact lowercase or ERC-55/EIP-55 text; <=128 transport bytes. Public output only; no network, persistence or pipeline/capture adapter.",
    input_type: "private.scalar",
    output_type: "ethereum.public_address",
    deterministic: true,
    reversible: Some(false),
    command: "blockchain ethereum",
    example: "cube blockchain ethereum --format hex --checksum eip55 --json < public-test-scalar.hex",
    parameters: &[
        Parameter {
            name: "format",
            description: "Exact declared scalar representation.",
            required: false,
            default: Some(ParameterValue::Text("raw")),
            constraints: ParameterType::Text {
                choices: &["raw", "hex"],
            },
        },
        Parameter {
            name: "checksum",
            description: "Lowercase hex or ERC-55/EIP-55 ASCII-hex checksum; no chain-specific checksum.",
            required: false,
            default: Some(ParameterValue::Text("eip55")),
            constraints: ParameterType::Text {
                choices: &["lowercase", "eip55"],
            },
        },
    ],
}];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn published_erc55_cases_and_zero_width_are_exact() {
        // https://github.com/ethereum/ercs/blob/master/ERCS/erc-55.md
        for expected in [
            "0x52908400098527886E0F7030069857D2E4169EE7",
            "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
            "0xde709f2102306220921060314715629080e2fb77",
            "0x27b1fdb04752bbc536007a920d24acb045561c26",
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ] {
            let bytes: [u8; 20] = encoding::hex_decode(&expected[2..])
                .unwrap()
                .try_into()
                .unwrap();
            assert_eq!(render(&bytes, Checksum::Eip55), expected);
            assert_eq!(
                render(&bytes, Checksum::Lowercase),
                expected.to_ascii_lowercase()
            );
        }
        for checksum in [Checksum::Lowercase, Checksum::Eip55] {
            assert_eq!(
                render(&[0; 20], checksum),
                "0x0000000000000000000000000000000000000000"
            );
        }
    }
}
