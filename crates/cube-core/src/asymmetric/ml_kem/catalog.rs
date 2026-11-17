use super::*;
use crate::asymmetric::kem_wire::{byte_port as port, byte_schema};
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};

const PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "profile",
    description: "Required FIPS 203 parameter set with raw key and ciphertext encodings.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text {
        choices: Profile::NAMES,
    },
}];

const fn row(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    command: &'static str,
    api: (&'static str, &'static str),
    example: &'static str,
    decapsulation: bool,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description,
            input_type: "json",
            output_type: "json",
            parameters: PARAMETERS,
            deterministic: decapsulation,
            reversible: Some(false),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "key_encapsulation",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "FIPS 203 raw KEM only. Correct-width invalid ciphertexts use implicit rejection. Treat every shared secret and expanded private key as private data.",
                private_input: decapsulation,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Binary FIPS 203 wire encodings.",
                key: "ML-KEM-512/768/1024 use 800/1184/1568-byte public keys and 1632/2400/3168-byte expanded private keys.",
                key_advancement: "One operation with an explicit parameter set.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "One key pair or ciphertext per call. Ciphertext widths 768/1088/1568 bytes. Shared secrets are exactly 32 bytes. Direct execution requires private retention consent in Play.",
            rust_api: api.0,
            python_api: api.1,
            references: &["https://csrc.nist.gov/pubs/fips/203/final"],
        }),
    }
}

const PROFILED: [m::Entry; 3] = [
    row(
        KEYGEN,
        "Generate ML-KEM key pair",
        "Generate a fresh FIPS 203 public key and expanded private key.",
        "cipher ml-kem keygen",
        ("cube_core::asymmetric::ml_kem::keygen", "ml_kem_keygen"),
        "cube cipher ml-kem keygen --parameters-file settings.json",
        false,
    ),
    row(
        ENCAPSULATE,
        "Encapsulate ML-KEM shared secret",
        "Encapsulate a fresh shared secret with a canonical FIPS 203 public key.",
        "cipher ml-kem encapsulate",
        (
            "cube_core::asymmetric::ml_kem::encapsulate",
            "ml_kem_encapsulate",
        ),
        "cube cipher ml-kem encapsulate --parameters-file settings.json --key-file public.bin",
        false,
    ),
    row(
        DECAPSULATE,
        "Decapsulate ML-KEM shared secret",
        "Decapsulate a FIPS 203 ciphertext with an expanded private key.",
        "cipher ml-kem decapsulate",
        (
            "cube_core::asymmetric::ml_kem::decapsulate",
            "ml_kem_decapsulate",
        ),
        "cube cipher ml-kem decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        true,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

const PUBLIC_KEY: c::Port = port("public_key", byte_schema::<800, MAX_PUBLIC_KEY>);
const PRIVATE_KEY: c::Port = port("private_key", byte_schema::<1632, MAX_PRIVATE_KEY>);
const CIPHERTEXT: c::Port = port("ciphertext", byte_schema::<768, MAX_CIPHERTEXT>);
const SHARED_SECRET: c::Port = port(
    "shared_secret",
    byte_schema::<SHARED_SECRET_BYTES, SHARED_SECRET_BYTES>,
);

const fn contract(
    index: usize,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    input_bytes: usize,
    output_bytes: usize,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        parameters: c::schema::<Parameters>,
        outputs,
        limits: c::Limits {
            max_input_bytes: Some(input_bytes as u64),
            max_output_bytes: Some(output_bytes as u64),
            max_items: Some(1),
            max_work: Some(1),
            max_memory_bytes: Some(MEMORY_RESERVATION as u64),
        },
        fixture,
    }
}

const CONTRACTS: &[c::Entry] = &[
    contract(
        0,
        &[],
        &[PUBLIC_KEY, PRIVATE_KEY],
        0,
        MAX_PUBLIC_KEY + MAX_PRIVATE_KEY,
        keygen_fixture,
    ),
    contract(
        1,
        &[PUBLIC_KEY],
        &[CIPHERTEXT, SHARED_SECRET],
        MAX_PUBLIC_KEY,
        MAX_CIPHERTEXT + SHARED_SECRET_BYTES,
        encapsulate_fixture,
    ),
    contract(
        2,
        &[CIPHERTEXT, PRIVATE_KEY],
        &[SHARED_SECRET],
        MAX_CIPHERTEXT + MAX_PRIVATE_KEY,
        SHARED_SECRET_BYTES,
        decapsulate_fixture,
    ),
];

pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);

fn vector() -> Value {
    let mut vectors: Value =
        serde_json::from_str(include_str!("vectors.json")).expect("pinned ACVP samples");
    vectors["profiles"][0].take()
}

fn bytes(v: &Value, key: &str) -> Value {
    let data = hex::decode(v[key].as_str().expect("ACVP vector field")).expect("ACVP hex field");
    json!({"base64":crate::encoding::base64_encode(&data)})
}

fn keygen_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let k = &v["keygen"];
    Ok(c::Fixture {
        inputs: json!({}),
        parameters: json!({"profile":v["profile"]}),
        outputs: json!({"public_key":bytes(k,"ek"),"private_key":bytes(k,"dk")}),
    })
}

fn encapsulate_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let k = &v["encapsulation"];
    Ok(c::Fixture {
        inputs: json!({"public_key":bytes(k,"ek")}),
        parameters: json!({"profile":v["profile"]}),
        outputs: json!({"ciphertext":bytes(k,"c"),"shared_secret":bytes(k,"k")}),
    })
}

fn decapsulate_fixture() -> Result<c::Fixture, OperationError> {
    let v = vector();
    let k = &v["encapsulation"];
    let key = hex::decode(k["dk"].as_str().unwrap()).unwrap();
    let ct = hex::decode(k["c"].as_str().unwrap()).unwrap();
    let profile = Profile::parse(&json!({"profile": v["profile"]}).to_string())?;
    let output = decapsulate(&ct, &key, profile)?;
    assert!(hex::encode(&output[..]).eq_ignore_ascii_case(k["k"].as_str().unwrap()));
    Ok(c::Fixture {
        inputs: json!({"ciphertext":bytes(k,"c"),"private_key":bytes(k,"dk")}),
        parameters: json!({"profile":v["profile"]}),
        outputs: json!({"shared_secret":bytes(k,"k")}),
    })
}
