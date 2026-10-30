use crate::operations::execution::{self as e, Entry, Failure, Representation::*, Result};
use serde_json::{Value, json};
fn defaults(p: &mut Value, b: e::Budget) {
    p["max_output_bytes"] = json!(b.output_bytes.min(super::MAX_OUTPUT_BYTES));
    p["max_memory_bytes"] = json!(b.memory_bytes.min(super::MAX_MEMORY_BYTES));
    p["max_instructions"] = json!(b.instructions.min(super::MAX_INSTRUCTIONS));
}
fn configure(p: &Value, b: e::Budget) -> Result<()> {
    for (name, max) in [
        ("max_output_bytes", b.output_bytes as u64),
        ("max_memory_bytes", b.memory_bytes as u64),
        ("max_instructions", b.instructions),
    ] {
        let value = p[name].as_u64().ok_or(Failure("invalid_parameters"))?;
        if value == 0 || value > max {
            return Err(Failure("input_limit"));
        }
    }
    Ok(())
}
fn execute(i: &Value, p: &Value, _: e::Budget) -> Result<Value> {
    let input: Vec<u8> =
        serde_json::from_value(i["input"].clone()).map_err(|_| Failure("contract_mismatch"))?;
    let limits = super::Limits {
        max_instructions: p["max_instructions"]
            .as_u64()
            .ok_or(Failure("invalid_parameters"))?,
        max_output_bytes: serde_json::from_value(p["max_output_bytes"].clone())
            .map_err(|_| Failure("invalid_parameters"))?,
        max_memory_bytes: serde_json::from_value(p["max_memory_bytes"].clone())
            .map_err(|_| Failure("invalid_parameters"))?,
    };
    let options = super::Options {
        cell_bits: serde_json::from_value(p["cell_bits"].clone())
            .map_err(|_| Failure("invalid_parameters"))?,
        eof: super::EofPolicy::parse(e::text(p, "eof")?)
            .map_err(|_| Failure("invalid_parameters"))?,
    };
    let data = super::Program::parse(e::text(i, "source")?)
        .and_then(|program| program.execute(&input, options, limits))
        .map_err(|_| Failure("operation_failed"))?;
    Ok(json!({"output":e::byte_value(data)}))
}
pub(crate) const ENTRIES: &[Entry] = &[Entry {
    defaults: Some(defaults),
    configure: Some(configure),
    ..Entry::pure(
        "esolang.brainfuck.execute",
        &[Text, ByteArray],
        &[Bytes],
        execute,
    )
}];
