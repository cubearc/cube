//! Nullable payload slots align one-to-one with symbols; bytes are never UTF-8 coerced.
use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, schema, values};
use schemars::Schema;
use serde_json::json;
fn parameters() -> Schema {
    c::descriptor_parameters(&OPERATIONS[0])
}
fn fixture() -> Result<Fixture, OperationError> {
    let rows: Vec<_> = include_str!("known-binary.matrix").lines().collect();
    let side = (rows.len() + 8) * 2;
    let mut samples = vec![255; side * side];
    for (y, row) in rows.iter().enumerate() {
        for (x, bit) in row.bytes().enumerate() {
            for dy in 0..2 {
                for dx in 0..2 {
                    samples[((y + 4) * 2 + dy) * side + (x + 4) * 2 + dx] =
                        if bit == b'1' { 0 } else { 255 };
                }
            }
        }
    }
    let image = Image::from_samples(
        side as u32,
        side as u32,
        ColorModel::Gray,
        samples,
        vec![],
        vec![],
    )?;
    let report = image.qr(8)?;
    assert_eq!(
        report.symbols[0].payload.as_deref(),
        Some(&b"Cube\0\xff"[..])
    );
    let payloads: Vec<_> = report
        .symbols
        .iter()
        .map(|s| {
            s.payload
                .as_ref()
                .map(|p| values::Bytes { data: p.clone() })
        })
        .collect();
    Ok(Fixture {
        inputs: json!({"input":values::Image::from(&image)}),
        parameters: json!({}),
        outputs: json!({"output":report,"payloads":payloads}),
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
            semantic_type: "image.qr_report",
            schema: schema::<QrReport>,
            required: true,
        },
        Port {
            name: "payloads",
            semantic_type: "bytes.collection",
            schema: schema::<Vec<Option<values::Bytes>>>,
            required: false,
        },
    ],
    limits: c::Limits {
        max_input_bytes: None,
        max_output_bytes: Some(MAX_QR_PAYLOAD as u64),
        max_items: Some(MAX_QR_SYMBOLS as u64),
        max_work: Some(MAX_QR_PIXELS),
        max_memory_bytes: None,
    },
    fixture,
}];
