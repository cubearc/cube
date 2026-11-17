use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "Complete ephemeral P-256 ECIES or ffdhe2048 DHIES using BC1.83 KDF2-SHA256 streaming XOR and HMAC-SHA256.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required complete ECIES or DHIES suite; no algorithm guessing.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "derivation_hex",
        description: "KDF derivation context, <=256 bytes in hexadecimal; default empty.",
        required: false,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "encoding_hex",
        description: "MAC encoding context, <=256 bytes in hexadecimal; default empty.",
        required: false,
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
            canonical_name: "Integrated encryption suites",
            family: "hybrid_encryption",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Ephemeral recipient encryption provides ciphertext integrity and confidentiality, not sender authentication. These complete IEEE1363a-style BC1.83 stream suites are distinct from HPKE RFC9180. No static sender mode, universal envelope, external key lookup, algorithm fallback or audit/certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "ECIES:65 uncompressed SEC1 P-256 public bytes or32 private scalar bytes. DHIES:256-byte RFC7919 ffdhe2048 public subgroup element or32-byte private exponent>1. Key is a separate private byte port.",
                key_advancement: "Fresh ephemeral secret from fallible OS/browser CSPRNG. V is the ephemeral public key; Z is the fixed-width raw agreement output. KDF2-SHA256(V||Z,counter,derivation) emits32 MAC key bytes then message-length XOR key stream. HMAC-SHA256 covers C||encoding||u64be(encoding bit length), verified before plaintext.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Body <=16MiB including empty; ciphertext V||C||T adds65/256 public bytes and32 tag bytes. Key <=256 bytes; JSON <=2048 bytes; each explicit context <=256 bytes. Public point/range/subgroup checks precede secret work. Ephemeral scalar draws <=8 and system entropy <=256 bytes. No IV. Owned private/shared/key-stream/plaintext buffers and SHA256/HMAC state erased where supported; caller/interpreter/OS copies are not guaranteed erased. No pipeline/automatic capture; Play requires explicit private retention consent and may enforce smaller limits.",
            rust_api: api,
            python_api,
            references: &[
                "https://github.com/bcgit/bc-java/blob/r1rv83/core/src/main/java/org/bouncycastle/crypto/engines/IESEngine.java",
                "https://www.rfc-editor.org/rfc/rfc7919",
                "https://www.rfc-editor.org/rfc/rfc9180",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher ies encrypt",
        "cube_core::asymmetric::ies::encrypt",
        "ies_encrypt",
        "cube cipher ies encrypt --parameters-file settings.json --key-file public.bin --file message.bin",
        true,
    ),
    row(
        DECRYPT,
        "cipher ies decrypt",
        "cube_core::asymmetric::ies::decrypt",
        "ies_decrypt",
        "cube cipher ies decrypt --parameters-file settings.json --key-file secret.bin --file ciphertext.bin",
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
            max_work: Some(MAX_BODY.div_ceil(32) as u64 + 8193),
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
    v[1].clone()
}
fn make(v: &serde_json::Value, input: Vec<u8>, key: Vec<u8>, output: Vec<u8>) -> c::Fixture {
    c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:input},"key":c::values::Bytes{data:key}}),
        parameters: v["parameters"].clone(),
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    }
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let s = DecryptionSettings::parse(&v["parameters"].to_string(), &key)?;
    let output = decrypt(&input, &s)?;
    assert_eq!(hex::encode(&output), v["plaintext_hex"]);
    Ok(make(&v, input, key, output))
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let key = hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap();
    let input = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    let s = EncryptionSettings::parse(&v["parameters"].to_string(), &key)?;
    let output = encrypt(&input, &s)?;
    let private = DecryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
    )?;
    assert_eq!(decrypt(&output, &private)?, input);
    Ok(make(&v, input, key, output))
}
