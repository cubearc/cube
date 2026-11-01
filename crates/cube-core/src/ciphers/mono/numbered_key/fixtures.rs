use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;
pub(crate) fn parameters() -> Parameters {
    serde_json::from_value(json!({"alphabet":"abcdefghijklmnopqrstuvwxyz","key":"I like ciphers.","preparation":"ascii_lowercase_letters","rotation":18,"origin":"0","width":2,"choice":"round_robin"})).expect("fixture parameters")
}
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let p = parameters();
    let result = if decode {
        super::decode(
            &[
                "23".into(),
                "15".into(),
                "20".into(),
                "11".into(),
                "13".into(),
                "17".into(),
            ],
            &p,
        )?
    } else {
        super::encode("A ee iii", &p)?
    };
    if result.text != "aeeiii"
        || result.tokens != ["23", "15", "20", "11", "13", "17"]
        || result
            .table
            .iter()
            .find(|e| e.symbol == 'a')
            .is_none_or(|e| e.tokens != ["23"])
        || result
            .table
            .iter()
            .find(|e| e.symbol == 'i')
            .is_none_or(|e| e.tokens != ["11", "13", "17"])
    {
        return Err(invalid("fixture"));
    }
    Ok(Fixture {
        inputs: if decode {
            json!({"tokens":result.tokens})
        } else {
            json!({"text":"A ee iii"})
        },
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
