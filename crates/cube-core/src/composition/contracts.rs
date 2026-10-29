use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, values};
use schemars::Schema;
use serde_json::json;

fn text() -> Schema {
    Schema::try_from(json!({"type":"string", "maxLength":MAX_BYTES,
        "description":"Exact Unicode text. Combined UTF-8 operand/separator bytes are also bounded by the operation."})).expect("text schema")
}
fn bytes() -> Schema {
    let mut schema = c::schema::<values::Bytes>().to_value();
    schema["properties"]["base64"]["maxLength"] = json!(MAX_BYTES.div_ceil(3) * 4);
    Schema::try_from(schema).expect("byte schema")
}
fn text_parameters() -> Schema {
    let mut schema = c::descriptor_parameters(&OPERATIONS[0]).to_value();
    schema["properties"]["separator"]["maxLength"] = json!(MAX_BYTES);
    Schema::try_from(schema).expect("separator schema")
}
fn byte_parameters() -> Schema {
    c::descriptor_parameters(&OPERATIONS[1])
}
fn text_fixture() -> Result<Fixture, OperationError> {
    let output = concat_text("e\u{301}\0", "世界\r\n", " | ")?;
    assert_eq!(output, "e\u{301}\0 | 世界\r\n");
    Ok(Fixture {
        inputs: json!({"left":"e\u{301}\0", "right":"世界\r\n"}),
        parameters: json!({"separator":" | "}),
        outputs: json!({"output":output}),
    })
}
fn byte_fixture() -> Result<Fixture, OperationError> {
    let output = concat_bytes(&[0, 255], &[128, 10])?;
    assert_eq!(output, [0, 255, 128, 10]);
    Ok(Fixture {
        inputs: json!({"left":values::Bytes {data:vec![0,255]}, "right":values::Bytes {data:vec![128,10]}}),
        parameters: json!({}),
        outputs: json!({"output":values::Bytes {data:output}}),
    })
}
const LIMITS: c::Limits = c::Limits {
    max_input_bytes: Some(MAX_BYTES as u64),
    max_output_bytes: Some(MAX_BYTES as u64),
    max_items: None,
    max_work: Some(MAX_BYTES as u64),
    max_memory_bytes: None,
};
pub(crate) const CONTRACTS: &[Entry] = &[
    Entry {
        operation_version: 1,
        descriptor: OPERATIONS[0],
        inputs: &[
            Port {
                name: "left",
                semantic_type: "text",
                schema: text,
                required: true,
            },
            Port {
                name: "right",
                semantic_type: "text",
                schema: text,
                required: true,
            },
        ],
        parameters: text_parameters,
        outputs: &[Port {
            name: "output",
            semantic_type: "text",
            schema: text,
            required: true,
        }],
        limits: LIMITS,
        fixture: text_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: OPERATIONS[1],
        inputs: &[
            Port {
                name: "left",
                semantic_type: "bytes",
                schema: bytes,
                required: true,
            },
            Port {
                name: "right",
                semantic_type: "bytes",
                schema: bytes,
                required: true,
            },
        ],
        parameters: byte_parameters,
        outputs: &[Port {
            name: "output",
            semantic_type: "bytes",
            schema: bytes,
            required: true,
        }],
        limits: LIMITS,
        fixture: byte_fixture,
    },
];
