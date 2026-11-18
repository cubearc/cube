//! Canonical invocation contracts for explicit text representations.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;

const UTF16: &[o::Parameter] = &[
    o::Parameter {
        name: "byte_order",
        description: "Interpreted code-unit byte order.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["le", "be"],
        },
    },
    o::Parameter {
        name: "bom",
        description: "Initial mark policy. Require and strip remove a matching mark during decoding; preserve treats it as data.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["require", "forbid", "strip", "preserve"],
        },
    },
];
const JSON: &[o::Parameter] = &[o::Parameter {
    name: "ascii_only",
    description: "Escape all non-ASCII scalars.",
    required: false,
    default: Some(o::ParameterValue::Boolean(true)),
    constraints: o::ParameterType::Boolean,
}];
#[allow(clippy::too_many_arguments)]
const fn row(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    command: &'static str,
    example: &'static str,
    types: [&'static str; 2],
    parameters: &'static [o::Parameter],
    reversible: bool,
    rust_api: &'static str,
    python_api: &'static str,
    references: &'static [&'static str],
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "text",
            description,
            input_type: types[0],
            output_type: types[1],
            parameters,
            deterministic: true,
            reversible: Some(reversible),
            command,
            example,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "text_representations",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::Encoding,
                limitations: "Text representation without security guarantees or markup interpretation.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: description,
                alphabet: "Exact text and the named representation.",
                key: "No key.",
                key_advancement: "Not applicable.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "See directional contracts for the byte ceilings. Work scales linearly with input.",
            rust_api,
            python_api,
            references,
        }),
    }
}
const PROFILED: [m::Entry; 8] = [
    row(
        ASCII_ENCODE,
        "ASCII",
        "Strict U+0000 through U+007F to exact bytes.",
        "text ascii encode",
        "cube text ascii encode --text ABC",
        ["text", "bytes"],
        &[],
        true,
        "cube_core::text::representation::ascii_encode",
        "ascii_encode",
        &["https://www.rfc-editor.org/rfc/rfc20"],
    ),
    row(
        ASCII_DECODE,
        "ASCII",
        "Strict ASCII bytes to their corresponding Unicode scalars.",
        "text ascii decode",
        "cube text ascii decode --text ABC",
        ["bytes", "text"],
        &[],
        true,
        "cube_core::text::representation::ascii_decode",
        "ascii_decode",
        &["https://www.rfc-editor.org/rfc/rfc20"],
    ),
    row(
        UTF16_ENCODE,
        "UTF-16",
        "Encode Unicode scalars in the selected byte order and initial marker policy.",
        "text utf16 encode",
        "cube text utf16 encode --byte-order be --bom require --text A",
        ["text", "bytes"],
        UTF16,
        false,
        "cube_core::text::representation::utf16_encode",
        "utf16_encode",
        &["https://www.rfc-editor.org/rfc/rfc2781"],
    ),
    row(
        UTF16_DECODE,
        "UTF-16",
        "Strict complete UTF-16 code units with the selected byte order and marker policy.",
        "text utf16 decode",
        "cube text utf16 decode --byte-order be --bom require --file text.utf16",
        ["bytes", "text"],
        UTF16,
        false,
        "cube_core::text::representation::utf16_decode",
        "utf16_decode",
        &["https://www.rfc-editor.org/rfc/rfc2781"],
    ),
    row(
        JSON_ENCODE,
        "JSON string escape",
        "JSON string content escapes without surrounding quotes, ASCII-only by default.",
        "text json-escape encode",
        "cube text json-escape encode --text '🧊'",
        ["text", "text"],
        JSON,
        true,
        "cube_core::text::representation::json_escape_encode",
        "json_escape_encode",
        &["https://www.rfc-editor.org/rfc/rfc8259"],
    ),
    row(
        JSON_DECODE,
        "JSON string escape",
        "Decode exact JSON string content, rejecting raw controls and malformed escapes.",
        "text json-escape decode",
        "cube text json-escape decode --text '\\uD83E\\uDDCA'",
        ["text", "text"],
        &[],
        false,
        "cube_core::text::representation::json_escape_decode",
        "json_escape_decode",
        &["https://www.rfc-editor.org/rfc/rfc8259"],
    ),
    row(
        HTML_ENCODE,
        "HTML character references",
        "Encode markup-sensitive scalars as canonical hexadecimal character references.",
        "text html-entity encode",
        "cube text html-entity encode --text '<&>'",
        ["text", "text"],
        &[],
        true,
        "cube_core::text::representation::html_entity_encode",
        "html_entity_encode",
        &["https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state"],
    ),
    row(
        HTML_DECODE,
        "HTML character references",
        "Decode WHATWG text-context named and numeric references in exact text.",
        "text html-entity decode",
        "cube text html-entity decode --text '&lt;&amp;&gt;'",
        ["text", "text"],
        &[],
        false,
        "cube_core::text::representation::html_entity_decode",
        "html_entity_decode",
        &["https://html.spec.whatwg.org/multipage/parsing.html#character-reference-state"],
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

#[derive(schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct EmptyParameters {}
fn json_encode_parameters() -> schemars::Schema {
    c::descriptor_parameters(&OPERATIONS[4])
}

#[allow(clippy::too_many_arguments)]
const fn contract(
    index: usize,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    input_limit: usize,
    output_limit: usize,
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        outputs,
        parameters,
        limits: c::Limits {
            max_input_bytes: Some(input_limit as u64),
            max_output_bytes: Some(output_limit as u64),
            max_work: Some((input_limit + output_limit) as u64),
            max_items: None,
            max_memory_bytes: None,
        },
        fixture,
    }
}

const UTF16_FIXTURE: Utf16Settings = Utf16Settings {
    byte_order: ByteOrder::Be,
    bom: Bom::Require,
};
const CONTRACTS: &[c::Entry] = &[
    contract(
        0,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "bytes",
            required: true,
            schema: c::bytes_schema::<MAX_TEXT>,
        }],
        MAX_TEXT,
        MAX_TEXT,
        c::schema::<EmptyParameters>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":"AB\0"}),
                parameters: json!({}),
                outputs: json!({"output":c::values::Bytes {data:ascii_encode("AB\0")?}}),
            })
        },
    ),
    contract(
        1,
        &[c::Port {
            name: "input",
            semantic_type: "bytes",
            required: true,
            schema: c::bytes_schema::<MAX_TEXT>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        MAX_TEXT,
        MAX_TEXT,
        c::schema::<EmptyParameters>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":{"base64":"QUIA"}}),
                parameters: json!({}),
                outputs: json!({"output":ascii_decode(b"AB\0")?}),
            })
        },
    ),
    contract(
        2,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "bytes",
            required: true,
            schema: c::bytes_schema::<MAX_UTF16>,
        }],
        MAX_TEXT,
        MAX_UTF16,
        c::schema::<Utf16Settings>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":"A🧊"}),
                parameters: json!({"byte_order":"be","bom":"require"}),
                outputs: json!({"output":c::values::Bytes {data:utf16_encode("A🧊", UTF16_FIXTURE)?}}),
            })
        },
    ),
    contract(
        3,
        &[c::Port {
            name: "input",
            semantic_type: "bytes",
            required: true,
            schema: c::bytes_schema::<MAX_UTF16>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_UTF16>,
        }],
        MAX_UTF16,
        MAX_UTF16,
        c::schema::<Utf16Settings>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":{"base64":crate::encoding::base64_encode(&[0xfe,0xff,0,0x41,0xd8,0x3e,0xdd,0xca])}}),
                parameters: json!({"byte_order":"be","bom":"require"}),
                outputs: json!({"output":utf16_decode(&[0xfe,0xff,0,0x41,0xd8,0x3e,0xdd,0xca], UTF16_FIXTURE)?}),
            })
        },
    ),
    contract(
        4,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_ESCAPED>,
        }],
        MAX_TEXT,
        MAX_ESCAPED,
        json_encode_parameters,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":"A🧊"}),
                parameters: json!({"ascii_only":true}),
                outputs: json!({"output":json_escape_encode("A🧊", true)?}),
            })
        },
    ),
    contract(
        5,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_ESCAPED>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        MAX_ESCAPED,
        MAX_TEXT,
        c::schema::<EmptyParameters>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":r"A\uD83E\uDDCA"}),
                parameters: json!({}),
                outputs: json!({"output":json_escape_decode(r"A\uD83E\uDDCA")?}),
            })
        },
    ),
    contract(
        6,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_ESCAPED>,
        }],
        MAX_TEXT,
        MAX_ESCAPED,
        c::schema::<EmptyParameters>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":"<&>"}),
                parameters: json!({}),
                outputs: json!({"output":html_entity_encode("<&>")?}),
            })
        },
    ),
    contract(
        7,
        &[c::Port {
            name: "input",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_ESCAPED>,
        }],
        &[c::Port {
            name: "output",
            semantic_type: "text",
            required: true,
            schema: c::text_schema::<MAX_TEXT>,
        }],
        MAX_ESCAPED,
        MAX_TEXT,
        c::schema::<EmptyParameters>,
        || {
            Ok(c::Fixture {
                inputs: json!({"input":"&lt;&amp;&gt;"}),
                parameters: json!({}),
                outputs: json!({"output":html_entity_decode("&lt;&amp;&gt;")?}),
            })
        },
    ),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
