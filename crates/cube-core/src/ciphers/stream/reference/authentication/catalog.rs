use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;
const DESCRIPTION: &str =
    "Original SKEW2011 Grain128a encryption with detached1..32-bit plaintext authentication.";
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact authenticated construction; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["grain128a_skew2011_msb_v1"],
        },
    },
    o::Parameter {
        name: "key_hex",
        description: "Required private128-bit key as32 hexadecimal digits, MSB-first octets.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "nonce_hex",
        description: "Required96-bit IV as24 hexadecimal digits, MSB-first octets.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "data_bits",
        description: "Exact MSB-packed input bit count; unused tail bits must be zero.",
        required: true,
        default: None,
        constraints: o::ParameterType::Unsigned {
            min: 0,
            max: (MAX_INPUT * 8) as u64,
        },
    },
    o::Parameter {
        name: "tag_bits",
        description: "Required1..32-bit tag: rightmost bits of full MAC, packed MSB-first with zero unused bits.",
        required: true,
        default: None,
        constraints: o::ParameterType::Unsigned { min: 1, max: 32 },
    },
];
const fn row(
    id: &'static str,
    command: &'static str,
    rust_api: &'static str,
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
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "Grain128a detached authentication",
            family: "stream_ciphers",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Historical SKEW2011 MAC has at most32 authentication bits; shorter tags have correspondingly weaker forgery resistance. No independent audit or constant-time guarantee. This is not the later Grain128AEAD.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact MSB-packed bits, unused bits zero.",
                key: "Required16-byte key and12-byte IV; no KDF or defaults.",
                key_advancement: "Fresh full256-clock setup plus64 initial pre-output bits on each call. Decimated keystream; MAC over plaintext bits and mandatory terminal1bit. No key/nonce reuse tracking.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "At most64KiB packed data,4-byte detached tag and4096-byte private settings. data_bits and tag_bits are mandatory. No associated data or encrypted padding. Decryption verifies before returning plaintext. Owned key/nonce/state and failed tentative plaintext are wiped; caller/parser/SDK/OS copies are not guaranteed erased. No capture or linear recipe adapter.",
            rust_api,
            python_api,
            references: &["https://lucris.lub.lu.se/ws/portalfiles/portal/6156802/1981684.pdf"],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher stream auth encrypt",
        "cube_core::ciphers::stream::reference::authentication::encrypt",
        "stream_auth_encrypt",
        "cube cipher stream auth encrypt --parameters-file settings.json --file message.bits",
    ),
    row(
        DECRYPT,
        "cipher stream auth decrypt",
        "cube_core::ciphers::stream::reference::authentication::decrypt",
        "stream_auth_decrypt",
        "cube cipher stream auth decrypt --parameters-file settings.json --file encrypted.bits --tag-file tag.bits",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn parameters() -> schemars::Schema {
    c::schema::<Parameters>()
}
fn bounded_bytes(maximum: usize) -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(maximum.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = json!(maximum);
    schemars::Schema::try_from(schema).expect("bounded authenticated bytes")
}
fn data_schema() -> schemars::Schema {
    bounded_bytes(MAX_INPUT)
}
fn tag_schema() -> schemars::Schema {
    let mut schema = bounded_bytes(MAX_TAG_BYTES).to_value();
    schema["properties"]["base64"]["minLength"] = json!(4);
    schemars::Schema::try_from(schema).expect("nonempty detached tag")
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: data_schema,
    required: true,
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const TAG: c::Port = c::Port {
    name: "tag",
    semantic_type: "bytes",
    schema: tag_schema,
    required: true,
};
const CONTRACTS: &[c::Entry] = &[
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[0],
        inputs: &[INPUT],
        parameters,
        outputs: &[OUTPUT, TAG],
        limits: limits(false),
        fixture: encrypt_fixture,
    },
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[1],
        inputs: &[INPUT, TAG],
        parameters,
        outputs: &[OUTPUT],
        limits: limits(true),
        fixture: decrypt_fixture,
    },
];
const fn limits(decrypting: bool) -> c::Limits {
    c::Limits {
        max_input_bytes: Some((MAX_INPUT + if decrypting { MAX_TAG_BYTES } else { 0 }) as u64),
        max_output_bytes: Some((MAX_INPUT + if decrypting { 0 } else { MAX_TAG_BYTES }) as u64),
        max_items: Some((MAX_INPUT * 8 + 32) as u64),
        max_work: Some((MAX_INPUT * 16 + 320) as u64),
        max_memory_bytes: None,
    }
}
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let vector: serde_json::Value = serde_json::from_str(include_str!("vectors.json"))
        .expect("independent authenticated vectors");
    let v = &vector[4];
    let parameters = v["parameters"].clone();
    let settings = Settings::parse(&parameters.to_string())?;
    let plain = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    let ciphertext = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let tag = hex::decode(v["tag_hex"].as_str().unwrap()).unwrap();
    if decrypting {
        let output = decrypt(&ciphertext, &tag, &settings)?;
        assert_eq!(output, plain);
        Ok(c::Fixture {
            inputs: json!({"input":c::values::Bytes{data:ciphertext},"tag":c::values::Bytes{data:tag}}),
            parameters,
            outputs: json!({"output":c::values::Bytes{data:output}}),
        })
    } else {
        let output = encrypt(&plain, &settings)?;
        assert_eq!(output.ciphertext, ciphertext);
        assert_eq!(output.tag, tag);
        Ok(c::Fixture {
            inputs: json!({"input":c::values::Bytes{data:plain}}),
            parameters,
            outputs: json!({"output":c::values::Bytes{data:output.ciphertext},"tag":c::values::Bytes{data:output.tag}}),
        })
    }
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
