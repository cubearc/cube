use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct ChaChaParameters {
    nonce: [u8; 12],
    counter: u32,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct XChaChaParameters {
    nonce: [u8; 24],
    counter: u32,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SalsaParameters {
    nonce: [u8; 8],
    counter: u64,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Bytes {
    #[schemars(length(max=22369624), extend("contentEncoding"="base64"))]
    base64: String,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Key {
    #[schemars(length(min=44,max=44), extend("contentEncoding"="base64"))]
    base64: String,
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: c::schema::<Bytes>,
    required: true,
};
const KEY: c::Port = c::Port {
    name: "key",
    semantic_type: "bytes",
    schema: c::schema::<Key>,
    required: true,
};
const PAD: c::Port = c::Port {
    name: "key",
    semantic_type: "bytes",
    schema: c::schema::<Bytes>,
    required: true,
};
const OUTPUT: &[c::Port] = &[c::Port {
    name: "output",
    semantic_type: "bytes",
    schema: c::schema::<Bytes>,
    required: true,
}];
const fn parameters(p: Profile) -> [o::Parameter; 2] {
    [
        o::Parameter {
            name: "nonce",
            description: "Required exact nonce bytes; never generated or defaulted.",
            required: true,
            default: None,
            constraints: o::ParameterType::UnsignedList {
                min: 0,
                max: 255,
                min_length: p.nonce_bytes(),
                max_length: p.nonce_bytes(),
                unique: false,
            },
        },
        o::Parameter {
            name: "counter",
            description: "Required initial 64-byte block counter; preflight includes partial final blocks.",
            required: true,
            default: None,
            constraints: o::ParameterType::Unsigned {
                min: 0,
                max: p.max_counter(),
            },
        },
    ]
}
const fn declaration(
    id: &'static str,
    name: &'static str,
    command: &'static str,
    example: &'static str,
    parameters: &'static [o::Parameter],
    python_api: &'static str,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: name,
            input_type: "bytes",
            output_type: "bytes",
            parameters,
            deterministic: true,
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "stream_ciphers",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Unauthenticated and malleable; caller must prevent key/nonce/position reuse. Reduced rounds are experimental. Salsa backend has no independent audit or complete timing assessment. Vernam is educational: uniform independent one-use pad delivery is external.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: name,
                alphabet: "Exact bytes; no text interpretation.",
                key: "Explicit 32-byte stream key, or Vernam pad exactly equal to message length.",
                key_advancement: "64-byte blocks with explicit initial counter; Vernam consumes each pad byte once within this call.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[m::ResourceBound {
                name: "input_bytes",
                maximum: MAX_INPUT as u64,
                unit: "bytes",
                scope: m::Scope::Core,
            }],
            resource_notes: "Each operand/output <=16 MiB. Counter preflight precedes allocation. Linear bounded work, no network/randomness/pad storage. Direct-only; private inputs and outputs are not captured automatically.",
            rust_api: "cube_core::ciphers::stream",
            python_api,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc8439",
                "https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03",
                "https://cr.yp.to/snuffle/spec.pdf",
            ],
        }),
    }
}
const CHACHA8_PARAMETERS: &[o::Parameter] = &parameters(Profile::ChaCha8);
const CHACHA12_PARAMETERS: &[o::Parameter] = &parameters(Profile::ChaCha12);
const CHACHA20_PARAMETERS: &[o::Parameter] = &parameters(Profile::ChaCha20);
const XCHACHA20_PARAMETERS: &[o::Parameter] = &parameters(Profile::XChaCha20);
const SALSA12_PARAMETERS: &[o::Parameter] = &parameters(Profile::Salsa12);
const SALSA20_PARAMETERS: &[o::Parameter] = &parameters(Profile::Salsa20);
const PROFILED: [m::Entry; 7] = [
    declaration(
        CHACHA8_ID,
        "ChaCha8 IETF",
        "cipher chacha8",
        "cube cipher chacha8 --key-file key.bin --nonce-hex 000000000000000000000000 --counter 0 --input message.bin --stdout",
        CHACHA8_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        CHACHA12_ID,
        "ChaCha12 IETF",
        "cipher chacha12",
        "cube cipher chacha12 --key-file key.bin --nonce-hex 000000000000000000000000 --counter 0 --input message.bin --stdout",
        CHACHA12_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        CHACHA20_ID,
        "ChaCha20 IETF",
        "cipher chacha20",
        "cube cipher chacha20 --key-file key.bin --nonce-hex 000000000000000000000000 --counter 0 --input message.bin --stdout",
        CHACHA20_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        XCHACHA20_ID,
        "XChaCha20 IETF counter",
        "cipher xchacha20",
        "cube cipher xchacha20 --key-file key.bin --nonce-hex 000000000000000000000000000000000000000000000000 --counter 0 --input message.bin --stdout",
        XCHACHA20_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        SALSA12_ID,
        "Salsa20/12",
        "cipher salsa20-12",
        "cube cipher salsa20-12 --key-file key.bin --nonce-hex 0000000000000000 --counter 0 --input message.bin --stdout",
        SALSA12_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        SALSA20_ID,
        "Salsa20/20",
        "cipher salsa20",
        "cube cipher salsa20 --key-file key.bin --nonce-hex 0000000000000000 --counter 0 --input message.bin --stdout",
        SALSA20_PARAMETERS,
        "stream_transform",
    ),
    declaration(
        VERNAM_ID,
        "Educational exact-pad Vernam",
        "cipher vernam",
        "cube cipher vernam --key-file pad.bin --input message.bin --stdout",
        &[],
        "vernam",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const CONTRACTS: &[c::Entry] = &[
    entry(0, &[INPUT, KEY], c::schema::<ChaChaParameters>, fixture_0),
    entry(1, &[INPUT, KEY], c::schema::<ChaChaParameters>, fixture_1),
    entry(2, &[INPUT, KEY], c::schema::<ChaChaParameters>, fixture_2),
    entry(3, &[INPUT, KEY], c::schema::<XChaChaParameters>, fixture_3),
    entry(4, &[INPUT, KEY], c::schema::<SalsaParameters>, fixture_4),
    entry(5, &[INPUT, KEY], c::schema::<SalsaParameters>, fixture_5),
    entry(6, &[INPUT, PAD], c::schema::<NoParameters>, fixture_vernam),
];
const fn entry(
    index: usize,
    inputs: &'static [c::Port],
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        parameters,
        outputs: OUTPUT,
        limits: c::Limits {
            max_input_bytes: Some(2 * MAX_INPUT as u64),
            max_output_bytes: Some(MAX_INPUT as u64),
            max_items: None,
            max_work: Some(MAX_INPUT as u64),
            max_memory_bytes: Some(3 * MAX_INPUT as u64 + 1024),
        },
        fixture,
    }
}
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn bytes(data: &[u8]) -> serde_json::Value {
    json!({"base64":crate::encoding::base64_encode(data)})
}
fn fixture(p: Profile, expected: &str) -> Result<c::Fixture, OperationError> {
    let key = [0; 32];
    let nonce = vec![0; p.nonce_bytes()];
    let output = transform(p, &[0; 16], &key, &nonce, 0)?;
    assert_eq!(hex::encode(&output), expected);
    Ok(c::Fixture {
        inputs: json!({"input":bytes(&[0;16]),"key":bytes(&key)}),
        parameters: json!({"nonce":nonce,"counter":0}),
        outputs: json!({"output":bytes(&output)}),
    })
}
fn fixture_0() -> Result<c::Fixture, OperationError> {
    fixture(Profile::ChaCha8, "3e00ef2f895f40d67f5bb8e81f09a5a1")
}
fn fixture_1() -> Result<c::Fixture, OperationError> {
    fixture(Profile::ChaCha12, "9bf49a6a0755f953811fce125f2683d5")
}
fn fixture_2() -> Result<c::Fixture, OperationError> {
    fixture(Profile::ChaCha20, "76b8e0ada0f13d90405d6ae55386bd28")
}
fn fixture_3() -> Result<c::Fixture, OperationError> {
    let key: Vec<u8> = (0x80..=0x9f).collect();
    let nonce =
        hex::decode("404142434445464748494a4b4c4d4e4f5051525354555658").expect("draft nonce");
    let output = transform(Profile::XChaCha20, &[0; 16], &key, &nonce, 0)?;
    assert_eq!(hex::encode(&output), "1131ce9a2a20ae0d67c8935c7789fa10");
    Ok(c::Fixture {
        inputs: json!({"input":bytes(&[0;16]),"key":bytes(&key)}),
        parameters: json!({"nonce":nonce,"counter":0}),
        outputs: json!({"output":bytes(&output)}),
    })
}
fn fixture_4() -> Result<c::Fixture, OperationError> {
    fixture(Profile::Salsa12, "bd78a2f8118a563c761db4f2fbe055da")
}
fn fixture_5() -> Result<c::Fixture, OperationError> {
    fixture(Profile::Salsa20, "9a97f65b9b4c721b960a672145fca8d4")
}
fn fixture_vernam() -> Result<c::Fixture, OperationError> {
    let output = vernam(&[0, 255], &[255, 15])?;
    assert_eq!(output, [255, 240]);
    Ok(c::Fixture {
        inputs: json!({"input":bytes(&[0,255]),"key":bytes(&[255,15])}),
        parameters: json!({}),
        outputs: json!({"output":bytes(&output)}),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixtures_validate_actual_results_and_private_policy() {
        for entry in CONTRACTS {
            let c = c::describe(&entry.descriptor).unwrap();
            c.validate_fixture(&(entry.fixture)().unwrap()).unwrap();
            assert!(c.policy.private_input && c.policy.private_output);
            assert!(!c.policy.automatic_capture);
            assert!(c.pipeline_adapter.is_none());
            assert!(c.validate_parameters(&json!({"extra":true})).is_err());
        }
    }
}
