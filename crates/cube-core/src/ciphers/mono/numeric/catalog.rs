use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required explicit finite numeric profile; see typed schema. No inferred alphabet or convention.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
const TEXT: &[c::Port] = &[c::Port {
    name: "text",
    semantic_type: "text",
    schema: c::schema::<super::super::params::Text>,
    required: true,
}];
const TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "cipher.symbol_tokens",
    schema: c::schema::<Tokens>,
    required: true,
}];
const GROUPS: &[c::Port] = &[c::Port {
    name: "groups",
    semantic_type: "json",
    schema: c::schema::<product::Groups>,
    required: true,
}];
const PRODUCTS: &[c::Port] = &[c::Port {
    name: "products",
    semantic_type: "json",
    schema: c::schema::<Tokens>,
    required: true,
}];
const ENCODED: &[c::Port] = &[
    c::Port {
        name: "tokens",
        semantic_type: "cipher.symbol_tokens",
        schema: || c::field::<Encoded>("tokens"),
        required: true,
    },
    c::Port {
        name: "table",
        semantic_type: "cipher.homophonic_table",
        schema: || c::field::<Encoded>("table"),
        required: true,
    },
    c::Port {
        name: "definition_sha256",
        semantic_type: "text",
        schema: || c::field::<Encoded>("definition_sha256"),
        required: true,
    },
];
const DECODED: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<Decoded>("text"),
        required: true,
    },
    c::Port {
        name: "table",
        semantic_type: "cipher.homophonic_table",
        schema: || c::field::<Decoded>("table"),
        required: true,
    },
    c::Port {
        name: "definition_sha256",
        semantic_type: "text",
        schema: || c::field::<Decoded>("definition_sha256"),
        required: true,
    },
];
const MULTISET: &[c::Port] = &[
    c::Port {
        name: "groups",
        semantic_type: "json",
        schema: || c::field::<product::Report>("groups"),
        required: true,
    },
    c::Port {
        name: "table",
        semantic_type: "cipher.homophonic_table",
        schema: || c::field::<product::Report>("table"),
        required: true,
    },
    c::Port {
        name: "factor_checks",
        semantic_type: "json",
        schema: || c::field::<product::Report>("factor_checks"),
        required: true,
    },
    c::Port {
        name: "definition_sha256",
        semantic_type: "text",
        schema: || c::field::<product::Report>("definition_sha256"),
        required: true,
    },
];
const ENCODE_REPR: &[e::Representation] = &[
    e::Representation::Json,
    e::Representation::Json,
    e::Representation::Text,
];
const DECODE_REPR: &[e::Representation] = &[
    e::Representation::Text,
    e::Representation::Json,
    e::Representation::Text,
];
const PRODUCT_REPR: &[e::Representation] = &[
    e::Representation::Json,
    e::Representation::Json,
    e::Representation::Json,
    e::Representation::Text,
];
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
    run: fn(&Value, &Value, e::Budget) -> e::Result<Value>,
    array_input: bool,
    product: bool,
    decode: bool,
}
struct Spec {
    id: &'static str,
    name: &'static str,
    python: &'static str,
    command: &'static str,
    reference: &'static [&'static str],
    schema: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
    run: fn(&Value, &Value, e::Budget) -> e::Result<Value>,
    product: bool,
    decode: bool,
}
const fn spec(definition: Spec) -> Row {
    let Spec {
        id,
        name,
        python,
        command,
        reference,
        schema,
        fixture,
        run,
        product,
        decode,
    } = definition;
    let descriptor = o::Descriptor {
        id,
        category: "ciphers",
        description: name,
        input_type: if product {
            "json"
        } else if decode {
            "cipher.symbol_tokens"
        } else {
            "text"
        },
        output_type: if product {
            "json"
        } else if decode {
            "text"
        } else {
            "cipher.symbol_tokens"
        },
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(!product),
        command,
        example: "cube cipher --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if product {
                if decode { PRODUCTS } else { GROUPS }
            } else if decode {
                TOKENS
            } else {
                TEXT
            },
            parameters: schema,
            outputs: if product {
                MULTISET
            } else if decode {
                DECODED
            } else {
                ENCODED
            },
            limits: c::Limits {
                max_input_bytes: Some(MAX_INPUT as u64),
                max_output_bytes: Some(MAX_OUTPUT as u64),
                max_items: Some(MAX_TOKENS as u64),
                max_work: None,
                max_memory_bytes: None,
            },
            fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: name,
                family: "numeric_puzzle",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::PuzzleCode,
                    limitations: "Puzzle code only; no confidentiality or authentication. Prime products preserve multiplicities, not original order.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: name,
                    alphabet: "Required exact unique Unicode scalars in caller order; no normalization or case folding.",
                    key: "Explicit indexed-prime bound, coprime affine parameters, or three distinct trits; no conventional alphabet inferred.",
                    key_advancement: "Stateless deterministic mapping. Product decode returns alphabet-ordered multisets, not guessed original text.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Parameters <=64 KiB; input <=1 MiB/65536 total scalars or tokens; report <=4 MiB. Prime sieve bound <=1000000 and work <=33554432. Product arithmetic checked u128; factor trials explicitly capped <=1000000; empty product is1.",
                rust_api: "cube_core::ciphers::mono::numeric",
                python_api: python,
                references: reference,
            }),
        },
        run,
        array_input: product || decode,
        product,
        decode,
    }
}
fn native<P: serde::de::DeserializeOwned, O: Serialize>(
    i: &Value,
    p: &Value,
    name: &'static str,
    run: impl FnOnce(&Value, &P) -> Result<O, OperationError>,
) -> e::Result<Value> {
    let run = || {
        let p: P = crate::ciphers::poly::parse_parameters(&p.to_string())?;
        let value = i
            .get(name)
            .filter(|_| i.as_object().is_some_and(|v| v.len() == 1))
            .ok_or_else(|| {
                error(
                    "numeric",
                    ErrorCode::InvalidInput,
                    name,
                    "exactly one named input required",
                )
            })?;
        let report = run(value, &p)?;
        serde_json::to_value(report).map_err(|_| invalid("numeric", "report"))
    };
    run().map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
fn text(v: &Value) -> Result<&str, OperationError> {
    v.as_str()
        .ok_or_else(|| error("numeric", ErrorCode::InvalidInput, "text", "expected text"))
}
fn tokens(v: &Value) -> Result<Vec<String>, OperationError> {
    serde_json::from_value(v.clone()).map_err(|_| {
        error(
            "numeric",
            ErrorCode::InvalidInput,
            "tokens",
            "expected an array of strings",
        )
    })
}
const ROWS: [Row; 8] = [
    spec(Spec {
        id: "cipher.prime_number.encode",
        name: "Encode Indexed prime alphabet",
        python: "prime_number_encode",
        command: "cipher prime-number",
        reference: &["https://dlmf.nist.gov/27.18"],
        schema: c::schema::<prime::Parameters>,
        fixture: super::fixtures::prime_encode,
        run: |i, p, _| native(i, p, "text", |v, p| prime::encode(text(v)?, p)),
        product: false,
        decode: false,
    }),
    spec(Spec {
        id: "cipher.prime_number.decode",
        name: "Decode Indexed prime alphabet",
        python: "prime_number_decode",
        command: "cipher prime-number",
        reference: &["https://dlmf.nist.gov/27.18"],
        schema: c::schema::<prime::Parameters>,
        fixture: super::fixtures::prime_decode,
        run: |i, p, _| native(i, p, "tokens", |v, p| prime::decode(&tokens(v)?, p)),
        product: false,
        decode: true,
    }),
    spec(Spec {
        id: "cipher.prime_multiplication.encode",
        name: "Encode Prime-product multiset",
        python: "prime_multiplication_encode",
        command: "cipher prime-multiplication",
        reference: &["https://dlmf.nist.gov/27.18"],
        schema: c::schema::<product::Parameters>,
        fixture: super::fixtures::product_encode,
        run: |i, p, _| native(i, p, "groups", |v, p| product::encode(&tokens(v)?, p)),
        product: true,
        decode: false,
    }),
    spec(Spec {
        id: "cipher.prime_multiplication.decode",
        name: "Decode Prime-product multiset",
        python: "prime_multiplication_decode",
        command: "cipher prime-multiplication",
        reference: &["https://dlmf.nist.gov/27.18"],
        schema: c::schema::<product::Parameters>,
        fixture: super::fixtures::product_decode,
        run: |i, p, _| native(i, p, "products", |v, p| product::decode(&tokens(v)?, p)),
        product: true,
        decode: true,
    }),
    spec(Spec {
        id: "cipher.modulo.encode",
        name: "Encode Affine modular residues",
        python: "modulo_encode",
        command: "cipher modulo",
        reference: &["https://dlmf.nist.gov/27.1"],
        schema: c::schema::<modulo::Parameters>,
        fixture: super::fixtures::modulo_encode,
        run: |i, p, _| native(i, p, "text", |v, p| modulo::encode(text(v)?, p)),
        product: false,
        decode: false,
    }),
    spec(Spec {
        id: "cipher.modulo.decode",
        name: "Decode Affine modular residues",
        python: "modulo_decode",
        command: "cipher modulo",
        reference: &["https://dlmf.nist.gov/27.1"],
        schema: c::schema::<modulo::Parameters>,
        fixture: super::fixtures::modulo_decode,
        run: |i, p, _| native(i, p, "tokens", |v, p| modulo::decode(&tokens(v)?, p)),
        product: false,
        decode: true,
    }),
    spec(Spec {
        id: "cipher.triliteral.encode",
        name: "Encode Ordered three-trit code",
        python: "triliteral_encode",
        command: "cipher triliteral",
        reference: &["https://www.cryptogram.org/downloads/aca.info/ciphers/FractionatedMorse.pdf"],
        schema: c::schema::<triliteral::Parameters>,
        fixture: super::fixtures::triliteral_encode,
        run: |i, p, _| native(i, p, "text", |v, p| triliteral::encode(text(v)?, p)),
        product: false,
        decode: false,
    }),
    spec(Spec {
        id: "cipher.triliteral.decode",
        name: "Decode Ordered three-trit code",
        python: "triliteral_decode",
        command: "cipher triliteral",
        reference: &["https://www.cryptogram.org/downloads/aca.info/ciphers/FractionatedMorse.pdf"],
        schema: c::schema::<triliteral::Parameters>,
        fixture: super::fixtures::triliteral_decode,
        run: |i, p, _| native(i, p, "tokens", |v, p| triliteral::decode(&tokens(v)?, p)),
        product: false,
        decode: true,
    }),
];
const PROFILED: [m::Entry; 8] = profiles();
const CONTRACTS: [c::Entry; 8] = contracts();
const fn profiles() -> [m::Entry; 8] {
    let mut result = [ROWS[0].profile; 8];
    let mut i = 0;
    while i < 8 {
        result[i] = ROWS[i].profile;
        i += 1;
    }
    result
}
const fn contracts() -> [c::Entry; 8] {
    let mut result = [ROWS[0].contract; 8];
    let mut i = 0;
    while i < 8 {
        result[i] = ROWS[i].contract;
        i += 1;
    }
    result
}
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const fn executor(s: Row) -> e::Entry {
    e::Entry::pure(
        s.contract.descriptor.id,
        if s.array_input {
            &[e::Representation::Json]
        } else {
            &[e::Representation::Text]
        },
        if s.product {
            PRODUCT_REPR
        } else if s.decode {
            DECODE_REPR
        } else {
            ENCODE_REPR
        },
        s.run,
    )
}
const EXECUTORS: &[e::Entry] = &[
    executor(ROWS[0]),
    executor(ROWS[1]),
    executor(ROWS[2]),
    executor(ROWS[3]),
    executor(ROWS[4]),
    executor(ROWS[5]),
    executor(ROWS[6]),
    executor(ROWS[7]),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
