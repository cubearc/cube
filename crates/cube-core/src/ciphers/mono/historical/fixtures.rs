use super::*;
use crate::operations::contracts::Fixture;
use serde_json::{Value, json};
fn fixture(
    op: &str,
    inputs: Value,
    parameters: Value,
    expected: Value,
) -> Result<Fixture, OperationError> {
    let outputs = execute(op, &inputs, &parameters)?;
    if outputs != expected {
        return Err(invalid("fixture"));
    }
    Ok(Fixture {
        inputs,
        parameters,
        outputs,
    })
}
pub(crate) fn malespin() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.malespin",
        json!({"text":"bueno Amigo á"}),
        json!({}),
        json!({"text":"tuani Epofi á"}),
    )
}
pub(crate) fn vatsyayana() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.vatsyayana",
        json!({"text":"QOEU"}),
        json!({"pairs":[["Q","W"],["O","I"],["E","S"],["U","H"]]}),
        json!({"text":"WISH"}),
    )
}
fn parameters() -> Value {
    json!({"definition":{"entries":[{"alternatives":["OR"],"codes":["10"]},{"alternatives":["D"],"codes":["05"]}],"null_codes":["99"],"cancel_codes":["27"]}})
}
fn report(units: Value, nulls: Value, canceled: Value, cancels: Value) -> Value {
    json!({"report":{"units":units,"null_indices":nulls,"canceled_indices":canceled,"cancel_indices":cancels}})
}
pub(crate) fn nomenclator_encode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.nomenclator.encode",
        json!({"units":["OR","D"]}),
        parameters(),
        json!({"tokens":["10","05"]}),
    )
}
pub(crate) fn nomenclator_decode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.nomenclator.decode",
        json!({"tokens":["10","05","27","99"]}),
        parameters(),
        report(
            json!([{"token_index":0,"alternatives":["OR"]}]),
            json!([3]),
            json!([1]),
            json!([2]),
        ),
    )
}
pub(crate) fn syllabary_parameters() -> Value {
    let rows = [
        "C 3 H 8 AR M ING P RI N",
        "CE A 1 AL AN AND ARE AS AT ATE",
        "ATI B 2 BE CA CO COM D 4 DA",
        "DE E 5 EA ED EN ENT ER ERE ERS",
        "ES EST F 6 G 7 HAS HE I 9",
        "IN ION IS IT IVE J 0 K L LA",
        "LE ME ND NE NT O OF ON OR OU",
        "Q R RA RE RED RES RO S SE SH",
        "ST STO T TE TED TER TH THE THI THR",
        "TI TO U V VE W WE X Y Z",
    ];
    let cells: Vec<_> = rows.iter().flat_map(|r| r.split_whitespace()).collect();
    json!({"definition":{"cells":cells,"row_labels":"8502341679","column_labels":"6719432508"}})
}
pub(crate) fn syllabary_encode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.syllabary.encode",
        json!({"units":["OR","DE","R","S","RE","CE","IVE","D"]}),
        syllabary_parameters(),
        json!({"tokens":["10","26","67","65","69","56","44","05"]}),
    )
}
pub(crate) fn syllabary_decode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.syllabary.decode",
        json!({"tokens":["10","26"]}),
        syllabary_parameters(),
        report(
            json!([{"token_index":0,"alternatives":["OR"]},{"token_index":1,"alternatives":["DE"]}]),
            json!([]),
            json!([]),
            json!([]),
        ),
    )
}
pub(crate) fn great_cipher_encode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.great_cipher.encode",
        json!({"units":["Les","En","Ne","Mi","S"]}),
        json!({}),
        json!({"tokens":["124","22","125","46","120"]}),
    )
}
pub(crate) fn great_cipher_decode() -> Result<Fixture, OperationError> {
    fixture(
        "cipher.great_cipher.decode",
        json!({"tokens":["124","22","125","46","345","27","120","14","55","502"]}),
        json!({}),
        report(
            json!([{"token_index":0,"alternatives":["Les"]},{"token_index":1,"alternatives":["En"]},{"token_index":2,"alternatives":["Ne"]},{"token_index":3,"alternatives":["Mi"]},{"token_index":6,"alternatives":["S"]},{"token_index":8,"alternatives":["I","J"]},{"token_index":9,"alternatives":["Bien"]}]),
            json!([7]),
            json!([4]),
            json!([5]),
        ),
    )
}
