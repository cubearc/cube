use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const DESCRIPTION: &str =
    "Exact FF1 and historical FF3/FF3-1 numeral-string profiles, without authentication.";

const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required FF1 or explicitly historical FF3/FF3-1 profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: Profile::NAMES,
        },
    },
    o::Parameter {
        name: "radix",
        description: "Required numeral radix,2..65536.",
        required: true,
        default: None,
        constraints: o::ParameterType::Integer { min: 2, max: 65536 },
    },
    o::Parameter {
        name: "tweak_hex",
        description: "Required exact hexadecimal tweak, up to65536 bytes for FF1;8 for FF3 or7 for FF3-1.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
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
            input_type: "json",
            output_type: "json",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(true),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "Format-preserving encryption",
            family: "format_preserving",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "No authentication. FF3 and FF3-1 are historical profiles with known cryptanalytic weaknesses, omitted from the2025 second NIST revision draft. No audit or certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex", "tweak_hex"],
            },
            profile: m::Profile {
                variant: DESCRIPTION,
                alphabet: "Exact integer digits0..radix-1; leading zeros and length preserved.",
                key: "Required16/24/32-byte AES key, exact hexadecimal.",
                key_advancement: "Explicit tweak and radix. No alphabet inference, RNG or KDF. Integer arithmetic throughout.",
                block_bits: Some(128),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "At most4096 digits,65536 tweak bytes and132096 settings bytes. Domain at least one million; FF3/FF3-1 half-domain at most2^96. Bounded10/8 Feistel rounds and AES blocks. Keys, tweaks and byte/digit scratch buffers wiped; bigint, caller and interpreter copies are not guaranteed erased. No automatic capture or pipeline. No constant-time claim for bigint operations.",
            rust_api: api,
            python_api,
            references: &[
                "https://doi.org/10.6028/NIST.SP.800-38G",
                "https://csrc.nist.gov/pubs/sp/800/38/g/r1/2pd",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCRYPT,
        "cipher fpe encrypt",
        "cube_core::ciphers::fpe::encrypt",
        "fpe_encrypt",
        "cube cipher fpe encrypt --parameters-file settings.json --file digits.json",
    ),
    row(
        DECRYPT,
        "cipher fpe decrypt",
        "cube_core::ciphers::fpe::decrypt",
        "fpe_decrypt",
        "cube cipher fpe decrypt --parameters-file settings.json --file digits.json",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn digits_schema() -> schemars::Schema {
    let mut value = c::schema::<Vec<u16>>().to_value();
    value["maxItems"] = serde_json::json!(MAX_DIGITS);
    value["minItems"] = serde_json::json!(2);
    schemars::Schema::try_from(value).expect("typed bounded digit array")
}
const PORT: c::Port = c::Port {
    name: "input",
    semantic_type: "json",
    schema: digits_schema,
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
            max_input_bytes: Some((MAX_DIGITS * 6 + 2) as u64),
            max_output_bytes: Some((MAX_DIGITS * 6 + 2) as u64),
            max_items: Some(MAX_DIGITS as u64),
            max_work: Some(10 * ((MAX_TWEAK + MAX_DIGITS * 4 + 31) / 16 + 2) as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encrypt_fixture), contract(1, decrypt_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(decrypting: bool) -> Result<c::Fixture, OperationError> {
    let plain = vec![0u16, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let encrypted = vec![2u16, 4, 3, 3, 4, 7, 7, 4, 8, 4];
    let parameters = serde_json::json!({"profile":"ff1_sp800_38g_integer","radix":10,"key_hex":"2b7e151628aed2a6abf7158809cf4f3c","tweak_hex":""});
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
        inputs: serde_json::json!({"input":input}),
        parameters,
        outputs: serde_json::json!({"output":actual}),
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
