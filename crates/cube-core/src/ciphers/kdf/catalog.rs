//! Direct-only named secret ports and resource limits for KDFs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;

const fn unsigned(
    name: &'static str,
    description: &'static str,
    min: u64,
    max: u64,
) -> o::Parameter {
    o::Parameter {
        name,
        description,
        required: true,
        default: None,
        constraints: o::ParameterType::Unsigned { min, max },
    }
}
const fn choice(
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
const PROFILE: o::Parameter = choice(
    "profile",
    "Named SHA-2 HMAC profile.",
    &["sha256", "sha512"],
);
const DIGEST: o::Parameter = choice(
    "digest",
    "Legacy digest selected explicitly.",
    &["md5", "sha256"],
);
const ITERATIONS: o::Parameter = unsigned(
    "iterations",
    "PBKDF2 iteration count.",
    1,
    MAX_ITERATIONS as u64,
);
const LENGTH: o::Parameter = unsigned(
    "output_bytes",
    "Exact derived length in bytes.",
    1,
    MAX_OUTPUT as u64,
);
const HARD_LENGTH: o::Parameter = unsigned(
    "output_bytes",
    "Exact derived length in bytes.",
    16,
    MAX_OUTPUT as u64,
);
const N: o::Parameter = unsigned(
    "n",
    "Power of two and subject to the combined memory and work limits.",
    2,
    524288,
);
const R: o::Parameter = unsigned("r", "scrypt block size factor.", 1, 1024);
const P: o::Parameter = unsigned("p", "scrypt parallelization factor.", 1, 1024);
const MEMORY: o::Parameter = unsigned(
    "memory_kib",
    "Requested Argon2 memory in KiB, at least eight times the lane count.",
    8,
    MAX_ARGON2_KIB as u64,
);
const PASSES: o::Parameter = unsigned("iterations", "Argon2 passes over memory.", 1, 255);
const LANES: o::Parameter = unsigned("lanes", "Argon2 lane count.", 1, 32);
const COUNT: o::Parameter = unsigned(
    "count",
    "Digest repetitions for each EVP_BytesToKey block.",
    1,
    MAX_EVP_WORK as u64,
);
const KEY_LENGTH: o::Parameter = unsigned(
    "key_bytes",
    "Key length, among at most 1024 combined key and IV bytes.",
    1,
    MAX_OUTPUT as u64,
);
const IV_LENGTH: o::Parameter = unsigned(
    "iv_bytes",
    "IV length within the remaining output bound.",
    0,
    (MAX_OUTPUT - 1) as u64,
);

#[allow(clippy::too_many_arguments)]
const fn row(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    command: &'static str,
    example: &'static str,
    parameters: &'static [o::Parameter],
    rust_api: &'static str,
    python_api: &'static str,
    references: &'static [&'static str],
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description,
            input_type: "bytes",
            output_type: "bytes",
            parameters,
            deterministic: true,
            reversible: Some(false),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "key_derivation",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Raw derived key material. Use an explicit private output destination. Parameters do not create a password database or authenticate artifacts.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exact bytes for key material, salt and context.",
                key: "Caller supplies secret material.",
                key_advancement: "Per named KDF.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Combined memory and work are checked before derivation. Direct execution requires explicit private ports.",
            rust_api,
            python_api,
            references,
        }),
    }
}
const PROFILED: [m::Entry; 5] = [
    row(
        PBKDF2,
        "PBKDF2-HMAC",
        "PBKDF2-HMAC-SHA256 or SHA512 with explicit iterations and output length.",
        "cipher kdf pbkdf2",
        "cube cipher kdf pbkdf2 --secret-file secret.bin --salt-file salt.bin --profile sha256 --iterations 2 --output-bytes 32 > derived.bin",
        &[PROFILE, ITERATIONS, LENGTH],
        "cube_core::ciphers::kdf::pbkdf2",
        "kdf_pbkdf2",
        &["https://www.rfc-editor.org/rfc/rfc8018#section-5.2"],
    ),
    row(
        HKDF,
        "HKDF",
        "RFC 5869 extract and expand with SHA256 or SHA512 and explicit salt and info.",
        "cipher kdf hkdf",
        "cube cipher kdf hkdf --secret-file secret.bin --salt-file salt.bin --profile sha256 --info-file info.bin --output-bytes 32 > derived.bin",
        &[PROFILE, LENGTH],
        "cube_core::ciphers::kdf::hkdf",
        "kdf_hkdf",
        &["https://www.rfc-editor.org/rfc/rfc5869"],
    ),
    row(
        SCRYPT,
        "scrypt",
        "RFC 7914 scrypt with explicit N, r, p and output length.",
        "cipher kdf scrypt",
        "cube cipher kdf scrypt --secret-file secret.bin --salt-file salt.bin --n 16 --r 1 --p 1 --output-bytes 32 > derived.bin",
        &[N, R, P, HARD_LENGTH],
        "cube_core::ciphers::kdf::scrypt",
        "kdf_scrypt",
        &["https://www.rfc-editor.org/rfc/rfc7914"],
    ),
    row(
        ARGON2ID,
        "Argon2id",
        "Version 0x13 Argon2id with explicit memory, iterations, lanes and output length.",
        "cipher kdf argon2id",
        "cube cipher kdf argon2id --secret-file secret.bin --salt-file salt.bin --memory-kib 32 --iterations 2 --lanes 1 --output-bytes 32 > derived.bin",
        &[MEMORY, PASSES, LANES, HARD_LENGTH],
        "cube_core::ciphers::kdf::argon2id",
        "kdf_argon2id",
        &["https://www.rfc-editor.org/rfc/rfc9106"],
    ),
    row(
        EVP_BYTES_TO_KEY,
        "EVP_BytesToKey",
        "Legacy OpenSSL digest chaining with an explicit digest, count, eight-byte salt and key and IV sizes.",
        "cipher kdf evp-bytes-to-key",
        "cube cipher kdf evp-bytes-to-key --secret-file secret.bin --salt-file salt.bin --digest md5 --count 1 --key-bytes 32 --iv-bytes 16 > derived.bin",
        &[DIGEST, COUNT, KEY_LENGTH, IV_LENGTH],
        "cube_core::ciphers::kdf::evp_bytes_to_key",
        "kdf_evp_bytes_to_key",
        &["https://docs.openssl.org/3.5/man3/EVP_BytesToKey/"],
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

fn constrained_bytes<const MIN: usize, const MAX: usize>() -> schemars::Schema {
    let mut shape = c::bytes_schema::<MAX>().to_value();
    shape["properties"]["base64"]["minLength"] = json!(MIN.div_ceil(3) * 4);
    shape["properties"]["base64"]["x-cube-min-decoded-bytes"] = json!(MIN);
    schemars::Schema::try_from(shape).expect("bounded private port")
}
const fn port(name: &'static str, schema: fn() -> schemars::Schema) -> c::Port {
    c::Port {
        name,
        semantic_type: "bytes",
        schema,
        required: true,
    }
}
const PASSWORD: c::Port = port("password", c::bytes_schema::<MAX_SECRET>);
const IKM: c::Port = port("ikm", c::bytes_schema::<MAX_SECRET>);
const SALT: c::Port = port("salt", c::bytes_schema::<MAX_CONTEXT>);
const INFO: c::Port = port("info", c::bytes_schema::<MAX_CONTEXT>);
const HARD_SALT: c::Port = port("salt", constrained_bytes::<8, MAX_SALT_HARD>);
const EVP_SALT: c::Port = port("salt", constrained_bytes::<8, 8>);
const OUT: c::Port = port("output", constrained_bytes::<1, MAX_OUTPUT>);
const HARD_OUT: c::Port = port("output", constrained_bytes::<16, MAX_OUTPUT>);
const KEY: c::Port = port("key", constrained_bytes::<1, MAX_OUTPUT>);
const IV: c::Port = port("iv", c::bytes_schema::<{ MAX_OUTPUT - 1 }>);
#[allow(clippy::too_many_arguments)]
const fn contract(
    index: usize,
    parameters: fn() -> schemars::Schema,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    input_bytes: usize,
    max_work: u64,
    memory: u64,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        outputs,
        parameters,
        limits: c::Limits {
            max_input_bytes: Some(input_bytes as u64),
            max_output_bytes: Some(MAX_OUTPUT as u64),
            max_work: Some(max_work),
            max_items: None,
            max_memory_bytes: Some(memory),
        },
        fixture,
    }
}
fn bytes(data: &[u8]) -> serde_json::Value {
    json!({"base64": crate::encoding::base64_encode(data)})
}
const CONTRACTS: &[c::Entry] = &[
    contract(
        0,
        c::schema::<Pbkdf2>,
        &[PASSWORD, SALT],
        &[OUT],
        MAX_SECRET + MAX_CONTEXT,
        MAX_ITERATIONS as u64,
        16 << 20,
        || {
            let parameters = json!({"profile":"sha256","iterations":2,"output_bytes":32});
            let options =
                serde_json::from_value(parameters.clone()).expect("fixed fixture parameters");
            let result = pbkdf2(b"password", b"12345678", options)?;
            Ok(c::Fixture {
                inputs: json!({"password": bytes(b"password"), "salt": bytes(b"12345678")}),
                parameters,
                outputs: json!({"output": bytes(result.as_slice())}),
            })
        },
    ),
    contract(
        1,
        c::schema::<Hkdf>,
        &[IKM, SALT, INFO],
        &[OUT],
        MAX_SECRET + 2 * MAX_CONTEXT,
        3 * MAX_SECRET as u64,
        16 << 20,
        || {
            let parameters = json!({"profile":"sha256","output_bytes":32});
            let options =
                serde_json::from_value(parameters.clone()).expect("fixed fixture parameters");
            let result = hkdf(b"password", b"12345678", b"context", options)?;
            Ok(c::Fixture {
                inputs: json!({"ikm": bytes(b"password"), "salt": bytes(b"12345678"), "info": bytes(b"context")}),
                parameters,
                outputs: json!({"output": bytes(result.as_slice())}),
            })
        },
    ),
    contract(
        2,
        c::schema::<Scrypt>,
        &[PASSWORD, HARD_SALT],
        &[HARD_OUT],
        MAX_SECRET + MAX_SALT_HARD,
        MAX_HARD_WORK,
        MAX_MEMORY,
        || {
            let parameters = json!({"n":16,"r":1,"p":1,"output_bytes":32});
            let options =
                serde_json::from_value(parameters.clone()).expect("fixed fixture parameters");
            let result = scrypt(b"password", b"12345678", options)?;
            Ok(c::Fixture {
                inputs: json!({"password": bytes(b"password"), "salt": bytes(b"12345678")}),
                parameters,
                outputs: json!({"output": bytes(result.as_slice())}),
            })
        },
    ),
    contract(
        3,
        c::schema::<Argon2id>,
        &[PASSWORD, HARD_SALT],
        &[HARD_OUT],
        MAX_SECRET + MAX_SALT_HARD,
        MAX_HARD_WORK,
        MAX_MEMORY,
        || {
            let parameters = json!({"memory_kib":32,"iterations":2,"lanes":1,"output_bytes":32});
            let options =
                serde_json::from_value(parameters.clone()).expect("fixed fixture parameters");
            let result = argon2id(b"password", b"12345678", options)?;
            Ok(c::Fixture {
                inputs: json!({"password": bytes(b"password"), "salt": bytes(b"12345678")}),
                parameters,
                outputs: json!({"output": bytes(result.as_slice())}),
            })
        },
    ),
    contract(
        4,
        c::schema::<EvpBytesToKey>,
        &[PASSWORD, EVP_SALT],
        &[KEY, IV],
        MAX_SECRET + 8,
        MAX_EVP_WORK as u64,
        16 << 20,
        || {
            let parameters = json!({"digest":"md5","count":1,"key_bytes":32,"iv_bytes":16});
            let options =
                serde_json::from_value(parameters.clone()).expect("fixed fixture parameters");
            let result = evp_bytes_to_key(b"password", b"12345678", options)?;
            Ok(c::Fixture {
                inputs: json!({"password": bytes(b"password"), "salt": bytes(b"12345678")}),
                parameters,
                outputs: json!({"key": bytes(result.key()), "iv": bytes(result.iv())}),
            })
        },
    ),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
