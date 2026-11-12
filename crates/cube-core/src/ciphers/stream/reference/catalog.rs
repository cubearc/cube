use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact stream construction and initialization; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "key_hex",
        description: "Required private hexadecimal key with profile-specific length.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "nonce_hex",
        description: "Required exact hexadecimal IV; explicitly empty for a construction without an IV.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "discard_bytes",
        description: "Required only for RC4-drop: explicit 0..65536 skipped bytes. Forbidden for other profiles.",
        required: false,
        default: None,
        constraints: o::ParameterType::Unsigned {
            min: 0,
            max: MAX_DISCARD as u64,
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
            description: "Exact raw historical stream profiles with explicit keys and initialization; no authentication.",
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
            canonical_name: "Raw stream reference profiles",
            family: "stream_ciphers",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Unauthenticated and malleable. RC4 is broken; discarding bytes does not repair it. These interoperability backends have no independent audit or constant-time guarantee.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: "Explicit raw stream construction",
                alphabet: "Exact bytes.",
                key: "Required exact key and explicit nonce bytes, including an empty nonce for a key-only profile.",
                key_advancement: "Fresh full initialization for each call. RC4-drop requires its explicit skipped-byte count. No hidden counter, nonce generation or KDF.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Input/output at most64KiB; private JSON at most4096bytes. RC4 discard at most65536bytes. Initialization and state are finite. Owned key/nonce and supported backend schedules are wiped; parser/caller/interpreter/OS copies are not guaranteed erased. No automatic capture or linear recipe adapter.",
            rust_api: api,
            python_api,
            references: &[
                "https://www.rfc-editor.org/rfc/rfc6229",
                "https://www.rfc-editor.org/rfc/rfc4503",
                "https://personal.ntu.edu.sg/wuhj/research/hc/index.html",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher stream encrypt",
        "cube_core::ciphers::stream::reference::encrypt",
        "stream_encrypt",
        "cube cipher stream encrypt --parameters-file settings.json --file message.bin",
    ),
    row(
        DECRYPT,
        "cipher stream decrypt",
        "cube_core::ciphers::stream::reference::decrypt",
        "stream_decrypt",
        "cube cipher stream decrypt --parameters-file settings.json --file encrypted.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn parameters_schema() -> schemars::Schema {
    let mut schema = c::schema::<Parameters>().to_value();
    schema["properties"]["discard_bytes"]
        .as_object_mut()
        .unwrap()
        .remove("default");
    let base = schema.clone();
    schema["oneOf"] = json!(
        Profile::ALL
            .iter()
            .map(|p| {
                let mut row = base.clone();
                for key in ["$defs", "$schema", "title"] {
                    row.as_object_mut().unwrap().remove(key);
                }
                row["properties"]["profile"] = json!({"const":p});
                let (min, max) = p.key_range();
                row["properties"]["key_hex"]["pattern"] =
                    json!(format!("^(?:[0-9A-Fa-f]{{2}}){{{min},{max}}}$"));
                row["properties"]["nonce_hex"]["pattern"] =
                    json!(format!("^[0-9A-Fa-f]{{{}}}$", 2 * p.nonce_bytes()));
                if *p == Profile::Rc4Drop {
                    row["required"] = json!(["profile", "key_hex", "nonce_hex", "discard_bytes"]);
                    row["properties"]["discard_bytes"] =
                        json!({"type":"integer","minimum":0,"maximum":MAX_DISCARD});
                } else {
                    row["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("discard_bytes");
                }
                row
            })
            .collect::<Vec<_>>()
    );
    schemars::Schema::try_from(schema).expect("exact profile parameters")
}
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(MAX_INPUT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = json!(MAX_INPUT);
    schemars::Schema::try_from(schema).expect("bounded byte transport")
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
        parameters: parameters_schema,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some(MAX_INPUT as u64),
            max_output_bytes: Some(MAX_INPUT as u64),
            max_items: Some(MAX_INPUT as u64),
            max_work: Some((MAX_INPUT * 8 + MAX_DISCARD) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let plain = b"Plaintext".to_vec();
    let encrypted = hex::decode("bbf316e8d940af0ad3").unwrap();
    let parameters = json!({"profile":"rc4_rfc6229_v1","key_hex":"4b6579","nonce_hex":""});
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
        inputs: json!({"input":c::values::Bytes{data:input}}),
        parameters,
        outputs: json!({"output":c::values::Bytes{data:actual}}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
