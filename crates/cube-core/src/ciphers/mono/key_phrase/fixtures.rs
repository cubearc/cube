use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let p = Parameters {
        phrase: "GIVEMELIBERTYORGIVEMEDEATH".into(),
    };
    let text = if decode {
        "I"
    } else {
        "A CIPHERTEXT LETTER MAY STAND FOR MORE THAN ONE PLAINTEXT LETTER."
    };
    let result = transform(text, &p, decode)?;
    if decode {
        if result.text.is_some()
            || !result.ambiguous
            || result.records.len() != 1
            || result.records[0].alternatives != ["B", "H", "Q"]
        {
            return Err(invalid("fixture"));
        }
    } else if result.text.as_deref()
        != Some("G VBGIMVMMAM TMMMMV YGT EMGOE ERV YRVM MIGO ROM GTGBOMMAM TMMMMV.")
    {
        return Err(invalid("fixture"));
    }
    Ok(Fixture {
        inputs: json!({"text":text}),
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(result).expect("report"),
    })
}
pub(crate) fn encode() -> Result<Fixture, OperationError> {
    fixture(false)
}
pub(crate) fn decode() -> Result<Fixture, OperationError> {
    fixture(true)
}
