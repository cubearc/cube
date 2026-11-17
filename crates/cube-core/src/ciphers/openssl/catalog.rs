use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str =
    "Decrypt-only explicit binary OpenSSL Salted__ AES-CBC/PKCS7 profiles; unauthenticated output.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact OpenSSL cipher/KDF/salt/padding profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "password_hex",
        description: "Required exact password bytes as even-length hexadecimal; up to1024 bytes.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "iterations",
        description: "Required for PBKDF2, forbidden for EVP_BytesToKey;1..100000.",
        required: false,
        default: None,
        constraints: o::ParameterType::Integer {
            min: 1,
            max: 100000,
        },
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
            canonical_name: "OpenSSL legacy decryption",
            family: "legacy_formats",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Unauthenticated historical interoperability only. Padding success does not verify a password, key or message; modified ciphertext and wrong credentials can yield plaintext. No secure encryption or authentication claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["password_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "Exact binary password; explicit MD5/SHA256 EVP_BytesToKey or PBKDF2-HMAC-SHA256 derives AES key and IV.",
                key_advancement: "Required Salted__ header and profile-specific8/16-byte salt. Explicit KDF and PBKDF2 iterations, no fallback.",
                block_bits: Some(128),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Binary ciphertext and plaintext each <=16MiB; password <=1024 bytes; settings <=4096 bytes. PBKDF2 requires1..100000 iterations, at most200000 HMAC invocations. Uniform header/length/padding failure with no partial plaintext. Keys, passwords and owned tentative plaintext wiped; caller/parser/interpreter copies are not guaranteed erased. No pipeline or automatic capture; Play requires private retention consent.",
            rust_api: api,
            python_api,
            references: &[
                "https://docs.openssl.org/3.5/man1/openssl-enc/",
                "https://docs.openssl.org/3.5/man3/EVP_BytesToKey/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 1] = [row(
    DECRYPT,
    "cipher openssl decrypt",
    "cube_core::ciphers::openssl::decrypt",
    "openssl_decrypt",
    "cube cipher openssl decrypt --parameters-file settings.json --file encrypted.bin",
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
fn parameters_schema() -> schemars::Schema {
    let mut value = c::schema::<Parameters>().to_value();
    let profiles: Vec<_> = Profile::NAMES
        .iter()
        .filter(|p| p.contains("pbkdf2"))
        .copied()
        .collect();
    value["properties"]["iterations"]
        .as_object_mut()
        .expect("typed cost")
        .remove("default");
    let legacy: Vec<_> = Profile::NAMES
        .iter()
        .filter(|p| !p.contains("pbkdf2"))
        .copied()
        .collect();
    value["allOf"] = serde_json::json!([{"oneOf":[
      {"type":"object","properties":{"profile":{"enum":profiles},"iterations":{"type":"integer","minimum":1,"maximum":100000}},"required":["profile","iterations"]},
      {"type":"object","properties":{"profile":{"enum":legacy},"iterations":false},"required":["profile"]}
    ]}]);
    schemars::Schema::try_from(value).expect("typed explicit KDF settings")
}
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[PORT],
    parameters: parameters_schema,
    outputs: &[OUTPUT],
    limits: c::Limits {
        max_input_bytes: Some(MAX_INPUT as u64),
        max_output_bytes: Some(MAX_INPUT as u64),
        max_items: None,
        max_work: Some(200000),
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture() -> Result<c::Fixture, OperationError> {
    let cases: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &cases[0];
    let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let expected = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
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
