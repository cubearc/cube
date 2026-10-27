//! Real Core execution checked against independent known answers.
use super::*;
use crate::{errors::ErrorCode, operations::contracts::Fixture};
use serde_json::{Value, json};
pub(crate) fn bifid_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square": "EXTRAKLMPOHWZQDGVUSIFCBYN", "period": 7}"##)
            .expect("static parameters");
    let p: Bifid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ODDPERIODSAREPOPULAR";
    let actual = json!({"text":bifid(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "MWEINGIMGEOYYRLVEYWY"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.bifid.encode: {actual} != {expected}"),
        )
        .operation("cipher.bifid.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ODDPERIODSAREPOPULAR"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn bifid_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square": "EXTRAKLMPOHWZQDGVUSIFCBYN", "period": 7}"##)
            .expect("static parameters");
    let p: Bifid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "MWEINGIMGEOYYRLVEYWY";
    let actual = json!({"text":bifid(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "ODDPERIODSAREPOPULAR"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.bifid.decode: {actual} != {expected}"),
        )
        .operation("cipher.bifid.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "MWEINGIMGEOYYRLVEYWY"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn trifid_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet": "EXTRAODINYBCFGHJKLMPQSUVWZ#", "period": 10}"##)
            .expect("static parameters");
    let p: Trifid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "TRIFIDSAREFRACTIONATEDCIPHERS";
    let actual = json!({"text":trifid(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "EYMXVUCRYYYYEAYVYOVVXITDPATHE"}"##)
        .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.trifid.encode: {actual} != {expected}"),
        )
        .operation("cipher.trifid.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "TRIFIDSAREFRACTIONATEDCIPHERS"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn trifid_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet": "EXTRAODINYBCFGHJKLMPQSUVWZ#", "period": 10}"##)
            .expect("static parameters");
    let p: Trifid = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "EYMXVUCRYYYYEAYVYOVVXITDPATHE";
    let actual = json!({"text":trifid(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "TRIFIDSAREFRACTIONATEDCIPHERS"}"##)
        .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.trifid.decode: {actual} != {expected}"),
        )
        .operation("cipher.trifid.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "EYMXVUCRYYYYEAYVYOVVXITDPATHE"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn adfgx_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ", "key": {"kind": "keyword", "text": "CAB"}}"##,
    )
    .expect("static parameters");
    let p: Adfgx = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABC";
    let actual = json!({"text":adfgx(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "AAAFAD"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.adfgx.encode: {actual} != {expected}"),
        )
        .operation("cipher.adfgx.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABC"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn adfgx_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"square": "ABCDEFGHIKLMNOPQRSTUVWXYZ", "key": {"kind": "keyword", "text": "CAB"}}"##,
    )
    .expect("static parameters");
    let p: Adfgx = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "AAAFAD";
    let actual = json!({"text":adfgx(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABC"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.adfgx.decode: {actual} != {expected}"),
        )
        .operation("cipher.adfgx.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "AAAFAD"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn adfgvx_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"square": "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789", "key": {"kind": "keyword", "text": "CAB"}}"##).expect("static parameters");
    let p: Adfgvx = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "AZ09";
    let actual = json!({"text":adfgvx(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "AVXVFADX"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.adfgvx.encode: {actual} != {expected}"),
        )
        .operation("cipher.adfgvx.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "AZ09"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn adfgvx_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"square": "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789", "key": {"kind": "keyword", "text": "CAB"}}"##).expect("static parameters");
    let p: Adfgvx = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "AVXVFADX";
    let actual = json!({"text":adfgvx(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "AZ09"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.adfgvx.decode: {actual} != {expected}"),
        )
        .operation("cipher.adfgvx.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "AVXVFADX"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn nihilist_substitution_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square": "SIMPLEABCDFGHKNOQRTUVWXYZ", "key": "EASY"}"##)
            .expect("static parameters");
    let p: Nihilist = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THEEARLYBIRD";
    let actual = json!({"tokens":nihilist_encode(text,&p)?});
    let expected: Value =
        serde_json::from_str(r##"{"tokens": [65, 55, 32, 75, 43, 65, 26, 108, 44, 34, 54, 79]}"##)
            .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(ErrorCode::InvalidInput,format!("known answer mismatch for cipher.nihilist_substitution.encode: {actual} != {expected}")).operation("cipher.nihilist_substitution.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "THEEARLYBIRD"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn nihilist_substitution_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"square": "SIMPLEABCDFGHKNOQRTUVWXYZ", "key": "EASY"}"##)
            .expect("static parameters");
    let p: Nihilist = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let tokens: Vec<u16> = vec![65, 55, 32, 75, 43, 65, 26, 108, 44, 34, 54, 79];
    let actual = json!({"text":nihilist_decode(&tokens,&p)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "THEEARLYBIRD"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(ErrorCode::InvalidInput,format!("known answer mismatch for cipher.nihilist_substitution.decode: {actual} != {expected}")).operation("cipher.nihilist_substitution.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(
            r##"{"tokens": [65, 55, 32, 75, 43, 65, 26, 108, 44, 34, 54, 79]}"##,
        )
        .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn bazeries_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key": 3752, "square": "THREOUSANDVFIYWBCGKLMPQXZ"}"##)
            .expect("static parameters");
    let p: Bazeries = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "SIMPLESUBSTITUTIONPLUSTRANSPOSITION";
    let actual = json!({"text":bazeries(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "ACYYUXYMRQKXKCKGCRQIYITNKYXKCYGQGCI"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.bazeries.encode: {actual} != {expected}"),
        )
        .operation("cipher.bazeries.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "SIMPLESUBSTITUTIONPLUSTRANSPOSITION"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn bazeries_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key": 3752, "square": "THREOUSANDVFIYWBCGKLMPQXZ"}"##)
            .expect("static parameters");
    let p: Bazeries = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ACYYUXYMRQKXKCKGCRQIYITNKYXKCYGQGCI";
    let actual = json!({"text":bazeries(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "SIMPLESUBSTITUTIONPLUSTRANSPOSITION"}"##)
            .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.bazeries.decode: {actual} != {expected}"),
        )
        .operation("cipher.bazeries.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ACYYUXYMRQKXKCKGCRQIYITNKYXKCYGQGCI"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn fractionated_morse_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"alphabet": "ROUNDTABLECFGHIJKMPQSVWXYZ"}"##)
        .expect("static parameters");
    let p: morse::FractionatedMorse =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "COME AT ONCE";
    let actual =
        serde_json::to_value(morse::fractionated_morse(text, &p, false)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "COME AT ONCE", "ciphertext": "CBIILTMHVVFL", "morse": "-.-.x---x--x.xx.-x-xx---x-.x-.-.x.xx", "padding_count": 2, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.fractionated_morse.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.fractionated_morse.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "COME AT ONCE"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn fractionated_morse_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"alphabet": "ROUNDTABLECFGHIJKMPQSVWXYZ", "padding_count": 2}"##)
            .expect("static parameters");
    let p: morse::FractionatedMorse =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "CBIILTMHVVFL";
    let actual =
        serde_json::to_value(morse::fractionated_morse(text, &p, true)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "COME AT ONCE", "ciphertext": "CBIILTMHVVFL", "morse": "-.-.x---x--x.xx.-x-xx---x-.x-.-.x.xx", "padding_count": 2, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.fractionated_morse.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.fractionated_morse.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "CBIILTMHVVFL"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn morbit_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"digits": "958427136"}"##).expect("static parameters");
    let p: morse::Morbit = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ONCE UPON A TIME";
    let actual = serde_json::to_value(morse::morbit(text, &p, false)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "ONCE UPON A TIME", "ciphertext": "27435881512827465679378", "morse": "---x-.x-.-.x.xx..-x.--.x---x-.xx.-xx-x..x--x.x", "padding_count": 1, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.morbit.encode: {actual} != {expected}"),
        )
        .operation("cipher.morbit.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ONCE UPON A TIME"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn morbit_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"digits": "958427136", "padding_count": 1}"##)
            .expect("static parameters");
    let p: morse::Morbit = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "27435881512827465679378";
    let actual = serde_json::to_value(morse::morbit(text, &p, true)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "ONCE UPON A TIME", "ciphertext": "27435881512827465679378", "morse": "---x-.x-.-.x.xx..-x.--.x---x-.xx.-xx-x..x--x.x", "padding_count": 1, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.morbit.decode: {actual} != {expected}"),
        )
        .operation("cipher.morbit.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "27435881512827465679378"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn pollux_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"classes": ".x-.x.--x."}"##).expect("static parameters");
    let p: morse::Pollux = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "LUCK HELPS";
    let actual = serde_json::to_value(morse::pollux(text, &p, false)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "LUCK HELPS", "ciphertext": "023519064732586971403598013259406738590", "morse": ".-..x..-x-.-.x-.-xx....x.x.-..x.--.x...", "padding_count": 0, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.pollux.encode: {actual} != {expected}"),
        )
        .operation("cipher.pollux.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "LUCK HELPS"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn pollux_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"classes": ".x-.x.--x."}"##).expect("static parameters");
    let p: morse::Pollux = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "023519064732586971403598013259406738590";
    let actual = serde_json::to_value(morse::pollux(text, &p, true)?).expect("typed report");
    let expected:Value=serde_json::from_str(r##"{"text": "LUCK HELPS", "ciphertext": "023519064732586971403598013259406738590", "morse": ".-..x..-x-.-.x-.-xx....x.x.-..x.--.x...", "padding_count": 0, "padding_ambiguous": false}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.pollux.decode: {actual} != {expected}"),
        )
        .operation("cipher.pollux.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "023519064732586971403598013259406738590"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
