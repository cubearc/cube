use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "SM2/SM3 encryption with explicit uncompressed C1C3C2, C1C2C3 or canonical DER ciphertext representation.";

const PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "profile",
    description: "Required exact SM2/SM3 ciphertext representation; no format guessing.",
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
    randomized: bool,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: DESCRIPTION,
            input_type: "bytes",
            output_type: "bytes",
            parameters: PARAMETERS,
            deterministic: !randomized,
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "SM2 encryption",
            family: "asymmetric_encryption",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "SM2/SM3 provides ciphertext integrity and confidentiality, not sender authentication. This uses the standard SM2 curve, not P-256 or secp256k1. No signature, key-exchange, custom curve, deterministic production randomness or audit/certification claim. Empty messages are refused.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "Encryption: exactly65 uncompressed SEC1 public bytes on SM2. Decryption: exactly32 big-endian private bytes in1..n-1. Key is a separate private byte port.",
                key_advancement: "Fresh fallible system CSPRNG encryption; SM3 counter KDF and C3=SM3(x2||message||y2). Decryption checks the full tag in constant time and rejects an all-zero KDF before releasing bytes.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Nonempty body <=16MiB; ciphertext <=body+128 bytes; key <=65 bytes; JSON <=256 bytes. Raw ciphertext C1 is exactly65 uncompressed SEC1 bytes; DER is canonical SEQUENCE(x INTEGER,y INTEGER,C3 OCTET STRING,C2 OCTET STRING). Point and length checks precede private work. Encryption entropy <=256 bytes (at most eight32-byte scalar draws). Owned private/plaintext buffers wiped on rejection/drop; backend, caller, hash/interpreter/OS copies are not guaranteed erased. No pipeline/automatic capture; Play requires explicit private retention consent and may enforce smaller bounds.",
            rust_api: api,
            python_api,
            references: &[
                "https://datatracker.ietf.org/doc/html/draft-shen-sm2-ecdsa-02#section-7",
                "https://docs.rs/sm2/0.14.0/sm2/",
                "https://docs.rs/sm3/0.5.0/sm3/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher sm2 encrypt",
        "cube_core::asymmetric::sm2::encrypt",
        "sm2_encrypt",
        "cube cipher sm2 encrypt --parameters-file settings.json --key-file public.bin --file message.bin",
        true,
    ),
    row(
        DECRYPT,
        "cipher sm2 decrypt",
        "cube_core::asymmetric::sm2::decrypt",
        "sm2_decrypt",
        "cube cipher sm2 decrypt --parameters-file settings.json --key-file secret.bin --file ciphertext.bin",
        false,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_CIPHERTEXT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_CIPHERTEXT);
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
    s["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_KEY.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_KEY);
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
const fn entry(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[PORT, KEY],
        parameters: c::schema::<Parameters>,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some((MAX_CIPHERTEXT + MAX_KEY) as u64),
            max_output_bytes: Some(MAX_CIPHERTEXT as u64),
            max_items: Some(1),
            max_work: Some(8 * MAX_BODY.div_ceil(32) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[entry(0, encrypt_fixture), entry(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn vector() -> serde_json::Value {
    let v: serde_json::Value = serde_json::from_str(include_str!("interoperability.json")).unwrap();
    v[0].clone()
}
fn make(input: Vec<u8>, key: Vec<u8>, output: Vec<u8>) -> c::Fixture {
    c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:input},"key":c::values::Bytes{data:key}}),
        parameters: serde_json::json!({"profile":"sm2_sm3_c1c3c2"}),
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    }
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let input = hex::decode(v["c1c3c2_hex"].as_str().unwrap()).unwrap();
    let s = DecryptionSettings::parse(r#"{"profile":"sm2_sm3_c1c3c2"}"#, &key)?;
    let output = decrypt(&input, &s)?;
    assert_eq!(hex::encode(&output), v["plaintext_hex"]);
    Ok(make(input, key, output))
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let key = hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap();
    let input = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    let s = EncryptionSettings::parse(r#"{"profile":"sm2_sm3_c1c3c2"}"#, &key)?;
    let output = encrypt(&input, &s)?;
    let private = DecryptionSettings::parse(
        r#"{"profile":"sm2_sm3_c1c3c2"}"#,
        &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
    )?;
    assert_eq!(decrypt(&output, &private)?, input);
    Ok(make(input, key, output))
}
