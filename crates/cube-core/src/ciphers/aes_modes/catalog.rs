use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "Explicit unauthenticated AES byte modes, NIST CBC-CS1/2/3 and IEEE1619 XTS or full-block XEX data units.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "mode",
        description: "Required exact byte, ciphertext-stealing or storage construction; no inferred mode.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &[
                "ecb",
                "cbc",
                "cfb128",
                "ofb",
                "ctr128_be",
                "cbc_cs1",
                "cbc_cs2",
                "cbc_cs3",
                "xts_aes128",
                "xts_aes256",
                "xex_aes128_ieee1619_full_blocks",
                "xex_aes256_ieee1619_full_blocks",
            ],
        },
    },
    o::Parameter {
        name: "key_hex",
        description: "Required private AES key, or32/64-byte combined storage key with distinct halves, as hexadecimal.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "iv_hex",
        description: "Required16-byte IV/counter for CBC/feedback/CTR/CTS; forbidden for ECB/storage.",
        required: false,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "tweak_hex",
        description: "Required16-byte initial tweak input for XTS/XEX; forbidden for other modes.",
        required: false,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "padding",
        description: "Required none or pkcs7 for ECB/CBC; forbidden for all other modes.",
        required: false,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["none", "pkcs7"],
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
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "AES byte modes",
            family: "modern_block",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Unauthenticated low-level interoperability only. ECB reveals repetition; CBC padding is not authentication. Decryption never verifies a key or integrity. CTS and XTS/XEX preserve length but remain malleable. No secure message-encryption claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "16/24/32 AES key bytes; XTS/XEX require32/64 combined bytes with unequal halves. Exact private hexadecimal.",
                key_advancement: "CBC/CFB128/OFB/CTS use an explicit16-byte IV; CTR increments the full128-bit big-endian state. XTS/XEX encrypt an explicit16-byte initial tweak under the second key, then advance in little-endian GF(2^128). No RNG.",
                block_bits: Some(128),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Core body <=16 MiB plus at most one padding block. CTS/storage need at least16 bytes; full-block XEX rejects partial blocks. A data unit contains at most2^20 AES blocks. Parameter JSON <=2048 bytes. Output allocation is bounded by input plus 16 bytes and fixed backend state. Expanded keys and owned decoded key buffers wiped on drop; caller, parser, OS and interpreter copies are not guaranteed erased. No pipeline or automatic capture. Play transport limits may be lower.",
            rust_api: api,
            python_api,
            references: &[
                "https://csrc.nist.gov/pubs/fips/197/final",
                "https://docs.rs/aes/0.9.3/aes/",
                "https://csrc.nist.gov/pubs/sp/800/38/a/sup/final",
                "https://csrc.nist.gov/pubs/sp/800/38/e/final",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher aes mode encrypt",
        "cube_core::ciphers::aes_modes::encrypt",
        "aes_mode_encrypt",
        "cube cipher aes mode encrypt --parameters-file settings.json --file input.bin",
    ),
    row(
        DECRYPT,
        "cipher aes mode decrypt",
        "cube_core::ciphers::aes_modes::decrypt",
        "aes_mode_decrypt",
        "cube cipher aes mode decrypt --parameters-file settings.json --file input.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
// Keep the shared secret field visible to contract privacy annotations while
// retaining each variant's complete derived mode-specific schema.
fn parameters_schema() -> schemars::Schema {
    let mut schema = c::schema::<Parameters>().to_value();
    schema["properties"] =
        serde_json::json!({"mode":{"type":"string"},"key_hex":common_key_schema()});
    schema["required"] = serde_json::json!(["mode", "key_hex"]);
    schemars::Schema::try_from(schema).expect("typed mode union with common private key")
}
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] =
        serde_json::json!((MAX_INPUT + 16).div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_INPUT + 16);
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
        parameters: parameters_schema,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some((MAX_INPUT + if index == 1 { 16 } else { 0 }) as u64),
            max_output_bytes: Some((MAX_INPUT + if index == 0 { 16 } else { 0 }) as u64),
            max_items: Some((MAX_INPUT / 16 + 1) as u64),
            max_work: Some((MAX_INPUT / 16 + 1) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let plain = hex::decode("6bc1bee22e409f96e93d7e117393172a").unwrap();
    let encrypted = hex::decode("7649abac8119b246cee98e9b12e9197d").unwrap();
    let key = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
    let parameters = serde_json::json!({"mode":"cbc","padding":"none","key_hex":hex::encode(&key),"iv_hex":"000102030405060708090a0b0c0d0e0f"});
    let settings = Settings::parse(&parameters.to_string())?;
    let (input, expected) = if decrypting {
        (encrypted, plain)
    } else {
        (plain, encrypted)
    };
    let actual = if decrypting {
        decrypt(&input, &settings)?
    } else {
        encrypt(&input, &settings)?
    };
    assert_eq!(actual, expected);
    Ok(c::Fixture {
        inputs: serde_json::json!({"input": c::values::Bytes { data: input }}),
        parameters,
        outputs: serde_json::json!({"output": c::values::Bytes { data: actual }}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
