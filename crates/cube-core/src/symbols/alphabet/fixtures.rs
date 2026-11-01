use super::*;
use crate::{
    errors::{ErrorCode, OperationError},
    operations::contracts::Fixture,
};
use serde_json::json;
pub(crate) fn encode() -> Result<Fixture, OperationError> {
    let p = Parameters {
        profile: Profile::AurebeshUcsur2016V1,
    };
    let units = vec!["A".into(), "CH".into(), "C".into(), "H".into(), "Z".into()];
    let result = super::encode(&units, &p)?;
    if result.tokens.0 != ["\u{e890}", "\u{e8aa}", "\u{e892}", "\u{e897}", "\u{e8a9}"] {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "source-known Aurebesh mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"units":units}),
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(result).expect("report"),
    })
}
pub(crate) fn decode() -> Result<Fixture, OperationError> {
    let p = Parameters {
        profile: Profile::StandardGalacticUcsur2019V1,
    };
    let tokens = vec![
        "\u{eb4a}".into(),
        "\u{eb44}".into(),
        "\u{eb44}".into(),
        "\u{eb4d}".into(),
        "\u{eb5f}".into(),
    ];
    let result = super::decode(&tokens, &p)?;
    for (index, unit) in ["K", "E", "E", "N", "."].iter().enumerate() {
        if result.readings[index].alternatives != [*unit]
            || result.readings[index].token_index != index as u32
        {
            return Err(OperationError::new(
                ErrorCode::InvalidInput,
                "source-known SGA mismatch",
            ));
        }
    }
    Ok(Fixture {
        inputs: json!({"tokens":tokens}),
        parameters: serde_json::to_value(p).expect("parameters"),
        outputs: serde_json::to_value(result).expect("report"),
    })
}
