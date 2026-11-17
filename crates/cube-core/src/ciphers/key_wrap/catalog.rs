use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str = "RFC3394 AES-KW or RFC5649 AES-KWP with fixed integrity checks and no unauthenticated key release.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact AES-KW or AES-KWP profile; no IV, KDF or default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "key_hex",
        description: "Required exact AES key: 32, 48 or 64 hexadecimal digits; no separators or prefix.",
        required: true,
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
            canonical_name: "AES key wrapping",
            family: "key_wrapping",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Deterministic key wrapping only, not a general message-encryption envelope. KW has a64-bit integrity field; caller manages KEK security and key usage. No certification or independent audit claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "Required16/24/32-byte KEK; private typed settings use exact hexadecimal.",
                key_advancement: "Fixed RFC3394 IV or RFC5649 AIV/MLI and zero padding; no caller IV, RNG or KDF.",
                block_bits: Some(128),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Key data <=65536 bytes, KW at least16 and multiple8, KWP at least1. Wrapped output <=65544 bytes. Settings <=1024 UTF-8 bytes. At most49152 AES wrapping steps; fixed-size backend state and bounded wiped temporary key buffers. Unwrap checks integrity before release. Expanded keys and owned decoded key buffers wiped on drop; caller, parser, OS and interpreter copies are not guaranteed erased. No pipeline or automatic capture. Play transport limits may be lower.",
            rust_api: api,
            python_api,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc3394",
                "https://www.rfc-editor.org/rfc/rfc5649",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        WRAP,
        "cipher aes key-wrap wrap",
        "cube_core::ciphers::key_wrap::wrap",
        "aes_key_wrap",
        "cube cipher aes key-wrap wrap --parameters-file settings.json --file key.bin",
    ),
    row(
        UNWRAP,
        "cipher aes key-wrap unwrap",
        "cube_core::ciphers::key_wrap::unwrap",
        "aes_key_unwrap",
        "cube cipher aes key-wrap unwrap --parameters-file settings.json --file wrapped.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_WRAPPED.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_WRAPPED);
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
            max_input_bytes: Some(if index == 1 {
                MAX_WRAPPED as u64
            } else {
                MAX_INPUT as u64
            }),
            max_output_bytes: Some(if index == 0 {
                MAX_WRAPPED as u64
            } else {
                MAX_INPUT as u64
            }),
            max_items: Some((MAX_INPUT / 8) as u64),
            max_work: Some((MAX_INPUT / 8 * 6) as u64),
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
    let encrypted = hex::decode("1fa68b0a8112b447aef34bd8fb5a7b829d3e862371d2cfe5").unwrap();
    let key = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
    let parameters = serde_json::json!({"profile":"aes_kw_rfc3394","key_hex":hex::encode(&key)});
    let settings = Settings::parse(&parameters.to_string())?;
    let (input, expected) = if decrypting {
        (encrypted, plain)
    } else {
        (plain, encrypted)
    };
    let actual = if decrypting {
        unwrap(&input, &settings)?
    } else {
        wrap(&input, &settings)?
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
