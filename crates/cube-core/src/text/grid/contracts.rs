//! The scalar grid remains available; an explicit color mapping adds an image port.
use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, schema, values};
use schemars::Schema;
use serde_json::json;
fn parameters() -> Schema {
    c::descriptor_parameters(&OPERATIONS[1])
}
fn fixture() -> Result<Fixture, OperationError> {
    let colors = r#"[["0","000000ff"],["1","ffffffff"]]"#;
    let grid = render("0110", 2, false, Traversal::Row)?;
    assert_eq!(grid.rows, vec![vec!["0", "1"], vec!["1", "0"]]);
    let image = grid.image(&parse_colors(colors)?)?;
    assert_eq!(
        image.samples(),
        [
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255
        ]
    );
    Ok(Fixture {
        inputs: json!({"input":"0110"}),
        parameters: json!({"width":2,"colors":colors}),
        outputs: json!({"output":grid,"image":values::Image::from(&image)}),
    })
}
pub(crate) const CONTRACTS: &[Entry] = &[Entry {
    operation_version: 1,
    descriptor: OPERATIONS[1],
    inputs: &[Port {
        name: "input",
        semantic_type: "text",
        schema: schema::<String>,
        required: true,
    }],
    parameters,
    outputs: &[
        Port {
            name: "output",
            semantic_type: "text.grid",
            schema: schema::<Grid>,
            required: true,
        },
        Port {
            name: "image",
            semantic_type: "image.samples",
            schema: schema::<values::Image>,
            required: false,
        },
    ],
    limits: c::Limits {
        max_input_bytes: Some(crate::text::MAX_INPUT as u64),
        max_output_bytes: None,
        max_items: Some(MAX_UNITS as u64),
        max_work: None,
        max_memory_bytes: None,
    },
    fixture,
}];
