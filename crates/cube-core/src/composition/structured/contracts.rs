use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port};
use schemars::Schema;
use serde_json::json;
fn text() -> Schema {
    Schema::try_from(json!({"type":"string","maxLength":MAX_DOCUMENT_BYTES})).expect("bounded text")
}
fn integer_text() -> Schema {
    Schema::try_from(json!({"type":"string","maxLength":17,"pattern":"^-?(0|[1-9][0-9]*)$"}))
        .expect("integer text")
}
fn json_value() -> Schema {
    Schema::try_from(json!({"$defs":{"value":{"anyOf":[{"type":"null"},{"type":"boolean"},{"type":"number","minimum":-MAX_INTEGER,"maximum":MAX_INTEGER},{"type":"string","maxLength":MAX_DOCUMENT_BYTES},{"type":"array","maxItems":MAX_NODES,"items":{"$ref":"#/$defs/value"}},{"type":"object","maxProperties":MAX_NODES,"additionalProperties":{"$ref":"#/$defs/value"}}]}},"$ref":"#/$defs/value"})).expect("JSON value")
}
fn integer() -> Schema {
    Schema::try_from(json!({"type":"integer","minimum":-MAX_INTEGER,"maximum":MAX_INTEGER}))
        .expect("exact integer")
}
fn integers() -> Schema {
    Schema::try_from(json!({"type":"array","minItems":1,"maxItems":MAX_NODES,"items":integer()}))
        .expect("integer list")
}
fn parameters() -> Schema {
    c::descriptor_parameters(&metadata::OPERATIONS[0])
}
fn selection_parameters() -> Schema {
    let mut value = c::descriptor_parameters(&metadata::OPERATIONS[1]).to_value();
    value["properties"]["pointer"]["maxLength"] = json!(MAX_POINTER_BYTES);
    Schema::try_from(value).expect("field parameters")
}
fn parse_fixture() -> Result<Fixture, OperationError> {
    let document = "{\"dimensions\":[3,5],\"caption\":\"é\",\"absent\":null}";
    let output = parse_json(document)?;
    assert_eq!(
        output,
        json!({"dimensions":[3,5],"caption":"é","absent":null})
    );
    Ok(Fixture {
        inputs: json!({"input":document}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
fn select_fixture() -> Result<Fixture, OperationError> {
    let value = json!({"a/b":{"~key":[null,{"size":7}]}});
    let output = select_field(&value, "/a~1b/~0key/1/size")?;
    assert_eq!(output, json!(7));
    Ok(Fixture {
        inputs: json!({"input":value}),
        parameters: json!({"pointer":"/a~1b/~0key/1/size"}),
        outputs: json!({"output":output}),
    })
}
fn text_fixture() -> Result<Fixture, OperationError> {
    let output = json_text(&json!("é\0世界\r\n"))?;
    assert_eq!(output, "é\0世界\r\n");
    Ok(Fixture {
        inputs: json!({"input":"é\0世界\r\n"}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
fn integer_fixture() -> Result<Fixture, OperationError> {
    let output = parse_integer("-17")?;
    assert_eq!(output, -17);
    Ok(Fixture {
        inputs: json!({"input":"-17"}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
fn maximum_fixture() -> Result<Fixture, OperationError> {
    let output = maximum(&[-7, 5, 3])?;
    assert_eq!(output, 5);
    Ok(Fixture {
        inputs: json!({"input":[-7,5,3]}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
fn add_fixture() -> Result<Fixture, OperationError> {
    let output = add(5, 7)?;
    assert_eq!(output, 12);
    Ok(Fixture {
        inputs: json!({"left":5,"right":7}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
const LIMITS: c::Limits = c::Limits {
    max_input_bytes: Some(MAX_DOCUMENT_BYTES as u64),
    max_output_bytes: Some(MAX_DOCUMENT_BYTES as u64),
    max_items: Some(MAX_NODES as u64),
    max_work: None,
    max_memory_bytes: None,
};
const fn port(name: &'static str, semantic_type: &'static str, schema: fn() -> Schema) -> Port {
    Port {
        name,
        semantic_type,
        schema,
        required: true,
    }
}
pub(super) const CONTRACTS: &[Entry] = &[
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[0],
        inputs: &[port("input", "text", text)],
        parameters,
        outputs: &[port("output", "json", json_value)],
        limits: LIMITS,
        fixture: parse_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[1],
        inputs: &[port("input", "json", json_value)],
        parameters: selection_parameters,
        outputs: &[port("output", "json", json_value)],
        limits: LIMITS,
        fixture: select_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[2],
        inputs: &[port("input", "json", json_value)],
        parameters,
        outputs: &[port("output", "text", text)],
        limits: LIMITS,
        fixture: text_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[3],
        inputs: &[port("input", "text", integer_text)],
        parameters,
        outputs: &[port("output", "json", integer)],
        limits: LIMITS,
        fixture: integer_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[4],
        inputs: &[port("input", "json", integers)],
        parameters,
        outputs: &[port("output", "json", integer)],
        limits: LIMITS,
        fixture: maximum_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: metadata::OPERATIONS[5],
        inputs: &[
            port("left", "json", integer),
            port("right", "json", integer),
        ],
        parameters,
        outputs: &[port("output", "json", integer)],
        limits: LIMITS,
        fixture: add_fixture,
    },
];
