//! Width/height facts from an already validated image, without pixel processing.
pub(crate) mod execution;
use super::Image;
use crate::{
    errors::OperationError,
    operations::{
        Descriptor,
        contracts::{self as c, Entry, Fixture, Port, values},
        metadata as m,
    },
};
use schemars::{JsonSchema, Schema};
use serde::Serialize;
use serde_json::json;
/// Exact width and height of a decoded image.
#[derive(Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    /// Exact [width, height] in pixels; neither value is inferred from a preview.
    pub dimensions: [u32; 2],
}
/// Read dimensions from a validated Core image without processing its pixels.
pub fn inspect(image: &Image) -> Dimensions {
    Dimensions {
        dimensions: [image.width(), image.height()],
    }
}
const DESCRIPTION: &str = "Read exact image width and height in pixels as an ordered dimensions array; no resizing or OCR.";
const PROFILED: [m::Entry; 1] = [m::Entry {
    descriptor: Descriptor {
        id: "image.dimensions",
        category: "image",
        description: DESCRIPTION,
        input_type: "image.samples",
        output_type: "json",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "image dimensions",
        example: "cube image dimensions sample.png",
    },
    metadata: Some(m::Metadata {
        schema_version: 1,
        canonical_name: "Image dimensions",
        family: "image_facts",
        aliases: &[],
        security: m::Security {
            classification: m::Classification::BinaryTransform,
            limitations: "Dimensions describe the decoded image; no recognition or validation of a challenge answer.",
            private_input: false,
            private_output: false,
            sensitive_parameters: &[],
        },
        profile: m::Profile {
            variant: DESCRIPTION,
            alphabet: "Width then height, in pixels.",
            key: "No key.",
            key_advancement: "Not applicable.",
            block_bits: None,
            output_bits: None,
        },
        resources: &[],
        resource_notes: "Reads two fields from a validated, bounded Core Image; does not allocate pixel buffers.",
        rust_api: "cube_core::image::dimensions::inspect",
        python_api: "image_dimensions",
        references: &[
            "https://github.com/cubearc/cube/blob/main/docs/reference/structured-composition.md",
        ],
    }),
}];
const OPERATIONS: &[Descriptor] = &m::descriptors(&PROFILED);
fn parameters() -> Schema {
    c::descriptor_parameters(&PROFILED[0].descriptor)
}
fn fixture() -> Result<Fixture, OperationError> {
    let image = Image::from_bytes(2, 3, super::ColorModel::Rgb, &[0; 18], &[], &[])?;
    let output = inspect(&image);
    assert_eq!(output.dimensions, [2, 3]);
    Ok(Fixture {
        inputs: json!({"input":values::Image::from(&image)}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
const CONTRACTS: &[Entry] = &[Entry {
    operation_version: 1,
    descriptor: PROFILED[0].descriptor,
    inputs: &[Port {
        name: "input",
        semantic_type: "image.samples",
        schema: c::schema::<values::Image>,
        required: true,
    }],
    parameters,
    outputs: &[Port {
        name: "output",
        semantic_type: "json",
        schema: c::schema::<Dimensions>,
        required: true,
    }],
    limits: c::Limits {
        max_input_bytes: None,
        max_output_bytes: Some(64),
        max_items: Some(2),
        max_work: Some(2),
        max_memory_bytes: None,
    },
    fixture,
}];
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED)
        .with_contracts(CONTRACTS)
        .with_executors(execution::ENTRIES);
