use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;
const DESCRIPTION: &str = "Explicit full-round raw block reference profiles; no padding, authentication or secure message-encryption claim.";
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact full-round block profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "key_hex",
        description: "Required private exact hexadecimal key, with profile-specific byte length.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "effective_key_bits",
        description: "Required for RC2 only: 1..1024 effective bits, independent of key byte length. Forbidden for other profiles.",
        required: false,
        default: None,
        constraints: o::ParameterType::Integer { min: 1, max: 1024 },
    },
    o::Parameter {
        name: "tweak_hex",
        description: "CRAFT only: required exact eight-byte hexadecimal tweak. Forbidden for other profiles; no implicit zero tweak.",
        required: false,
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
            canonical_name: "Raw block reference profiles",
            family: "modern_block",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Unaudited non-constant-time reference backends for legacy interoperability and education. Small blocks and weak keys remain insecure. No authentication; decryption does not verify key or integrity.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex", "tweak_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact bytes; no text conversion.",
                key: "Explicit required profile and exact key bytes. DES parity bits are ignored, not corrected; weak and degenerate keys remain accepted for forensic compatibility.",
                key_advancement: "Independent whole blocks. No IV, counter, padding, nonce, KDF, RNG or mode.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Input/output at most 1 MiB (64 KiB for educational reference implementations), whole profile blocks; empty identity after settings validation. Private JSON at most 4096 bytes. One output allocation plus fixed backend state. Owned key and supported expanded schedules wiped on drop; upstream temporary stacks, caller/parser/interpreter/OS copies are not guaranteed erased. No pipeline or automatic capture. Host transport budgets remain independent.",
            rust_api: api,
            python_api,
            references: &[
                "https://github.com/RustCrypto/block-ciphers/tree/9a84f2f8b581f62bcd00e16b5e11a2f6251916ee",
                "https://www.rfc-editor.org/rfc/rfc2268",
                "https://www.schneier.com/academic/blowfish/",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher block encrypt",
        "cube_core::ciphers::block_profiles::encrypt",
        "block_encrypt_blocks",
        "cube cipher block encrypt --profile des_fips46_v1 --key-file key.bin --file blocks.bin",
    ),
    row(
        DECRYPT,
        "cipher block decrypt",
        "cube_core::ciphers::block_profiles::decrypt",
        "block_decrypt_blocks",
        "cube cipher block decrypt --profile des_fips46_v1 --key-file key.bin --file blocks.bin",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn parameters_schema() -> schemars::Schema {
    let mut schema = c::schema::<Parameters>().to_value();
    // Missing profile-specific settings stay absent; configuration must not
    // inject null into a profile that forbids the setting.
    for field in ["effective_key_bits", "tweak_hex"] {
        schema["properties"][field]
            .as_object_mut()
            .unwrap()
            .remove("default");
    }
    let base = schema.clone();
    schema["oneOf"] = json!(
        Profile::ALL
            .iter()
            .map(|p| {
                let mut row = base.clone();
                for redundant in ["$defs", "$schema", "title"] {
                    row.as_object_mut().unwrap().remove(redundant);
                }
                row["properties"]["profile"] = json!({"const":p});
                row["properties"]["key_hex"]["pattern"] = json!(p.key_pattern());
                if *p == Profile::Craft {
                    row["properties"]["tweak_hex"] =
                        json!({"type":"string","pattern":"^[0-9A-Fa-f]{16}$"});
                    row["required"] = json!(["profile", "key_hex", "tweak_hex"]);
                } else {
                    row["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("tweak_hex");
                }
                if *p == Profile::Rc2 {
                    row["required"] = json!(["profile", "key_hex", "effective_key_bits"]);
                    row["properties"]["effective_key_bits"] =
                        json!({"type":"integer","minimum":1,"maximum":1024});
                } else {
                    row["properties"]
                        .as_object_mut()
                        .unwrap()
                        .remove("effective_key_bits");
                }
                row
            })
            .collect::<Vec<_>>()
    );
    schemars::Schema::try_from(schema).expect("profile-specific typed parameters")
}
fn bytes_schema() -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(MAX_INPUT.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = json!(MAX_INPUT);
    schemars::Schema::try_from(schema).expect("bounded canonical byte transport")
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
            max_items: Some((MAX_INPUT / 4) as u64),
            max_work: Some((MAX_INPUT / 4) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let key = hex::decode("0113b970fd34f2ce").unwrap();
    let plain = hex::decode("059b5e0851cf143a").unwrap();
    let encrypted = hex::decode("86a560f10ec6d85b").unwrap();
    let (input, expected) = if decrypting {
        (encrypted, plain)
    } else {
        (plain, encrypted)
    };
    let settings = Settings::new(Profile::Des, &key, None)?;
    let actual = if decrypting {
        decrypt(&input, &settings)?
    } else {
        encrypt(&input, &settings)?
    };
    assert_eq!(actual, expected);
    Ok(c::Fixture {
        inputs: json!({"input":c::values::Bytes{data:input}}),
        parameters: json!({"profile":"des_fips46_v1","key_hex":hex::encode(key)}),
        outputs: json!({"output":c::values::Bytes{data:actual}}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
