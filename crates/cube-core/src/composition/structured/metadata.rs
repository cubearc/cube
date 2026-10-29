use super::*;
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue, metadata as m};
const fn profile(descriptor: Descriptor, name: &'static str, api: [&'static str; 2]) -> m::Entry {
    m::Entry {
        descriptor,
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "structured_composition",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Selection and integer composition provide no secrecy or authentication.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: descriptor.description,
                alphabet: "JSON values retain their type. Integer operations use exact signed JSON integers.",
                key: "No key.",
                key_advancement: "Not applicable.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "JSON: 256 KiB, depth 16, 4096 nodes. Integers: -9007199254740991 through 9007199254740991. Interfaces may impose lower ceilings.",
            rust_api: api[0],
            python_api: api[1],
            references: &[
                "https://github.com/cubearc/cube/blob/main/docs/reference/structured-composition.md",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 6] = [
    profile(
        Descriptor {
            id: "json.parse",
            category: "text",
            description: "Parse bounded inert JSON; reject duplicate keys, trailing data and excessive bytes/depth/nodes.",
            input_type: "text",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose parse-json",
            example: "cube compose parse-json --text '{\"dimensions\":[3,5]}'",
        },
        "Parse JSON",
        [
            "cube_core::composition::structured::parse_json",
            "parse_json",
        ],
    ),
    profile(
        Descriptor {
            id: "json.select",
            category: "text",
            description: "Select a field or array index by JSON Pointer, preserving its JSON type and distinguishing null from an absent field.",
            input_type: "json",
            output_type: "json",
            parameters: &[Parameter {
                name: "pointer",
                description: "JSON Pointer: / separates tokens; ~0 means ~ and ~1 means /. Empty selects the root.",
                required: false,
                default: Some(ParameterValue::Text("")),
                constraints: ParameterType::Text { choices: &[] },
            }],
            deterministic: true,
            reversible: Some(false),
            command: "compose select",
            example: "cube compose select --json '{\"dimensions\":[3,5]}' --pointer /dimensions",
        },
        "Select field",
        [
            "cube_core::composition::structured::select_field",
            "select_field",
        ],
    ),
    profile(
        Descriptor {
            id: "json.text",
            category: "text",
            description: "Use an actual JSON string as text; reject numbers, null, arrays and objects without coercion.",
            input_type: "json",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose json-text",
            example: "cube compose json-text --json '\"Gur xrl.\"'",
        },
        "Use text value",
        ["cube_core::composition::structured::json_text", "json_text"],
    ),
    profile(
        Descriptor {
            id: "text.integer",
            category: "numbers",
            description: "Parse exact signed decimal integer text; reject spaces, plus signs, leading zeros, fractions, exponents and out-of-range values.",
            input_type: "text",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose integer",
            example: "cube compose integer --text 7",
        },
        "Parse integer",
        [
            "cube_core::composition::structured::parse_integer",
            "parse_integer",
        ],
    ),
    profile(
        Descriptor {
            id: "number.maximum",
            category: "numbers",
            description: "Find the maximum of 1..4096 exact signed JSON integers; empty lists, non-integers and values outside the exact range fail.",
            input_type: "json",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose maximum",
            example: "cube compose maximum --values '[3,5]'",
        },
        "Maximum",
        ["cube_core::composition::structured::maximum", "maximum"],
    ),
    profile(
        Descriptor {
            id: "number.add",
            category: "numbers",
            description: "Add two exact signed JSON integers; reject invalid operands and sums outside the exact range without coercion or wrapping.",
            input_type: "json.pair",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose add",
            example: "cube compose add --left 5 --right 7",
        },
        "Add",
        ["cube_core::composition::structured::add", "add_integers"],
    ),
];
pub(super) const OPERATIONS: &[Descriptor] = &m::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED).with_contracts(contracts::CONTRACTS);
