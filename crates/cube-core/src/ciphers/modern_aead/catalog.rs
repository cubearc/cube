use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;
const DESCRIPTION: &str = "Explicit authenticated byte profiles with exact associated data and detached tags; authenticate completely before plaintext release.";
const SIV_DESCRIPTION: &str = "RFC5297 deterministic nonce-free AES-SIV-CMAC with ordered associated-data components and a detached16-byte synthetic IV.";
const fn parameter(
    name: &'static str,
    description: &'static str,
    choices: &'static [&'static str],
) -> o::Parameter {
    o::Parameter {
        name,
        description,
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices },
    }
}
const PARAMETERS: &[o::Parameter] = &[
    parameter(
        "profile",
        "Required exact algorithm, key/nonce/tag profile; no default.",
        Profile::NAMES,
    ),
    parameter(
        "key_hex",
        "Required private key as exact hexadecimal, without separators or prefix.",
        &[],
    ),
    parameter(
        "nonce_hex",
        "Required exact profile-specific nonce; caller manages uniqueness across calls.",
        &[],
    ),
];
const SIV_PARAMETERS: &[o::Parameter] = &[
    parameter(
        "profile",
        "Required exact nonce-free SIV profile; no nonce field is accepted.",
        SivProfile::NAMES,
    ),
    parameter(
        "key_hex",
        "Required32/64-byte total SIV-CMAC key as exact hexadecimal.",
        &[],
    ),
];
const fn row(
    id: &'static str,
    command: &'static str,
    api: &'static str,
    python: &'static str,
    siv: bool,
) -> m::Entry {
    let description = if siv { SIV_DESCRIPTION } else { DESCRIPTION };
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description,
            input_type: "bytes",
            output_type: "bytes",
            parameters: if siv { SIV_PARAMETERS } else { PARAMETERS },
            deterministic: true,
            reversible: Some(true),
            command,
            example: if siv {
                "cube cipher aead siv encrypt --parameters-file settings.json --aad-components-file components.json --file input.bin"
            } else {
                "cube cipher aead encrypt --parameters-file settings.json --aad-file context.bin --file input.bin"
            },
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: if siv {
                "AES-SIV nonce-free authentication"
            } else {
                "Authenticated byte encryption"
            },
            family: "authenticated_ciphers",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Caller manages key/nonce usage and per-key security limits; no reuse tracking, certification or blanket backend audit/constant-time guarantee. SIV reveals equality for repeated plaintext and identical AAD components. GCM-SIV misuse resistance does not permit unlimited reuse. See profile-specific backend qualifications.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &["key_hex"],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exact binary data, associated data and full detached tag; no implicit text encoding or envelope.",
                key: if siv {
                    "Required32/64-byte combined SIV-CMAC key; no nonce, RNG or KDF."
                } else {
                    "Exact profile-specific key/nonce; full8/16/32-byte tag. No RNG, truncation, KDF or defaults."
                },
                key_advancement: "Fresh bounded backend state per call; borrowed caller data remains caller-owned. AAD is authenticated but not encrypted.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Core plaintext/ciphertext plus AAD <=16MiB; CCM nonce13 body <=65535 and nonce12 body <=16777215 bytes. SIV accepts <=126 ordered AAD byte components. Private settings <=2048 UTF-8 bytes. Output preserves body length with a separate full tag; decryption returns nothing before verification. Owned decoded keys and temporary plaintext wiped; backend, parser, caller, interpreter and OS copies may remain. No automatic capture or linear recipe adapter; host transport budgets may be lower.",
            rust_api: api,
            python_api: python,
            references: &[
                "https://csrc.nist.gov/pubs/sp/800/38/d/final",
                "https://www.rfc-editor.org/rfc/rfc8439",
                "https://www.rfc-editor.org/rfc/rfc5297",
                "https://www.rfc-editor.org/rfc/rfc8452",
                "https://csrc.nist.gov/pubs/sp/800/232/final",
                "https://www.rfc-editor.org/rfc/rfc10032",
                "https://github.com/RustCrypto/AEADs",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 4] = [
    row(
        ENCRYPT,
        "cipher aead encrypt",
        "cube_core::ciphers::modern_aead::encrypt",
        "aead_encrypt",
        false,
    ),
    row(
        DECRYPT,
        "cipher aead decrypt",
        "cube_core::ciphers::modern_aead::decrypt",
        "aead_decrypt",
        false,
    ),
    row(
        SIV_ENCRYPT,
        "cipher aead siv encrypt",
        "cube_core::ciphers::modern_aead::siv_encrypt",
        "aes_siv_encrypt",
        true,
    ),
    row(
        SIV_DECRYPT,
        "cipher aead siv decrypt",
        "cube_core::ciphers::modern_aead::siv_decrypt",
        "aes_siv_decrypt",
        true,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn hex_pattern(size: usize) -> serde_json::Value {
    json!({"type":"string","pattern":format!("^[0-9A-Fa-f]{{{}}}$",size*2)})
}
fn parameters() -> schemars::Schema {
    let mut schema = c::schema::<Parameters>().to_value();
    schema["properties"]["key_hex"]["maxLength"] = json!(64);
    schema["properties"]["nonce_hex"]["maxLength"] = json!(64);
    schema["allOf"]=json!(profiles::specs().iter().map(|s|json!({"if":{"properties":{"profile":{"const":s.name}}},"then":{"properties":{"key_hex":hex_pattern(s.key),"nonce_hex":hex_pattern(s.nonce)}}})).collect::<Vec<_>>());
    schemars::Schema::try_from(schema).expect("typed exact authenticated profiles")
}
fn siv_parameters() -> schemars::Schema {
    let mut schema = c::schema::<siv::Parameters>().to_value();
    schema["properties"]["key_hex"]["maxLength"] = json!(128);
    schema["allOf"]=json!(SivProfile::NAMES.iter().zip([32,64]).map(|(name,size)|json!({"if":{"properties":{"profile":{"const":name}}},"then":{"properties":{"key_hex":hex_pattern(size)}}})).collect::<Vec<_>>());
    schemars::Schema::try_from(schema).expect("typed nonce-free SIV profiles")
}
fn bounded_bytes(maximum: usize) -> schemars::Schema {
    let mut schema = c::schema::<c::values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(maximum.div_ceil(3) * 4);
    schema["properties"]["base64"]["x-cube-max-decoded-bytes"] = json!(maximum);
    schemars::Schema::try_from(schema).expect("bounded authenticated byte port")
}
fn bytes() -> schemars::Schema {
    bounded_bytes(MAX_INPUT)
}
fn tag() -> schemars::Schema {
    let mut schema = bounded_bytes(32).to_value();
    schema["properties"]["base64"]["minLength"] = json!(12);
    schemars::Schema::try_from(schema).unwrap()
}
fn siv_tag() -> schemars::Schema {
    let mut schema = bounded_bytes(16).to_value();
    schema["properties"]["base64"]["minLength"] = json!(24);
    schemars::Schema::try_from(schema).unwrap()
}
fn components() -> schemars::Schema {
    let mut schema = c::schema::<Vec<Vec<u8>>>().to_value();
    schema["maxItems"] = json!(MAX_AAD_COMPONENTS);
    schema["items"]["maxItems"] = json!(MAX_INPUT);
    schemars::Schema::try_from(schema).expect("bounded ordered byte arrays")
}
const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    schema: bytes,
    required: true,
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const AAD: c::Port = c::Port {
    name: "aad",
    ..INPUT
};
const COMPONENTS: c::Port = c::Port {
    name: "aad_components",
    semantic_type: "json",
    schema: components,
    required: true,
};
const TAG: c::Port = c::Port {
    name: "tag",
    semantic_type: "bytes",
    schema: tag,
    required: true,
};
const SIV_TAG: c::Port = c::Port {
    schema: siv_tag,
    ..TAG
};
const fn entry(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    let siv = index >= 2;
    let inverse = index % 2 == 1;
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: match (siv, inverse) {
            (false, false) => &[INPUT, AAD],
            (false, true) => &[INPUT, AAD, TAG],
            (true, false) => &[INPUT, COMPONENTS],
            (true, true) => &[INPUT, COMPONENTS, SIV_TAG],
        },
        parameters: if siv { siv_parameters } else { parameters },
        outputs: if inverse {
            &[OUTPUT]
        } else if siv {
            &[OUTPUT, SIV_TAG]
        } else {
            &[OUTPUT, TAG]
        },
        limits: c::Limits {
            max_input_bytes: Some((MAX_INPUT + if inverse { 32 } else { 0 }) as u64),
            max_output_bytes: Some((MAX_INPUT + if inverse { 0 } else { 32 }) as u64),
            max_items: if siv {
                Some(MAX_AAD_COMPONENTS as u64)
            } else {
                None
            },
            max_work: Some(MAX_INPUT as u64),
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    entry(0, encrypt_fixture),
    entry(1, decrypt_fixture),
    entry(2, siv_encrypt_fixture),
    entry(3, siv_decrypt_fixture),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn transport(bytes: &[u8]) -> serde_json::Value {
    json!(c::values::Bytes {
        data: bytes.to_vec()
    })
}
fn fixture(siv: bool, inverse: bool) -> Result<c::Fixture, OperationError> {
    let (plain, cipher, tag, parameters, aad) = if siv {
        (
            hex::decode("112233445566778899aabbccddee").unwrap(),
            hex::decode("40c02b9690c4dc04daef7f6afe5c").unwrap(),
            hex::decode("85632d07c6e8f37f950acd320a2ecc93").unwrap(),
            json!({"profile":"aes128_siv_cmac_nonce_free","key_hex":"fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"}),
            hex::decode("101112131415161718191a1b1c1d1e1f2021222324252627").unwrap(),
        )
    } else {
        (
            vec![],
            vec![],
            hex::decode("58e2fccefa7e3061367f1d57a4e7455a").unwrap(),
            json!({"profile":"aes128_gcm","key_hex":"00".repeat(16),"nonce_hex":"00".repeat(12)}),
            vec![],
        )
    };
    let mut inputs = json!({"input":transport(if inverse{&cipher}else{&plain})});
    if siv {
        inputs["aad_components"] = json!([aad]);
    } else {
        inputs["aad"] = transport(&aad);
    }
    let mut outputs = json!({"output":transport(if inverse{&plain}else{&cipher})});
    if inverse {
        inputs["tag"] = transport(&tag);
    } else {
        outputs["tag"] = transport(&tag);
    }
    if siv {
        let s = SivSettings::parse(&parameters.to_string())?;
        let components: Vec<Vec<u8>> =
            serde_json::from_value(inputs["aad_components"].clone()).unwrap();
        if inverse {
            assert_eq!(siv_decrypt(&cipher, &components, &tag, &s)?, plain);
        } else {
            let result = siv_encrypt(&plain, &components, &s)?;
            assert_eq!(result.ciphertext, cipher);
            assert_eq!(result.tag, tag);
        }
    } else {
        let s = Settings::parse(&parameters.to_string())?;
        if inverse {
            assert_eq!(decrypt(&cipher, &aad, &tag, &s)?, plain);
        } else {
            let result = encrypt(&plain, &aad, &s)?;
            assert_eq!(result.ciphertext, cipher);
            assert_eq!(result.tag, tag);
        }
    }
    Ok(c::Fixture {
        inputs,
        parameters,
        outputs,
    })
}
fn encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false, false)
}
fn decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false, true)
}
fn siv_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true, false)
}
fn siv_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true, true)
}
