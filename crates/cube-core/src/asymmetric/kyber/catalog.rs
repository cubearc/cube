use super::*;
use crate::{
    asymmetric::{
        kem_wire::{byte_port, byte_schema},
        ml_kem::MEMORY_RESERVATION,
    },
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::{Value, json};

const DESCRIPTION: &str = "Decapsulate historical CRYSTALS-Kyber 3.0 Round 3 SHAKE ciphertexts using an explicit raw key profile.";

const PROFILED: [m::Entry; 1] = [m::Entry {
    descriptor: o::Descriptor {
        id: DECAPSULATE,
        category: "ciphers",
        description: DESCRIPTION,
        input_type: "json",
        output_type: "json",
        parameters: &[o::Parameter {
            name: "profile",
            description: "Required Kyber Round 3 SHAKE parameter set.",
            required: true,
            default: None,
            constraints: o::ParameterType::Text {
                choices: Profile::NAMES,
            },
        }],
        deterministic: true,
        reversible: Some(false),
        command: "cipher kyber decapsulate",
        example: "cube cipher kyber decapsulate --parameters-file settings.json --key-file legacy.bin --file ciphertext.bin",
    },
    metadata: Some(m::Metadata {
        schema_version: 1,
        canonical_name: "Decapsulate Kyber Round 3 shared secret",
        family: "key_encapsulation",
        aliases: &[],
        security: m::Security {
            classification: m::Classification::PrivateMaterial,
            limitations: "Historical compatibility only. Use the specified Kyber version. Correct-width ciphertexts use implicit rejection without a validity flag.",
            private_input: true,
            private_output: true,
            sensitive_parameters: &[],
        },
        profile: m::Profile {
            variant: DESCRIPTION,
            alphabet: "Binary Kyber Round 3 encodings.",
            key: "Round 3 SHAKE private key widths: 1632, 2400 or 3168 bytes, selected by the explicit profile.",
            key_advancement: "One decapsulation. Separate from FIPS 203 profiles.",
            block_bits: None,
            output_bits: None,
        },
        resources: &[],
        resource_notes: "Ciphertext widths 768, 1088 or 1568 bytes. Shared secrets are 32 bytes. Direct private retention requires consent in Play.",
        rust_api: "cube_core::asymmetric::kyber::decapsulate",
        python_api: "kyber_decapsulate",
        references: &["https://github.com/pq-crystals/kyber/tree/v3.0"],
    }),
}];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[
        byte_port("ciphertext", byte_schema::<768, MAX_CIPHERTEXT>),
        byte_port("private_key", byte_schema::<1632, MAX_PRIVATE_KEY>),
    ],
    parameters: c::schema::<Parameters>,
    outputs: &[byte_port("shared_secret", byte_schema::<32, 32>)],
    limits: c::Limits {
        max_input_bytes: Some((MAX_PRIVATE_KEY + MAX_CIPHERTEXT) as u64),
        max_output_bytes: Some(SHARED_SECRET_BYTES as u64),
        max_items: Some(1),
        max_work: Some(1),
        max_memory_bytes: Some(MEMORY_RESERVATION as u64),
    },
    fixture,
}];

pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);

fn fixture() -> Result<c::Fixture, OperationError> {
    let v: Value =
        serde_json::from_str(include_str!("vectors.json")).expect("Round 3 reference output");
    let case = &v["profiles"][0];
    let raw = |field: &str| hex::decode(case[field].as_str().unwrap()).unwrap();
    let ciphertext = raw("ciphertext");
    let private_key = raw("dk");
    let expected = raw("shared_secret");
    let actual = decapsulate(&ciphertext, &private_key, Profile::Kyber512)?;
    assert!(actual[..] == expected);
    Ok(c::Fixture {
        inputs: json!({"ciphertext":c::values::Bytes{data:ciphertext},"private_key":c::values::Bytes{data:private_key}}),
        parameters: json!({"profile":case["profile"]}),
        outputs: json!({"shared_secret":c::values::Bytes{data:expected}}),
    })
}
