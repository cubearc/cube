use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "Bounded binary agev1/X25519 decryption with complete header and STREAM authentication before output.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact agev1/X25519 binary-only profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "identity",
        description: "Required one canonical uppercase Bech32 X25519 identity; private transport only.",
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
            canonical_name: "Age authenticated file decryption",
            family: "authenticated_formats",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "One X25519 identity and binaryagev1 only. Canonical unknown recipient stanzas are bounded and ignored; no alternative identity, KDF, SSH or plugin is invoked. Scrypt and armor are refused before unwrap. Authentication does not identify a sender. No audit or certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["identity"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "One canonical uppercase74-character Bech32 X25519 identity; no identity-file grammar, password or key lookup.",
                key_advancement: "C2SPv1.1.0 X25519/HKDF wrapping, header HMAC-SHA256 and nonce/counter/final-marker ChaCha20-Poly1305 STREAM.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Ciphertext/plaintext each <=16MiB, header <=65536 bytes and8 total stanzas, settings <=2048 bytes. Allocation-free framing preflight and fixed32-byte canonical base64 fields precede maintained parser/recipient work. No scrypt or external lookup. Quarantined plaintext is wiped on failure; no output until every chunk and final marker/EOF authenticate. Caller/parser/interpreter copies are not guaranteed erased. No pipeline/automatic capture; Play requires explicit private retention consent.",
            rust_api: api,
            python_api,
            references: &[
                "https://c2sp.org/age@v1.1.0",
                "https://docs.rs/age/0.12.1/age/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 1] = [row(
    DECRYPT,
    "cipher age decrypt",
    "cube_core::ciphers::age::decrypt",
    "age_decrypt",
    "cube cipher age decrypt --parameters-file settings.json --file encrypted.bin",
)];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_INPUT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_INPUT);
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
    ..PORT
};
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[PORT],
    parameters: c::schema::<Parameters>,
    outputs: &[OUTPUT],
    limits: c::Limits {
        max_input_bytes: Some(MAX_INPUT as u64),
        max_output_bytes: Some(MAX_INPUT as u64),
        max_items: None,
        max_work: Some(264),
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture() -> Result<c::Fixture, OperationError> {
    let cases: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &cases[0];
    let input = crate::encoding::base64_decode(v["ciphertext_base64"].as_str().unwrap()).unwrap();
    let expected = crate::encoding::base64_decode(v["plaintext_base64"].as_str().unwrap()).unwrap();
    let parameters = v["parameters"].clone();
    let settings = Settings::parse(&parameters.to_string())?;
    let output = decrypt(&input, &settings)?;
    assert_eq!(output, expected);
    Ok(c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:input}}),
        parameters,
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    })
}
