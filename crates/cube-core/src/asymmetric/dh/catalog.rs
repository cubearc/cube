use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str =
    "RFC7919 named finite-field raw agreement with subgroup validation; no authentication or KDF.";

const PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "profile",
    description: "Required RFC7919 named finite-field profile with exact byte encodings.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text {
        choices: Profile::NAMES,
    },
}];
const fn row(
    id: &'static str,
    command: &'static str,
    api: &'static str,
    python_api: &'static str,
    example: &'static str,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: DESCRIPTION,
            input_type: "bytes",
            output_type: "bytes",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "Finite-field key agreement",
            family: "key_agreement",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Raw DH does not authenticate peers or encrypt data. Use an explicit context-bound KDF before using the shared secret as key material. Peers must be in the named prime-order subgroup. No arbitrary groups, key generation or certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "RFC7919 ffdhe2048/3072/4096/6144/8192. Exact fixed-width big-endian public values256/384/512/768/1024 bytes; separate private exponents32/40/48/56/64 bytes. Secret0/1 refused; caller must supply fresh uniformly random keys.",
                key_advancement: "One public subgroup check and one private exponentiation. Constant-time fixed-window secret arithmetic at the public per-profile exponent width; no implicit KDF.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Peer <=1024 bytes, private key <=64 bytes, output fixed to modulus width, JSON <=256 bytes. Public range and subgroup checks precede secret work. Bounded named groups, no unbounded prime search. Owned private arithmetic wiped on drop; caller/interpreter copies are not guaranteed erased. No pipeline/automatic capture; native Play requires explicit private retention consent.",
            rust_api: api,
            python_api,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc7919",
                "https://docs.rs/crypto-bigint/0.7.11/crypto_bigint/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 1] = [row(
    AGREE,
    "cipher dh agree",
    "cube_core::asymmetric::dh::agree",
    "dh_agree",
    "cube cipher dh agree --parameters-file settings.json --key-file secret.bin --file peer.bin",
)];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_PUBLIC_KEY.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_PUBLIC_KEY);
    schemars::Schema::try_from(schema).expect("bounded typed byte transport")
}
const PORT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: bytes_schema,
    required: true,
};
fn key_schema() -> schemars::Schema {
    let mut s = bytes_schema().to_value();
    s["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_PRIVATE_KEY.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_PRIVATE_KEY);
    schemars::Schema::try_from(s).expect("bounded key")
}
const KEY: c::Port = c::Port {
    name: "key",
    schema: key_schema,
    ..PORT
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    schema: output_schema,
    ..PORT
};
fn output_schema() -> schemars::Schema {
    bytes_schema()
}
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[PORT, KEY],
    parameters: c::schema::<Parameters>,
    outputs: &[OUTPUT],
    limits: c::Limits {
        max_input_bytes: Some((MAX_PUBLIC_KEY + MAX_PRIVATE_KEY) as u64),
        max_output_bytes: Some(MAX_PUBLIC_KEY as u64),
        max_items: None,
        max_work: Some(16384),
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture() -> Result<c::Fixture, OperationError> {
    let vectors: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &vectors[0];
    let parameters = v["parameters"].clone();
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let peer = hex::decode(v["peer_hex"].as_str().unwrap()).unwrap();
    let s = Settings::parse(&parameters.to_string(), &key)?;
    let output = agree(&peer, &s)?;
    assert_eq!(hex::encode(&output), v["shared_hex"]);
    Ok(c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:peer},"key":c::values::Bytes{data:key}}),
        parameters,
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    })
}
