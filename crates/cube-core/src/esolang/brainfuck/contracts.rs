//! Version two repairs the missing program source in the initial direct contract.
//! Public Rust/CLI/Python calls and ordinary terminal output remain unchanged.
use super::*;
use crate::operations::contracts::{self as c, Entry, Fixture, Port, schema, values};
use schemars::Schema;
use serde_json::json;

fn source() -> Schema {
    Schema::try_from(json!({"type":"string","maxLength":MAX_SOURCE_BYTES,"description":"Strict Brainfuck source; the parser also enforces a 1 MiB UTF-8 byte bound."})).expect("source schema")
}
fn input() -> Schema {
    Schema::try_from(json!({"type":"array","maxItems":MAX_INPUT_BYTES,"items":{"type":"integer","minimum":0,"maximum":255}})).expect("input schema")
}
const INPUTS: &[Port] = &[
    Port {
        name: "source",
        semantic_type: "text",
        schema: source,
        required: true,
    },
    Port {
        name: "input",
        semantic_type: "bytes.array",
        schema: input,
        required: true,
    },
];
fn parameters(index: usize) -> Schema {
    let mut value = c::descriptor_parameters(&OPERATIONS[index]).to_value();
    value["properties"]
        .as_object_mut()
        .expect("parameters")
        .remove("input");
    Schema::try_from(value).expect("parameters schema")
}
fn execute_parameters() -> Schema {
    parameters(0)
}
fn report_parameters() -> Schema {
    parameters(1)
}
fn compare_parameters() -> Schema {
    parameters(2)
}
fn execute_fixture() -> Result<Fixture, OperationError> {
    let output = execute(",.", &[255], Limits::default())?;
    assert_eq!(output, [255]);
    Ok(Fixture {
        inputs: json!({"source":",.","input":[255]}),
        parameters: json!({}),
        outputs: json!({"output":values::Bytes {data:output}}),
    })
}
fn report_fixture() -> Result<Fixture, OperationError> {
    let output = Program::parse(",.")?.report(
        &[255],
        Options::default(),
        Limits::default(),
        Diagnostics::default(),
    )?;
    assert!(output.completed);
    assert_eq!(output.output, [255]);
    Ok(Fixture {
        inputs: json!({"source":",.","input":[255]}),
        parameters: json!({}),
        outputs: json!({"output":output}),
    })
}
fn compare_fixture() -> Result<Fixture, OperationError> {
    let output = compare(
        ",.",
        &[255],
        &[8, 16, 32],
        EofPolicy::Error,
        Limits::default(),
        ComparisonLimits::default(),
    )?;
    assert!(output.all_completed);
    assert_eq!(output.equal_completed_outputs, Some(true));
    for run in &output.runs {
        assert_eq!(run.output, [255]);
    }
    Ok(Fixture {
        inputs: json!({"source":",.","input":[255]}),
        parameters: json!({"widths":[8,16,32]}),
        outputs: json!({"output":output}),
    })
}
const LIMITS: c::Limits = c::Limits {
    max_input_bytes: Some(MAX_SOURCE_BYTES as u64 + MAX_INPUT_BYTES as u64),
    max_output_bytes: Some(MAX_OUTPUT_BYTES as u64),
    max_items: Some(MAX_COMMANDS as u64),
    max_work: Some(MAX_INSTRUCTIONS),
    max_memory_bytes: Some(MAX_MEMORY_BYTES as u64),
};
pub(crate) const CONTRACTS: &[Entry] = &[
    Entry {
        operation_version: 2,
        descriptor: OPERATIONS[0],
        inputs: INPUTS,
        parameters: execute_parameters,
        outputs: &[Port {
            name: "output",
            semantic_type: "bytes",
            schema: schema::<values::Bytes>,
            required: true,
        }],
        limits: LIMITS,
        fixture: execute_fixture,
    },
    Entry {
        operation_version: 2,
        descriptor: OPERATIONS[1],
        inputs: INPUTS,
        parameters: report_parameters,
        outputs: &[Port {
            name: "output",
            semantic_type: "brainfuck_report",
            schema: schema::<Report>,
            required: true,
        }],
        limits: LIMITS,
        fixture: report_fixture,
    },
    Entry {
        operation_version: 2,
        descriptor: OPERATIONS[2],
        inputs: INPUTS,
        parameters: compare_parameters,
        outputs: &[Port {
            name: "output",
            semantic_type: "brainfuck_comparison",
            schema: schema::<Comparison>,
            required: true,
        }],
        limits: LIMITS,
        fixture: compare_fixture,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_and_bytes_are_separate_required_ports_with_exact_bounds() {
        for entry in CONTRACTS {
            let c = c::describe(&entry.descriptor).unwrap();
            assert_eq!(c.operation_version, 2);
            c.validate_fixture(&(entry.fixture)().unwrap()).unwrap();
            assert!(c.validate_inputs(&json!({"input":[255]})).is_err());
            assert!(
                c.validate_inputs(&json!({"source":",.","input":[256]}))
                    .is_err()
            );
            c.validate_inputs(&json!({"source":"+".repeat(MAX_SOURCE_BYTES),"input":[]}))
                .unwrap();
            assert!(
                c.validate_inputs(&json!({"source":"+".repeat(MAX_SOURCE_BYTES+1),"input":[]}))
                    .is_err()
            );
            assert!(c.parameters.as_value()["properties"].get("input").is_none());
        }
    }
}
