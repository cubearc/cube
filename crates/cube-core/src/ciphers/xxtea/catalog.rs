use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;
const DESCRIPTION: &str = "Raw Corrected Block TEA over one complete little-endian word sequence; no padding, length suffix or authentication.";
const PARAMETERS: &[o::Parameter] = &[o::Parameter {
    name: "key_hex",
    description: "Required private exact sixteen-byte hexadecimal key; no normalization or default.",
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
            canonical_name: "XXTEA raw word sequence",
            family: "modern_block",
            aliases: &[
                m::Alias {
                    name: "XXTEA",
                    usage: "Raw full little-endian word sequence; no string/length wrapper.",
                },
                m::Alias {
                    name: "Corrected Block TEA",
                    usage: "The complete input is one variable-width block.",
                },
            ],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Unaudited educational reference; no constant-time guarantee or secure message-encryption claim. Wrong valid keys produce bytes without integrity verification.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes interpreted as little-endian 32-bit words.",
                key: "Exactly sixteen caller-supplied bytes, four little-endian words; no short-key padding or string conversion.",
                key_advancement: "The entire sequence is one variable-width block; exactly 6 + floor(52 / word_count) cycles. No independent eight-byte chunks.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "At most 64 KiB input/output; empty identity after key validation, otherwise 2..16384 whole32-bit words. At most4096 private JSON bytes. One word buffer and an equally sized output, bounded round work. Owned key/word buffers wiped on drop; borrowed/parser/SDK/OS copies are outside the guarantee. No pipeline or automatic capture.",
            rust_api: api,
            python_api,
            references: &[
                "https://www.cl.cam.ac.uk/ftp/papers/djw-rmn/djw-rmn-tea.html",
                "https://github.com/xxtea/xxtea-c/tree/7ec961540996934d939572d885ea1d5b21689688",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher xxtea encrypt",
        "cube_core::ciphers::xxtea::encrypt",
        "xxtea_encrypt_words",
        "cube cipher xxtea encrypt --key-file key.bin --file words.bin",
    ),
    row(
        DECRYPT,
        "cipher xxtea decrypt",
        "cube_core::ciphers::xxtea::decrypt",
        "xxtea_decrypt_words",
        "cube cipher xxtea decrypt --key-file key.bin --file words.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(MAX_INPUT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = json!(MAX_INPUT);
    schemars::Schema::try_from(schema).expect("bounded XXTEA byte transport")
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: bytes_schema,
    required: true,
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const fn contract(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[INPUT],
        parameters: c::schema::<Parameters>,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(MAX_INPUT as u64),
            max_items: Some((MAX_INPUT / 4) as u64),
            max_work: Some((6 * (MAX_INPUT / 4) + 52) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(inverse: bool) -> Result<c::Fixture, OperationError> {
    let plain = vec![0u8; 8];
    let encrypted = hex::decode("ab043705808c5d57").unwrap();
    let key = Key::new(&[0; 16])?;
    let (input, expected) = if inverse {
        (encrypted, plain)
    } else {
        (plain, encrypted)
    };
    let output = if inverse {
        decrypt(&input, &key)?
    } else {
        encrypt(&input, &key)?
    };
    assert_eq!(output, expected);
    Ok(c::Fixture {
        inputs: json!({"input":c::values::Bytes{data:input}}),
        parameters: json!({"key_hex":"00".repeat(16)}),
        outputs: json!({"output":c::values::Bytes{data:output}}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
