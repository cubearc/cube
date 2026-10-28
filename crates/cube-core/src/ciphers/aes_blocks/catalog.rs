use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str =
    "Raw FIPS 197 AES whole blocks; no padding, nonce, authentication or message format.";

const PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "key_hex",
    description: "Required exact AES key: 32, 48 or 64 hexadecimal digits; no separators or prefix.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
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
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "AES raw blocks",
            family: "modern_block",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::RawBlockCipher,
                limitations: "Unauthenticated low-level interoperability only. Equal blocks reveal repetition. Decryption never verifies a key or integrity. No secure message-encryption claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "16/24/32 caller-supplied bytes; typed JSON uses exact hexadecimal, CLI uses a raw key file.",
                key_advancement: "Independent blocks under the same key; no IV, counter, padding or RNG.",
                block_bits: Some(128),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Core input/output <=16 MiB, whole 16-byte blocks; empty identity with a valid key. Parameter JSON <=1024 bytes. Output allocation equals input size plus fixed backend state. Expanded keys and owned decoded key buffers wiped on drop; caller, parser, OS and interpreter copies are not guaranteed erased. No pipeline or automatic capture. Play transport limits may be lower.",
            rust_api: api,
            python_api,
            references: &[
                "https://csrc.nist.gov/pubs/fips/197/final",
                "https://docs.rs/aes/0.9.3/aes/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher aes encrypt",
        "cube_core::ciphers::aes_blocks::encrypt",
        "aes_encrypt_blocks",
        "cube cipher aes encrypt --key-file key.bin --file blocks.bin",
    ),
    row(
        DECRYPT,
        "cipher aes decrypt",
        "cube_core::ciphers::aes_blocks::decrypt",
        "aes_decrypt_blocks",
        "cube cipher aes decrypt --key-file key.bin --file blocks.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_INPUT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_INPUT);
    schema["properties"]["base64"]["pattern"] = serde_json::json!(
        "^(?:[A-Za-z0-9+/]{64})*(?:[A-Za-z0-9+/]{21}[AQgw]==|[A-Za-z0-9+/]{42}[AEIMQUYcgkosw048]=)?$"
    );
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
const fn contract(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[PORT],
        parameters: c::schema::<Parameters>,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(MAX_INPUT as u64),
            max_items: Some((MAX_INPUT / 16) as u64),
            max_work: Some((MAX_INPUT / 16) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let plain = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    let encrypted = hex::decode("69c4e0d86a7b0430d8cdb78070b4c55a").unwrap();
    let key = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
    let (input, expected) = if decrypting {
        (encrypted, plain)
    } else {
        (plain, encrypted)
    };
    let actual = if decrypting {
        decrypt(&input, &key)?
    } else {
        encrypt(&input, &key)?
    };
    assert_eq!(actual, expected);
    Ok(c::Fixture {
        inputs: serde_json::json!({"input": c::values::Bytes { data: input }}),
        parameters: serde_json::json!({"key_hex": hex::encode(key)}),
        outputs: serde_json::json!({"output": c::values::Bytes { data: actual }}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
