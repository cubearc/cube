use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const EL_DESCRIPTION: &str = "Raw ElGamal in the RFC7919 ffdhe2048 prime-order subgroup; exact integer group messages, fresh uniform coins, no authentication.";
const CS_DESCRIPTION: &str = "Raw Cramer-Shoup in the RFC7919 ffdhe2048 subgroup with pinned second generator and domain-bound fixed-width SHA256 transcript and label.";
const EL_PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "profile",
    description: "Required exact ElGamal raw group profile.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text {
        choices: &["elgamal_ffdhe2048_raw_v1"],
    },
}];
const CS_PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required complete Cramer-Shoup raw group/hash profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["cramer_shoup_ffdhe2048_sha256_raw_v1"],
        },
    },
    o::Parameter {
        name: "label_hex",
        description: "Optional binary transcript label, <=256 bytes in hexadecimal; default empty.",
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
    cs: bool,
    randomized: bool,
) -> m::Entry {
    let description = if cs { CS_DESCRIPTION } else { EL_DESCRIPTION };
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description,
            input_type: "bytes",
            output_type: "bytes",
            parameters: if cs { CS_PARAMETERS } else { EL_PARAMETERS },
            deterministic: !randomized,
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: if cs {
                "Raw Cramer-Shoup encryption"
            } else {
                "Raw ElGamal encryption"
            },
            family: "asymmetric_encryption",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: if cs {
                    "Complete explicit raw group profile, not a universal standardized ciphertext format. Ciphertext verification precedes output; no sender identity or audit/certification claim. Input is a subgroup element, not arbitrary bytes, files or text; there is no implicit embedding or padding."
                } else {
                    "Raw ElGamal is malleable and unauthenticated: successful output does not establish integrity or sender identity. Input is a prime-order subgroup element, not arbitrary bytes, files or text; no implicit embedding, padding or audit/certification claim."
                },
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exactly256-byte big-endian group messages, retaining leading zeroes. Identity message1 is permitted.",
                key: if cs {
                    "Public c||d||h:768 bytes. Private x1||x2||y1||y2||z:1280 bytes, each scalar in1..q-1. g1=2; g2 pinned by the documented SHAKE256 public derivation."
                } else {
                    "Public y:256 bytes. Private x:256 bytes in1..q-1. RFC7919 ffdhe2048 p,prime order q=(p-1)/2,g=2; no arbitrary groups."
                },
                key_advancement: if cs {
                    "Fresh uniform r in1..q-1. Cipher u1||u2||e||v, four256-byte fields. alpha=SHA256(profile domain||fixed(u1)||fixed(u2)||fixed(e)||label); private verification equation is checked in constant time."
                } else {
                    "Fresh uniform r in1..q-1. Cipher u||e, two256-byte fields. No authentication, text guesses or deterministic production coins."
                },
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "One bounded raw group operation. Key <=1280 bytes, JSON <=1024 bytes, input/ciphertext <=1024 bytes. Fixed2048-bit constant-time backend secret exponentiation; at most50000 exponent rounds including validation and private-key public derivation. Ephemeral generation <=8 draws/2048 entropy bytes. Range and subgroup checks reject0,p-1,p,nonmembers and nonidentity-required fields; e/v may equal1. Owned private scalars/intermediates wiped on drop; backend/caller/interpreter copies are not guaranteed erased. No pipeline/automatic capture; Play requires explicit private retention consent.",
            rust_api: api,
            python_api: python,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc7919",
                "https://www.shoup.net/papers/cs.pdf",
                "https://cacr.uwaterloo.ca/hac/about/chap8.pdf",
                "https://docs.rs/crypto-bigint/0.7.11/crypto_bigint/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 4] = [
    row(
        ELGAMAL_ENCRYPT,
        "cipher elgamal encrypt",
        "cube_core::asymmetric::group_encryption::elgamal_encrypt",
        "elgamal_encrypt",
        "cube cipher elgamal encrypt --parameters-file settings.json --key-file public.bin --file group.bin",
        false,
        true,
    ),
    row(
        ELGAMAL_DECRYPT,
        "cipher elgamal decrypt",
        "cube_core::asymmetric::group_encryption::elgamal_decrypt",
        "elgamal_decrypt",
        "cube cipher elgamal decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        false,
        false,
    ),
    row(
        CRAMER_SHOUP_ENCRYPT,
        "cipher cramer-shoup encrypt",
        "cube_core::asymmetric::group_encryption::cramer_shoup_encrypt",
        "cramer_shoup_encrypt",
        "cube cipher cramer-shoup encrypt --parameters-file settings.json --key-file public.bin --file group.bin",
        true,
        true,
    ),
    row(
        CRAMER_SHOUP_DECRYPT,
        "cipher cramer-shoup decrypt",
        "cube_core::asymmetric::group_encryption::cramer_shoup_decrypt",
        "cramer_shoup_decrypt",
        "cube cipher cramer-shoup decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        true,
        false,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bounded_bytes(max: usize) -> schemars::Schema {
    let mut s = c::schema::<c::values::Bytes>().to_value();
    s["properties"]["base64"]["maxLength"] = serde_json::json!(max.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(max);
    schemars::Schema::try_from(s).unwrap()
}
fn input_schema() -> schemars::Schema {
    bounded_bytes(MAX_CIPHERTEXT)
}
fn key_schema() -> schemars::Schema {
    bounded_bytes(MAX_KEY)
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: input_schema,
    required: true,
};
const KEY: c::Port = c::Port {
    name: "key",
    schema: key_schema,
    ..INPUT
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const fn entry(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[INPUT, KEY],
        parameters: if index < 2 {
            c::schema::<ElGamalParameters>
        } else {
            c::schema::<CramerShoupParameters>
        },
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some((MAX_KEY + MAX_CIPHERTEXT) as u64),
            max_output_bytes: Some(if index % 2 == 1 {
                256
            } else if index == 0 {
                512
            } else {
                1024
            }),
            max_items: Some(1),
            max_work: Some(50000),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    entry(0, el_encrypt_fixture),
    entry(1, el_decrypt_fixture),
    entry(2, cs_encrypt_fixture),
    entry(3, cs_decrypt_fixture),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(index: usize, encrypt: bool) -> Result<c::Fixture, OperationError> {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &rows[index];
    let parameters = v["parameters"].clone();
    let doc = parameters.to_string();
    let private = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let public = hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap();
    let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    let cipher = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let cs = index >= 3;
    let (input, key, output) = if encrypt {
        let output = if cs {
            cramer_shoup_encrypt(
                &plain,
                &CramerShoupEncryptionSettings::parse(&doc, &public)?,
            )?
        } else {
            elgamal_encrypt(&plain, &ElGamalEncryptionSettings::parse(&doc, &public)?)?
        };
        let recovered = if cs {
            cramer_shoup_decrypt(
                &output,
                &CramerShoupDecryptionSettings::parse(&doc, &private)?,
            )?
        } else {
            elgamal_decrypt(&output, &ElGamalDecryptionSettings::parse(&doc, &private)?)?
        };
        assert_eq!(recovered, plain);
        (plain, public, output)
    } else {
        let output = if cs {
            cramer_shoup_decrypt(
                &cipher,
                &CramerShoupDecryptionSettings::parse(&doc, &private)?,
            )?
        } else {
            elgamal_decrypt(&cipher, &ElGamalDecryptionSettings::parse(&doc, &private)?)?
        };
        assert_eq!(output, plain);
        (cipher, private, output)
    };
    Ok(c::Fixture {
        inputs: serde_json::json!({"input":c::values::Bytes{data:input},"key":c::values::Bytes{data:key}}),
        parameters,
        outputs: serde_json::json!({"output":c::values::Bytes{data:output}}),
    })
}
fn el_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(1, true)
}
fn el_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(1, false)
}
fn cs_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(5, true)
}
fn cs_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(5, false)
}
