use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str =
    "Explicit X25519 or P-256 raw key agreement; no encryption, authentication or implicit KDF.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact X25519 RFC7748 or P-256 SEC1 profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "private_key_hex",
        description: "Required exactly32 private bytes as64 hexadecimal digits; private transport only.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
];
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
            canonical_name: "Elliptic-curve key agreement",
            family: "key_agreement",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Agreement alone does not authenticate a peer or encrypt data. Feed the raw shared secret into an explicit context-bound KDF. X25519 masks/reduces RFC7748 encodings and rejects zero output; P-256 requires canonical SEC1 points and a valid nonzero scalar. No certification or audit claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["private_key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "X25519:32-byte secret and peer public. P-256:32-byte scalar and33/65-byte SEC1 peer public.",
                key_advancement: "One fixed-curve multiplication; no RNG, KDF, context guessing or encryption.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Peer public key <=65 bytes; output exactly32 bytes; settings <=1024 bytes. Curve-specific lengths checked before point parsing. Owned private key buffers and backend keys wiped on drop; caller/parser/interpreter copies are not guaranteed erased. No pipeline/automatic capture; Play requires explicit private retention consent.",
            rust_api: api,
            python_api,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc7748",
                "https://www.rfc-editor.org/rfc/rfc5903",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 1] = [row(
    AGREE,
    "cipher ecdh agree",
    "cube_core::asymmetric::ecdh::agree",
    "ecdh_agree",
    "cube cipher ecdh agree --parameters-file settings.json --file peer.bin",
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
const OUTPUT: c::Port = c::Port {
    name: "output",
    schema: output_schema,
    ..PORT
};
fn output_schema() -> schemars::Schema {
    let mut schema = bytes_schema().to_value();
    schema["properties"]["base64"]["minLength"] = serde_json::json!(44);
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(44);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(32);
    schemars::Schema::try_from(schema).expect("exact32-byte shared secret")
}
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[PORT],
    parameters: c::schema::<Parameters>,
    outputs: &[OUTPUT],
    limits: c::Limits {
        max_input_bytes: Some(MAX_PUBLIC_KEY as u64),
        max_output_bytes: Some(32),
        max_items: None,
        max_work: Some(1),
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture() -> Result<c::Fixture, OperationError> {
    let peer =
        hex::decode("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f").unwrap();
    let expected =
        hex::decode("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742").unwrap();
    let parameters = serde_json::json!({"profile":"x25519_rfc7748","private_key_hex":"77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a"});
    let settings = Settings::parse(&parameters.to_string())?;
    let output = agree(&peer, &settings)?;
    assert_eq!(output, expected);
    Ok(c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:peer}}),
        parameters,
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    })
}
