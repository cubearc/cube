//! Canonical contracts for compression with one explicit stream type.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;

const FORMAT: o::Parameter = o::Parameter {
    name: "profile",
    description: "Explicit compressed stream and wrapper.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text {
        choices: Profile::NAMES,
    },
};
const LEVEL: o::Parameter = o::Parameter {
    name: "level",
    description: "Deflate, zlib and gzip only. Range 0 through 9, with 6 used when omitted.",
    required: false,
    default: None,
    constraints: o::ParameterType::Unsigned { min: 0, max: 9 },
};
const fn row(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    parameters: &'static [o::Parameter],
    command: &'static str,
    example: &'static str,
    api: [&'static str; 2],
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "binary",
            description,
            parameters,
            command,
            example,
            input_type: "bytes",
            output_type: "bytes",
            deterministic: true,
            reversible: Some(true),
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "compression",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Lossless compression without authentication. Only explicitly selected single streams are accepted.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exact bytes.",
                key: "No key.",
                key_advancement: "Not applicable.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "16 MiB input and output, 16 MiB maximum advertised window or dictionary, and 256:1 expansion ceiling.",
            rust_api: api[0],
            python_api: api[1],
            references: &[
                "https://www.rfc-editor.org/rfc/rfc1950",
                "https://www.rfc-editor.org/rfc/rfc1951",
                "https://www.rfc-editor.org/rfc/rfc1952",
                "https://www.rfc-editor.org/rfc/rfc7932",
                "https://www.rfc-editor.org/rfc/rfc8878",
                "https://tukaani.org/xz/lzma-file-format.txt",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    row(
        ENCODE,
        "Compress bytes",
        "Compress exact bytes with one named wire format and bounded output.",
        &[FORMAT, LEVEL],
        "binary compress",
        "cube binary compress --profile zlib --text abc",
        ["cube_core::binary::compression::compress", "compress_bytes"],
    ),
    row(
        DECODE,
        "Decompress bytes",
        "Read one complete frame in the named wire format and reject trailing data.",
        &[FORMAT],
        "binary decompress",
        "cube binary decompress --profile zlib --file data.zlib",
        [
            "cube_core::binary::compression::decompress",
            "decompress_bytes",
        ],
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

const INPUT: c::Port = c::Port {
    name: "input",
    semantic_type: "bytes",
    required: true,
    schema: c::bytes_schema::<MAX_BYTES>,
};
const OUTPUT: c::Port = c::Port {
    name: "output",
    ..INPUT
};
const fn contract(
    index: usize,
    settings: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: &[INPUT],
        parameters: settings,
        outputs: &[OUTPUT],
        limits: c::Limits {
            max_input_bytes: Some(MAX_BYTES as u64),
            max_output_bytes: Some(MAX_BYTES as u64),
            max_items: None,
            max_work: None,
            max_memory_bytes: Some(64 << 20),
        },
        fixture,
    }
}
fn encode_parameters() -> schemars::Schema {
    // The caller may omit the level. Its effective default depends on the profile.
    let mut schema = c::schema::<EncodeSettings>().to_value();
    schema["properties"]["level"]
        .as_object_mut()
        .expect("typed level schema")
        .remove("default");
    schemars::Schema::try_from(schema).expect("compression settings schema")
}
const CONTRACTS: &[c::Entry] = &[
    contract(0, encode_parameters, || {
        let output = compress(
            b"abc",
            EncodeSettings {
                profile: Profile::Zlib,
                level: None,
            },
        )?;
        Ok(c::Fixture {
            inputs: json!({"input":{"base64":"YWJj"}}),
            parameters: json!({"profile":"zlib"}),
            outputs: json!({"output":c::values::Bytes{data:output}}),
        })
    }),
    contract(1, c::schema::<DecodeSettings>, || {
        let input = vec![
            0x78, 0x9c, 0x4b, 0x4c, 0x4a, 0x06, 0x00, 0x02, 0x4d, 0x01, 0x27,
        ];
        let output = decompress(
            &input,
            DecodeSettings {
                profile: Profile::Zlib,
            },
        )?;
        Ok(c::Fixture {
            inputs: json!({"input":c::values::Bytes{data:input}}),
            parameters: json!({"profile":"zlib"}),
            outputs: json!({"output":c::values::Bytes{data:output}}),
        })
    }),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
