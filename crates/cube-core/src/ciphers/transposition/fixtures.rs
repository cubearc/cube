//! Real Core execution checked against independent known answers.
use super::*;
use crate::{errors::ErrorCode, operations::contracts::Fixture};
use serde_json::{Value, json};
pub(crate) fn scytale_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"columns": 3}"##).expect("static parameters");
    let p: Scytale = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEFGH";
    let actual = json!({"text":scytale(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ADGBEHCF"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.scytale.encode: {actual} != {expected}"),
        )
        .operation("cipher.scytale.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn scytale_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"columns": 3}"##).expect("static parameters");
    let p: Scytale = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ADGBEHCF";
    let actual = json!({"text":scytale(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.scytale.decode: {actual} != {expected}"),
        )
        .operation("cipher.scytale.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ADGBEHCF"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn rail_fence_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"rails": 3, "initial_rail": 0, "direction": "down"}"##)
            .expect("static parameters");
    let p: Rails = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "WEAREDISCOVEREDFLEEATONCE";
    let actual = json!({"text":rail_fence(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "WECRLTEERDSOEEFEAOCAIVDEN"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.rail_fence.encode: {actual} != {expected}"),
        )
        .operation("cipher.rail_fence.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "WEAREDISCOVEREDFLEEATONCE"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn rail_fence_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"rails": 3, "initial_rail": 0, "direction": "down"}"##)
            .expect("static parameters");
    let p: Rails = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "WECRLTEERDSOEEFEAOCAIVDEN";
    let actual = json!({"text":rail_fence(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "WEAREDISCOVEREDFLEEATONCE"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.rail_fence.decode: {actual} != {expected}"),
        )
        .operation("cipher.rail_fence.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "WECRLTEERDSOEEFEAOCAIVDEN"}"##)
            .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn redefence_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"rails": 3, "initial_rail": 0, "direction": "down", "order": [2, 0, 1]}"##,
    )
    .expect("static parameters");
    let p: Redefence = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEFGH";
    let actual = json!({"text":redefence(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "CGAEBDFH"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.redefence.encode: {actual} != {expected}"),
        )
        .operation("cipher.redefence.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn redefence_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"rails": 3, "initial_rail": 0, "direction": "down", "order": [2, 0, 1]}"##,
    )
    .expect("static parameters");
    let p: Redefence = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "CGAEBDFH";
    let actual = json!({"text":redefence(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.redefence.decode: {actual} != {expected}"),
        )
        .operation("cipher.redefence.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "CGAEBDFH"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn sequence_transposition_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"order": [2, 0, 3, 1]}"##).expect("static parameters");
    let p: Sequence = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "A😀β!";
    let actual = json!({"text":sequence(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "βA!😀"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(ErrorCode::InvalidInput,format!("known answer mismatch for cipher.sequence_transposition.encode: {actual} != {expected}")).operation("cipher.sequence_transposition.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "A😀β!"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn sequence_transposition_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"order": [2, 0, 3, 1]}"##).expect("static parameters");
    let p: Sequence = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "βA!😀";
    let actual = json!({"text":sequence(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "A😀β!"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(ErrorCode::InvalidInput,format!("known answer mismatch for cipher.sequence_transposition.decode: {actual} != {expected}")).operation("cipher.sequence_transposition.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "βA!😀"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn skip_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"start": "0", "step": "-9223372036854775808"}"##)
            .expect("static parameters");
    let p: Skip = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDE";
    let actual = json!({"text":skip(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ACEBD"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.skip.encode: {actual} != {expected}"),
        )
        .operation("cipher.skip.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDE"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn skip_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"start": "0", "step": "-9223372036854775808"}"##)
            .expect("static parameters");
    let p: Skip = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ACEBD";
    let actual = json!({"text":skip(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDE"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.skip.decode: {actual} != {expected}"),
        )
        .operation("cipher.skip.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ACEBD"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn columnar_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key": {"kind": "keyword", "text": "CAB"}}"##)
            .expect("static parameters");
    let p: Columnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEFGH";
    let actual = json!({"text":columnar(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "BEHCFADG"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.columnar.encode: {actual} != {expected}"),
        )
        .operation("cipher.columnar.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn columnar_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"key": {"kind": "keyword", "text": "CAB"}}"##)
            .expect("static parameters");
    let p: Columnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "BEHCFADG";
    let actual = json!({"text":columnar(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEFGH"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.columnar.decode: {actual} != {expected}"),
        )
        .operation("cipher.columnar.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "BEHCFADG"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn complete_columnar_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"key": {"kind": "keyword", "text": "CAB"}, "padding": "X", "original_length": 11}"##,
    )
    .expect("static parameters");
    let p: CompleteColumnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "FILLEDBLOCK";
    let actual = serde_json::to_value(complete_columnar(text, &p, false)?).expect("typed report");
    let expected: Value = serde_json::from_str(
        r##"{"text": "IELKLDOXFLBC", "original_length": 11, "padding_count": 1}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.complete_columnar.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.complete_columnar.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "FILLEDBLOCK"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn complete_columnar_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"key": {"kind": "keyword", "text": "CAB"}, "padding": "X", "original_length": 11}"##,
    )
    .expect("static parameters");
    let p: CompleteColumnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "IELKLDOXFLBC";
    let actual = serde_json::to_value(complete_columnar(text, &p, true)?).expect("typed report");
    let expected: Value = serde_json::from_str(
        r##"{"text": "FILLEDBLOCK", "original_length": 11, "padding_count": 1}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.complete_columnar.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.complete_columnar.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "IELKLDOXFLBC"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn double_columnar_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"first": {"kind": "keyword", "text": "CAB"}, "second": {"kind": "keyword", "text": "BA"}}"##).expect("static parameters");
    let p: DoubleColumnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEF";
    let actual = json!({"text":double_columnar(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "EFDBCA"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.double_columnar.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.double_columnar.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn double_columnar_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"first": {"kind": "keyword", "text": "CAB"}, "second": {"kind": "keyword", "text": "BA"}}"##).expect("static parameters");
    let p: DoubleColumnar = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "EFDBCA";
    let actual = json!({"text":double_columnar(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.double_columnar.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.double_columnar.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "EFDBCA"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn myszkowski_encode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"keyword": "BANANA"}"##).expect("static parameters");
    let p: Myszkowski = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEFGHIJ";
    let actual = json!({"text":myszkowski(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "BDFHJAGCEI"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.myszkowski.encode: {actual} != {expected}"),
        )
        .operation("cipher.myszkowski.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEFGHIJ"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn myszkowski_decode() -> Result<Fixture, OperationError> {
    let parameters: Value =
        serde_json::from_str(r##"{"keyword": "BANANA"}"##).expect("static parameters");
    let p: Myszkowski = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "BDFHJAGCEI";
    let actual = json!({"text":myszkowski(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "ABCDEFGHIJ"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.myszkowski.decode: {actual} != {expected}"),
        )
        .operation("cipher.myszkowski.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "BDFHJAGCEI"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn amsco_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"key": {"kind": "permutation", "order": [1, 3, 2, 0, 4]}, "initial_cell": 2, "pattern": "aca"}"##).expect("static parameters");
    let p: Amsco = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "INCOMPLETECOLUMNARWITHALTERNATINGSINGLELETTERSANDDIGRAPHS";
    let actual = json!({"text":amsco(text,&p,false)?});
    let expected: Value = serde_json::from_str(
        r##"{"text": "CECRTEGLENPHPLUTNANTEIOMOWIRSITDDSINTNALINESAALEMHATGLRGR"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.amsco.encode: {actual} != {expected}"),
        )
        .operation("cipher.amsco.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(
            r##"{"text": "INCOMPLETECOLUMNARWITHALTERNATINGSINGLELETTERSANDDIGRAPHS"}"##,
        )
        .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn amsco_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"key": {"kind": "permutation", "order": [1, 3, 2, 0, 4]}, "initial_cell": 2, "pattern": "aca"}"##).expect("static parameters");
    let p: Amsco = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "CECRTEGLENPHPLUTNANTEIOMOWIRSITDDSINTNALINESAALEMHATGLRGR";
    let actual = json!({"text":amsco(text,&p,true)?});
    let expected: Value = serde_json::from_str(
        r##"{"text": "INCOMPLETECOLUMNARWITHALTERNATINGSINGLELETTERSANDDIGRAPHS"}"##,
    )
    .expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.amsco.decode: {actual} != {expected}"),
        )
        .operation("cipher.amsco.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(
            r##"{"text": "CECRTEGLENPHPLUTNANTEIOMOWIRSITDDSINTNALINESAALEMHATGLRGR"}"##,
        )
        .expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn route_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"rows": 2, "columns": 3, "fill": {"kind": "rows", "corner": "top_left"}, "read": {"kind": "columns", "corner": "top_left"}}"##).expect("static parameters");
    let p: routes::Routes = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEF";
    let actual = json!({"text":routes::route(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ADBECF"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.route.encode: {actual} != {expected}"),
        )
        .operation("cipher.route.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn route_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"rows": 2, "columns": 3, "fill": {"kind": "rows", "corner": "top_left"}, "read": {"kind": "columns", "corner": "top_left"}}"##).expect("static parameters");
    let p: routes::Routes = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ADBECF";
    let actual = json!({"text":routes::route(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.route.decode: {actual} != {expected}"),
        )
        .operation("cipher.route.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ADBECF"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn spiral_encode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"rows": 2, "columns": 3, "fill": {"kind": "rows", "corner": "top_left"}, "read": {"kind": "clockwise", "corner": "top_left"}}"##).expect("static parameters");
    let p: routes::Routes = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCDEF";
    let actual = json!({"text":routes::spiral(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCFED"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.spiral.encode: {actual} != {expected}"),
        )
        .operation("cipher.spiral.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn spiral_decode() -> Result<Fixture, OperationError> {
    let parameters:Value=serde_json::from_str(r##"{"rows": 2, "columns": 3, "fill": {"kind": "rows", "corner": "top_left"}, "read": {"kind": "clockwise", "corner": "top_left"}}"##).expect("static parameters");
    let p: routes::Routes = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCFED";
    let actual = json!({"text":routes::spiral(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCDEF"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.spiral.decode: {actual} != {expected}"),
        )
        .operation("cipher.spiral.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCFED"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn caesar_box_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"side": 2}"##).expect("static parameters");
    let p: routes::CaesarBox = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ABCD";
    let actual = json!({"text":routes::caesar_box(text,&p,false)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ACBD"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.caesar_box.encode: {actual} != {expected}"),
        )
        .operation("cipher.caesar_box.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ABCD"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn caesar_box_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"side": 2}"##).expect("static parameters");
    let p: routes::CaesarBox = serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "ACBD";
    let actual = json!({"text":routes::caesar_box(text,&p,true)?});
    let expected: Value = serde_json::from_str(r##"{"text": "ABCD"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!("known answer mismatch for cipher.caesar_box.decode: {actual} != {expected}"),
        )
        .operation("cipher.caesar_box.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "ACBD"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn turning_grille_encode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"side": 4, "holes": [0, 7, 9, 11], "orientations": [0, 1, 2, 3]}"##,
    )
    .expect("static parameters");
    let p: grille::TurningGrille =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "THETURNINGGRILLE";
    let actual = json!({"text":grille::turning_grille(text,&p,false)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "TILUNRGHGELTENIR"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.turning_grille.encode: {actual} != {expected}"
            ),
        )
        .operation("cipher.turning_grille.encode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "THETURNINGGRILLE"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn turning_grille_decode() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"side": 4, "holes": [0, 7, 9, 11], "orientations": [0, 1, 2, 3]}"##,
    )
    .expect("static parameters");
    let p: grille::TurningGrille =
        serde_json::from_value(parameters.clone()).expect("typed fixture");
    let text = "TILUNRGHGELTENIR";
    let actual = json!({"text":grille::turning_grille(text,&p,true)?});
    let expected: Value =
        serde_json::from_str(r##"{"text": "THETURNINGGRILLE"}"##).expect("static answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            format!(
                "known answer mismatch for cipher.turning_grille.decode: {actual} != {expected}"
            ),
        )
        .operation("cipher.turning_grille.decode"));
    }
    Ok(Fixture {
        inputs: serde_json::from_str(r##"{"text": "TILUNRGHGELTENIR"}"##).expect("static input"),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn cardan_extract() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(r##"{"rows": 2, "columns": 3, "holes": [0, 4]}"##)
        .expect("parameters");
    let p: grille::Cardan = serde_json::from_value(parameters.clone()).expect("typed");
    let actual = json!({"text":grille::cardan_extract("ABCDEF",&p)?});
    let expected: Value = serde_json::from_str(r##"{"text": "AE"}"##).expect("answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            "Cardan fixture mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"text":"ABCDEF"}),
        parameters,
        outputs: actual,
    })
}
pub(crate) fn cardan_overlay() -> Result<Fixture, OperationError> {
    let parameters: Value = serde_json::from_str(
        r##"{"rows": 2, "columns": 3, "holes": [0, 4], "replacements": "xy"}"##,
    )
    .expect("parameters");
    let p: grille::Overlay = serde_json::from_value(parameters.clone()).expect("typed");
    let actual = serde_json::to_value(grille::cardan_overlay("ABCDEF", &p)?).expect("typed report");
    let expected: Value = serde_json::from_str(
        r##"{"text": "xBCDyF", "positions": [0, 4], "original_selected": "AE"}"##,
    )
    .expect("answer");
    if actual != expected {
        return Err(crate::errors::OperationError::new(
            ErrorCode::InvalidInput,
            "Cardan fixture mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"text":"ABCDEF"}),
        parameters,
        outputs: actual,
    })
}
