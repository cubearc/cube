use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
const PARAMETERS_0: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact educational reference profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["rabin_blum_reference_v1"],
        },
    },
    o::Parameter {
        name: "max_work",
        description: "Combined key-validation and message-processing weighted integer-step budget.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_WORK)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_WORK,
        },
    },
    o::Parameter {
        name: "max_memory_bytes",
        description: "Conservative owned-data reservation ceiling; caller/process overhead excluded.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_MEMORY_BYTES)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_MEMORY_BYTES,
        },
    },
];
const PARAMETERS_1: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact educational reference profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["goldwasser_micali_reference_v1"],
        },
    },
    o::Parameter {
        name: "max_work",
        description: "Combined key-validation and message-processing weighted integer-step budget.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_WORK)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_WORK,
        },
    },
    o::Parameter {
        name: "max_memory_bytes",
        description: "Conservative owned-data reservation ceiling; caller/process overhead excluded.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_MEMORY_BYTES)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_MEMORY_BYTES,
        },
    },
];
const PARAMETERS_2: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact educational reference profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["benaloh_prime_r_reference_v1"],
        },
    },
    o::Parameter {
        name: "max_work",
        description: "Combined key-validation and message-processing weighted integer-step budget.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_WORK)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_WORK,
        },
    },
    o::Parameter {
        name: "max_memory_bytes",
        description: "Conservative owned-data reservation ceiling; caller/process overhead excluded.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_MEMORY_BYTES)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_MEMORY_BYTES,
        },
    },
];
const PARAMETERS_3: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Required exact educational reference profile; no default.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["merkle_hellman_permuted_reference_v1"],
        },
    },
    o::Parameter {
        name: "max_work",
        description: "Combined key-validation and message-processing weighted integer-step budget.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_WORK)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_WORK,
        },
    },
    o::Parameter {
        name: "max_memory_bytes",
        description: "Conservative owned-data reservation ceiling; caller/process overhead excluded.",
        required: false,
        default: Some(o::ParameterValue::Unsigned(MAX_MEMORY_BYTES)),
        constraints: o::ParameterType::Unsigned {
            min: 1,
            max: MAX_MEMORY_BYTES,
        },
    },
];
const fn row(
    adapter: (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
    name: &'static str,
    description: &'static str,
    parameters: &'static [o::Parameter],
    randomized: bool,
    reversible: bool,
) -> m::Entry {
    let (id, command, api, python, example) = adapter;
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description,
            input_type: "json",
            output_type: "json",
            parameters,
            deterministic: !randomized,
            reversible: Some(reversible),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_public_key_reference",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Educational executable reference only, unsuitable for live secrets. Variable-time bigint arithmetic and no guarantee of private numeric heap erasure. Rabin is ambiguous and has no padding; Goldwasser-Micali and Benaloh are malleable/homomorphic without authentication; Merkle-Hellman is broken. No production-security, primality-certificate, audit or certification claim.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Canonical ASCII decimal integer tokens; GM explicit single bits; MH exact fixed-width binary blocks. No float, byte packing or text inference.",
                key: "Separate <=64KiB UTF8 JSON key with decimal strings and profile-specific fields. Private factor checks use64 bounded fresh uniform Miller-Rabin bases, not a primality certificate. Benaloh r is an odd prime<=4093; MH has2..256 weights and explicit zero-based permutation.",
                key_advancement: "No key generation. GM/Benaloh encryption uses fresh uniformly sampled unit coins; other transforms deterministic. No deterministic production-coin API, arbitrary factoring or unbounded discrete-log/subset search.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Values <=4096 bits; <=4096 tokens; key <=65536 bytes; settings <=1024 bytes; input/report <=4MiB. Combined max_work<=4194304 weighted integer steps; max_memory_bytes<=64MiB conservative owned-data reservation, excluding caller/interpreter/process overhead. Limits combine and can be lowered; exhaustion fails without partial results. Random requests <=16MiB per validation/encryption phase and rejection loops <=128 attempts. Transport/string buffers wiped where owned; bigint heaps/copies not assured erased. Explicit private retention required in Play; hosts may impose lower budgets.",
            rust_api: api,
            python_api: python,
            references: &[
                "https://cacr.uwaterloo.ca/hac/about/chap8.pdf",
                "https://www.microsoft.com/en-us/research/publication/dense-probabilistic-encryption/",
                "https://arxiv.org/abs/1008.2991",
                "https://docs.rs/crypto-primes/0.7.2/crypto_primes/hazmat/struct.MillerRabin.html",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 8] = [
    row(
        (
            RABIN_ENCRYPT,
            "cipher rabin encrypt",
            "cube_core::asymmetric::historical_public_key::rabin::encrypt",
            "rabin_encrypt",
            "cube cipher rabin encrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Rabin Blum-modulus reference",
        "Raw Rabin modular squaring and complete sorted root sets; no root selection or secure padding.",
        PARAMETERS_0,
        false,
        false,
    ),
    row(
        (
            RABIN_DECRYPT,
            "cipher rabin decrypt",
            "cube_core::asymmetric::historical_public_key::rabin::decrypt",
            "rabin_decrypt",
            "cube cipher rabin decrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Rabin Blum-modulus reference",
        "Raw Rabin modular squaring and complete sorted root sets; no root selection or secure padding.",
        PARAMETERS_0,
        false,
        false,
    ),
    row(
        (
            GOLDWASSER_MICALI_ENCRYPT,
            "cipher goldwasser-micali encrypt",
            "cube_core::asymmetric::historical_public_key::goldwasser_micali::encrypt",
            "goldwasser_micali_encrypt",
            "cube cipher goldwasser-micali encrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Goldwasser-Micali reference",
        "One-bit-per-residue Goldwasser-Micali reference encryption with explicit unit coins and Legendre decoding.",
        PARAMETERS_1,
        true,
        true,
    ),
    row(
        (
            GOLDWASSER_MICALI_DECRYPT,
            "cipher goldwasser-micali decrypt",
            "cube_core::asymmetric::historical_public_key::goldwasser_micali::decrypt",
            "goldwasser_micali_decrypt",
            "cube cipher goldwasser-micali decrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Goldwasser-Micali reference",
        "One-bit-per-residue Goldwasser-Micali reference encryption with explicit unit coins and Legendre decoding.",
        PARAMETERS_1,
        false,
        true,
    ),
    row(
        (
            BENALOH_ENCRYPT,
            "cipher benaloh encrypt",
            "cube_core::asymmetric::historical_public_key::benaloh::encrypt",
            "benaloh_encrypt",
            "cube cipher benaloh encrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Prime-r Benaloh reference",
        "Exact prime-r Benaloh reference encryption with corrected private order checks and bounded finite decryption table.",
        PARAMETERS_2,
        true,
        true,
    ),
    row(
        (
            BENALOH_DECRYPT,
            "cipher benaloh decrypt",
            "cube_core::asymmetric::historical_public_key::benaloh::decrypt",
            "benaloh_decrypt",
            "cube cipher benaloh decrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Prime-r Benaloh reference",
        "Exact prime-r Benaloh reference encryption with corrected private order checks and bounded finite decryption table.",
        PARAMETERS_2,
        false,
        true,
    ),
    row(
        (
            MERKLE_HELLMAN_ENCRYPT,
            "cipher merkle-hellman encrypt",
            "cube_core::asymmetric::historical_public_key::merkle_hellman::encrypt",
            "merkle_hellman_encrypt",
            "cube cipher merkle-hellman encrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Permuted Merkle-Hellman reference",
        "Broken Merkle-Hellman reference with exact binary blocks, explicit private permutation and checked greedy decoding.",
        PARAMETERS_3,
        false,
        true,
    ),
    row(
        (
            MERKLE_HELLMAN_DECRYPT,
            "cipher merkle-hellman decrypt",
            "cube_core::asymmetric::historical_public_key::merkle_hellman::decrypt",
            "merkle_hellman_decrypt",
            "cube cipher merkle-hellman decrypt --parameters-file settings.json --key-file key.json --file tokens.json",
        ),
        "Permuted Merkle-Hellman reference",
        "Broken Merkle-Hellman reference with exact binary blocks, explicit private permutation and checked greedy decoding.",
        PARAMETERS_3,
        false,
        true,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn decimal_schema() -> schemars::Schema {
    let mut s = c::schema::<Vec<String>>().to_value();
    s["maxItems"] = serde_json::json!(MAX_TOKENS);
    s["items"]["maxLength"] = serde_json::json!(MAX_TOKEN_BYTES);
    s["items"]["pattern"] = serde_json::json!("^(0|[1-9][0-9]*)$");
    schemars::Schema::try_from(s).unwrap()
}
fn bit_schema() -> schemars::Schema {
    let mut s = decimal_schema().to_value();
    s["items"]["enum"] = serde_json::json!(["0", "1"]);
    schemars::Schema::try_from(s).unwrap()
}
fn binary_schema() -> schemars::Schema {
    let mut s = decimal_schema().to_value();
    s["items"]["maxLength"] = serde_json::json!(256);
    s["items"]["pattern"] = serde_json::json!("^[01]{2,256}$");
    schemars::Schema::try_from(s).unwrap()
}
fn roots_schema() -> schemars::Schema {
    let mut s = c::schema::<Vec<Vec<String>>>().to_value();
    s["maxItems"] = serde_json::json!(MAX_TOKENS);
    s["items"] = decimal_schema().to_value();
    s["items"]["maxItems"] = serde_json::json!(4);
    s["items"]["minItems"] = serde_json::json!(1);
    schemars::Schema::try_from(s).unwrap()
}
fn key_schema() -> schemars::Schema {
    let mut s = c::schema::<c::values::Bytes>().to_value();
    s["properties"]["base64"]["maxLength"] = serde_json::json!(MAX_KEY_BYTES.div_ceil(3) * 4);
    s["properties"]["base64"]["x-cube-max-decoded-bytes"] = serde_json::json!(MAX_KEY_BYTES);
    schemars::Schema::try_from(s).unwrap()
}
const DECIMAL: c::Port = c::Port {
    name: "tokens",
    semantic_type: "json",
    schema: decimal_schema,
    required: true,
};
const BIT: c::Port = c::Port {
    schema: bit_schema,
    ..DECIMAL
};
const BINARY: c::Port = c::Port {
    schema: binary_schema,
    ..DECIMAL
};
const ROOTS: c::Port = c::Port {
    name: "candidates",
    schema: roots_schema,
    ..DECIMAL
};
const KEY: c::Port = c::Port {
    name: "key",
    semantic_type: "bytes",
    schema: key_schema,
    required: true,
};
const fn entry(
    index: usize,
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: if index == 2 {
            &[BIT, KEY]
        } else if index == 6 {
            &[BINARY, KEY]
        } else {
            &[DECIMAL, KEY]
        },
        parameters,
        outputs: if index == 1 {
            &[ROOTS]
        } else if index == 3 {
            &[BIT]
        } else if index == 7 {
            &[BINARY]
        } else {
            &[DECIMAL]
        },
        limits: c::Limits {
            max_input_bytes: Some((MAX_REPORT_BYTES + MAX_KEY_BYTES) as u64),
            max_output_bytes: Some(MAX_REPORT_BYTES as u64),
            max_items: Some(MAX_TOKENS as u64),
            max_work: Some(MAX_WORK),
            max_memory_bytes: Some(MAX_MEMORY_BYTES),
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    entry(0, c::schema::<rabin::Parameters>, rabin_encrypt_fixture),
    entry(1, c::schema::<rabin::Parameters>, rabin_decrypt_fixture),
    entry(
        2,
        c::schema::<goldwasser_micali::Parameters>,
        goldwasser_micali_encrypt_fixture,
    ),
    entry(
        3,
        c::schema::<goldwasser_micali::Parameters>,
        goldwasser_micali_decrypt_fixture,
    ),
    entry(4, c::schema::<benaloh::Parameters>, benaloh_encrypt_fixture),
    entry(5, c::schema::<benaloh::Parameters>, benaloh_decrypt_fixture),
    entry(
        6,
        c::schema::<merkle_hellman::Parameters>,
        merkle_hellman_encrypt_fixture,
    ),
    entry(
        7,
        c::schema::<merkle_hellman::Parameters>,
        merkle_hellman_decrypt_fixture,
    ),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
fn fixture(index: usize, encrypt: bool) -> Result<c::Fixture, OperationError> {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &rows[index];
    let parameters = v["parameters"].clone();
    let doc = parameters.to_string();
    let input: Vec<String> = serde_json::from_value(
        v[if encrypt {
            "tokens"
        } else {
            "ciphertext_tokens"
        }]
        .clone(),
    )
    .unwrap();
    let key = serde_json::to_vec(&v[if encrypt { "public_key" } else { "private_key" }]).unwrap();
    let outputs = match (index, encrypt) {
        (0, true) => {
            serde_json::json!({"tokens":rabin::encrypt(&input,&rabin::EncryptionSettings::parse(&doc,&key)?)?})
        }
        (0, false) => {
            serde_json::json!({"candidates":rabin::decrypt(&input,&rabin::DecryptionSettings::parse(&doc,&key)?)?})
        }
        (2, true) => {
            serde_json::json!({"tokens":goldwasser_micali::encrypt(&input,&goldwasser_micali::EncryptionSettings::parse(&doc,&key)?)?})
        }
        (2, false) => {
            serde_json::json!({"tokens":goldwasser_micali::decrypt(&input,&goldwasser_micali::DecryptionSettings::parse(&doc,&key)?)?})
        }
        (3, true) => {
            serde_json::json!({"tokens":benaloh::encrypt(&input,&benaloh::EncryptionSettings::parse(&doc,&key)?)?})
        }
        (3, false) => {
            serde_json::json!({"tokens":benaloh::decrypt(&input,&benaloh::DecryptionSettings::parse(&doc,&key)?)?})
        }
        (7, true) => {
            serde_json::json!({"tokens":merkle_hellman::encrypt(&input,&merkle_hellman::EncryptionSettings::parse(&doc,&key)?)?})
        }
        (7, false) => {
            serde_json::json!({"tokens":merkle_hellman::decrypt(&input,&merkle_hellman::DecryptionSettings::parse(&doc,&key)?)?})
        }
        _ => unreachable!("reviewed reference fixture"),
    };
    if encrypt && (index == 2 || index == 3) {
        let cipher: Vec<String> = serde_json::from_value(outputs["tokens"].clone()).unwrap();
        let private = serde_json::to_vec(&v["private_key"]).unwrap();
        let recovered = if index == 2 {
            goldwasser_micali::decrypt(
                &cipher,
                &goldwasser_micali::DecryptionSettings::parse(&doc, &private)?,
            )?
        } else {
            benaloh::decrypt(
                &cipher,
                &benaloh::DecryptionSettings::parse(&doc, &private)?,
            )?
        };
        assert_eq!(recovered, input);
    } else {
        let expected = if !encrypt && index == 0 {
            serde_json::json!({"candidates":v["candidate_tokens"]})
        } else {
            serde_json::json!({"tokens":v[if encrypt{"ciphertext_tokens"}else{"tokens"}]})
        };
        assert_eq!(outputs, expected);
    }
    Ok(c::Fixture {
        inputs: serde_json::json!({"tokens":input,"key":c::values::Bytes{data:key}}),
        parameters,
        outputs,
    })
}
fn rabin_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(0, true)
}
fn rabin_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(0, false)
}
fn goldwasser_micali_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(2, true)
}
fn goldwasser_micali_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(2, false)
}
fn benaloh_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(3, true)
}
fn benaloh_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(3, false)
}
fn merkle_hellman_encrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(7, true)
}
fn merkle_hellman_decrypt_fixture() -> Result<c::Fixture, OperationError> {
    fixture(7, false)
}
