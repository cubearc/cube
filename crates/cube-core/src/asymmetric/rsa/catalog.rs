use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "RFC8017 RSA-OAEP SHA256/MGF1-SHA256 with CFRG draft10 depadding, or decrypt-only PKCS1v1.5 draft10 implicit rejection; bounded canonical keys and fresh system randomness.";
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact OAEP or implicit-rejection legacy profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &[
                "rsa_oaep_sha256_mgf1_sha256_cfrg10",
                "rsaes_pkcs1_v1_5_implicit_rejection_cfrg10",
            ],
        },
    },
    o::Parameter {
        name: "key_format",
        description: "Required SPKI public or unencrypted PKCS8 private DER/PEM encoding.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["spki_der", "spki_pem", "pkcs8_der", "pkcs8_pem"],
        },
    },
    o::Parameter {
        name: "label_hex",
        description: "OAEP label, at most4096 bytes; default empty. Refused for PKCS1v1.5.",
        required: false,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
];
const fn row(
    id: &'static str,
    command: &'static str,
    api: &'static str,
    python: &'static str,
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
            canonical_name: "RSA encryption profiles",
            family: "asymmetric_encryption",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "OAEP provides ciphertext confidentiality with label checking, not sender authentication. Legacy PKCS1v1.5 is decrypt-only and uses implicit rejection: success can contain synthetic bytes and never establishes padding validity or authenticity. CFRG draft10 deliberately ignores the OAEP leading encoded-message octet; this differs from strict RFC8017 rejection. No raw RSA, signatures, key generation, deterministic coins, OAEP fallback or certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "Explicit SPKI or PKCS8 DER/PEM; RSA encryption OID with NULL parameters, two balanced primes,2048..8192-bit modulus and backend exponent constraints3..2^33-1 odd.",
                key_advancement: "Fresh OS/browser CSPRNG for OAEP seed and base/exponent/CRT blinding. OAEP SHA256/MGF1-SHA256 only; label <=4096 bytes, default empty.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Encoded key <=65536 bytes; JSON settings <=10240 bytes; key is a separate private byte input. ASN1 and component-size preflight precedes bigint/private work. OAEP input <=k-66 and ciphertext exactlyk<=1024; legacy output <=k-11. One RSA operation per call; system entropy requests <=65536 bytes. Owned decoded private buffers and backend key arithmetic wiped on drop; caller/parser/interpreter copies are not guaranteed erased. No pipeline/automatic capture; native Play may enforce smaller transport limits.",
            rust_api: api,
            python_api: python,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc8017",
                "https://www.ietf.org/archive/id/draft-irtf-cfrg-rsa-guidance-10.html#section-7",
                "https://rustsec.org/advisories/RUSTSEC-2023-0071.html",
                "https://docs.rs/rsa/0.10.0/rsa/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 3] = [
    row(
        ENCRYPT,
        "cipher rsa oaep encrypt",
        "cube_core::asymmetric::rsa::encrypt",
        "rsa_oaep_encrypt",
        "cube cipher rsa oaep encrypt --parameters-file settings.json --key-file key.bin --file message.bin",
        true,
    ),
    row(
        DECRYPT,
        "cipher rsa oaep decrypt",
        "cube_core::asymmetric::rsa::decrypt",
        "rsa_oaep_decrypt",
        "cube cipher rsa oaep decrypt --parameters-file settings.json --key-file key.bin --file ciphertext.bin",
        false,
    ),
    row(
        LEGACY_DECRYPT,
        "cipher rsa pkcs1-v1-5 decrypt",
        "cube_core::asymmetric::rsa::legacy_decrypt",
        "rsa_pkcs1_v1_5_decrypt",
        "cube cipher rsa pkcs1-v1-5 decrypt --parameters-file settings.json --key-file key.bin --file ciphertext.bin",
        false,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut s = c::schema::<c::values::Bytes>().to_value();
    s["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_CIPHERTEXT.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_CIPHERTEXT);
    schemars::Schema::try_from(s).unwrap()
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: bytes_schema,
    required: true,
};
fn key_schema() -> schemars::Schema {
    let mut s = c::schema::<c::values::Bytes>().to_value();
    s["properties"]["base64"]["maxLength"] = serde_json::json!(keys::MAX_KEY.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(keys::MAX_KEY);
    schemars::Schema::try_from(s).unwrap()
}
const KEY: c::Port = c::Port {
    name: "key",
    semantic_type: "bytes",
    schema: key_schema,
    required: true,
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const fn entry(
    index: usize,
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[INPUT, KEY],
        parameters,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some(keys::MAX_KEY as u64 + if index == 0 { 958 } else { 1024 }),
            max_output_bytes: Some(if index == 0 {
                1024
            } else if index == 1 {
                958
            } else {
                1013
            }),
            max_items: Some(1),
            max_work: Some(8192),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    entry(0, c::schema::<PublicParameters>, encrypt_fixture),
    entry(1, c::schema::<PrivateParameters>, decrypt_fixture),
    entry(2, c::schema::<LegacyParameters>, legacy_fixture),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixture.json")).unwrap()
}
fn make(
    input: Vec<u8>,
    key: Vec<u8>,
    parameters: serde_json::Value,
    output: Vec<u8>,
) -> c::Fixture {
    c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:input},"key":c::values::Bytes{data:key}}),
        parameters,
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    }
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    let v = fixture();
    let p = v["parameters"].clone();
    let key = hex::decode(v["key_hex"].as_str().unwrap()).unwrap();
    let s = DecryptionSettings::parse(&p.to_string(), &key)?;
    let input = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let output = decrypt(&input, &s)?;
    assert_eq!(hex::encode(&output), v["plaintext_hex"]);
    Ok(make(input, key, p, output))
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    use rsa::pkcs8::EncodePublicKey;
    let v = fixture();
    let key = hex::decode(v["key_hex"].as_str().unwrap()).unwrap();
    let private = DecryptionSettings::parse(&v["parameters"].to_string(), &key)?;
    let public = private
        .key
        .to_public_key()
        .to_public_key_der()
        .map_err(|_| keys::invalid())?
        .as_bytes()
        .to_vec();
    let p = serde_json::json!({"profile":"rsa_oaep_sha256_mgf1_sha256_cfrg10","key_format":"spki_der","label_hex":v["parameters"]["label_hex"]});
    let s = EncryptionSettings::parse(&p.to_string(), &public)?;
    let input = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    let output = encrypt(&input, &s)?;
    assert_eq!(decrypt(&output, &private)?, input);
    Ok(make(input, public, p, output))
}
fn legacy_fixture() -> Result<c::Fixture, OperationError> {
    let v: serde_json::Value = serde_json::from_str(include_str!("wycheproof_pkcs1.json")).unwrap();
    let g = &v["testGroups"][0];
    let test = g["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["result"] == "valid" && v["msg"] != "")
        .unwrap();
    let key = hex::decode(g["privateKeyPkcs8"].as_str().unwrap()).unwrap();
    let p = serde_json::json!({"profile":"rsaes_pkcs1_v1_5_implicit_rejection_cfrg10","key_format":"pkcs8_der"});
    let s = LegacySettings::parse(&p.to_string(), &key)?;
    let input = hex::decode(test["ct"].as_str().unwrap()).unwrap();
    let output = legacy_decrypt(&input, &s)?;
    assert_eq!(hex::encode(&output), test["msg"]);
    Ok(make(input, key, p, output))
}
