//! Preserve the existing manifest and expose its PNGs as an aligned optional port.
use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, schema, values};
use schemars::Schema;
use serde_json::json;
fn parameters() -> Schema {
    c::descriptor_parameters(&OPERATIONS[0])
}
fn fixture() -> Result<Fixture, OperationError> {
    let image = Image::from_bytes(2, 1, ColorModel::Gray, &[0, 255], &[], &[])?;
    let result = image.color_layers(None, ColorOutput::Mask, 16, 16)?;
    assert_eq!(result.outputs.len(), 2);
    assert_eq!(Image::decode(&result.outputs[0].png)?.samples(), [255, 0]);
    let images: Vec<_> = result
        .outputs
        .iter()
        .map(|r| values::Bytes {
            data: r.png.clone(),
        })
        .collect();
    Ok(Fixture {
        inputs: json!({"input":values::Image::from(&image)}),
        parameters: json!({}),
        outputs: json!({"output":result,"images":images}),
    })
}
pub(crate) const CONTRACTS: &[Entry] = &[Entry {
    operation_version: 1,
    descriptor: OPERATIONS[0],
    inputs: &[Port {
        name: "input",
        semantic_type: "image.samples",
        schema: schema::<values::Image>,
        required: true,
    }],
    parameters,
    outputs: &[
        Port {
            name: "output",
            semantic_type: "image.color_exports",
            schema: schema::<ColorExports>,
            required: true,
        },
        Port {
            name: "images",
            semantic_type: "image.png_collection",
            schema: schema::<Vec<values::Bytes>>,
            required: false,
        },
    ],
    limits: c::Limits {
        max_input_bytes: None,
        max_output_bytes: Some(MAX_BYTES as u64),
        max_items: Some(MAX_OUTPUTS as u64),
        max_work: Some(MAX_WORK),
        max_memory_bytes: None,
    },
    fixture,
}];
