//! Contract for single-scan baseline JPEG coefficient evidence.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::json;

const fn unsigned(name: &'static str, detail: &'static str, min: u64, max: u64) -> o::Parameter {
    o::Parameter {
        name,
        description: detail,
        required: true,
        default: None,
        constraints: o::ParameterType::Unsigned { min, max },
    }
}
const PARAMETERS: &[o::Parameter] = &[
    o::Parameter {
        name: "profile",
        description: "Baseline one-scan quantized AC magnitude parity.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text {
            choices: &["baseline_ac_parity"],
        },
    },
    unsigned(
        "component_id",
        "Exact identifier in the JPEG frame header.",
        0,
        255,
    ),
    unsigned(
        "start_bit",
        "Zero-based offset among the selected nonzero AC coefficients.",
        0,
        MAX_BITS as u64,
    ),
    unsigned(
        "max_bits",
        "Maximum count of extracted bits.",
        1,
        MAX_BITS as u64,
    ),
];
const DESCRIPTOR: o::Descriptor = o::Descriptor {
    id: EXTRACT,
    category: "image",
    description: "Inspect baseline JPEG quantized AC magnitude parity in row-major block and zigzag AC order.",
    input_type: "bytes",
    output_type: "image.jpeg.coefficients",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: Some(false),
    command: "image jpeg-coefficients",
    example: "cube image jpeg-coefficients --profile baseline_ac_parity --component-id 1 --start-bit 0 --max-bits 16 --file source.jpg",
};
const PROFILED: [m::Entry; 1] = [{
    m::Entry {
        descriptor: o::Descriptor {
            id: EXTRACT,
            ..DESCRIPTOR
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: "JPEG coefficient parity",
            family: "image_bits",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Bit evidence depends on the original JPEG quantized coefficients. Recompression can change all extracted evidence.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: DESCRIPTOR.description,
                alphabet: "Nonzero AC magnitudes, one explicitly selected component.",
                key: "No key.",
                key_advancement: "MSB-first packed evidence.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "16 MiB JPEG bytes, 1048576 coefficients, 1048576 extracted bits. Single scan and bounded tables and markers.",
            rust_api: "cube_core::image::jpeg_coefficients::extract",
            python_api: "image_jpeg_coefficients",
            references: &[
                "https://www.itu.int/rec/T-REC-T.81-199209-I/en",
                "https://docs.rs/jpxl-jpeg/0.3.0/jpxl_jpeg/",
            ],
        }),
    }
}];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const CONTRACTS: &[c::Entry] = &[c::Entry {
    operation_version: 1,
    descriptor: DESCRIPTOR,
    inputs: &[c::Port {
        name: "input",
        semantic_type: "bytes",
        schema: c::bytes_schema::<MAX_INPUT>,
        required: true,
    }],
    parameters: c::schema::<Settings>,
    outputs: &[
        c::Port {
            name: "bits",
            semantic_type: "bytes",
            schema: c::bytes_schema::<{ MAX_BITS / 8 }>,
            required: true,
        },
        c::Port {
            name: "report",
            semantic_type: "image.jpeg.coefficients",
            schema: c::schema::<Report>,
            required: true,
        },
    ],
    limits: c::Limits {
        max_input_bytes: Some(MAX_INPUT as u64),
        max_output_bytes: Some((MAX_BITS / 8 + 256) as u64),
        max_items: Some(MAX_COEFFICIENTS as u64),
        max_work: Some((MAX_INPUT + 64 * MAX_COEFFICIENTS) as u64),
        max_memory_bytes: Some(64 << 20),
    },
    fixture: || {
        let input = include_bytes!("fixture.jpg");
        let result = extract(
            input,
            Settings {
                profile: Profile::BaselineAcParity,
                component_id: 1,
                start_bit: 3,
                max_bits: 19,
            },
        )?;
        Ok(c::Fixture {
            inputs: json!({"input":{"base64":crate::encoding::base64_encode(input)}}),
            parameters: json!({"profile":"baseline_ac_parity","component_id":1,"start_bit":3,"max_bits":19}),
            outputs: json!({"bits":c::values::Bytes{data:result.bits},"report":result.report}),
        })
    },
}];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
