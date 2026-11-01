use super::*;
use crate::{
    errors::{ErrorCode, OperationError},
    operations::contracts::Fixture,
};
use serde_json::json;
fn parameters() -> Parameters {
    Parameters {
        profile: Profile::Gajewski1902MonosyllablesV1,
    }
}
pub(crate) fn encode() -> Result<Fixture, OperationError> {
    let p = parameters();
    let terms = vec![
        "non".into(),
        "oui".into(),
        "à".into(),
        "si (condition)".into(),
    ];
    let result = super::encode(&terms, &p)?;
    if result.tokens.0 != ["do", "si", "fa", "sol"] {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "source-known Solresol encoding mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"terms":terms}),
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(result).expect("report"),
    })
}
pub(crate) fn decode() -> Result<Fixture, OperationError> {
    let p = parameters();
    let tokens = vec![
        "d".into(),
        "si".into(),
        "r".into(),
        "ré".into(),
        "do".into(),
        "do".into(),
    ];
    let result = super::decode(&tokens, &p)?;
    if result.readings[0].alternatives != ["non", "pas", "point", "ne", "ni"]
        || result.readings[1].alternatives != ["oui", "soit", "volontiers", "d'accord"]
        || result.readings[2].alternatives != ["et", "ainsi que"]
        || result.readings[5].token_index != 5
    {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "source-known Solresol readings mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"tokens":tokens}),
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(result).expect("report"),
    })
}
