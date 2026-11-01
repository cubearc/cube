//! ACA Conjugated Matrix Bifid: two explicit independent I/J squares.
use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CmBifid {
    #[schemars(length(equal = 25))]
    pub input_square: String,
    #[schemars(length(equal = 25))]
    pub output_square: String,
    #[schemars(range(min = 1, max = 4096))]
    #[serde(deserialize_with = "wire::integer")]
    pub period: usize,
}
/// Decode inverts the coordinate stream, then reads the first square. It is not
/// encryption with swapped squares. J aliases to I in the same way as Bifid.
pub fn cm_bifid(text: &str, p: &CmBifid, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.cm_bifid.decode"
    } else {
        "cipher.cm_bifid.encode"
    };
    period(p.period, op)?;
    let first = square(&p.input_square, op)?;
    let second = square(&p.output_square, op)?;
    let (from, to) = if decode {
        (&second, &first)
    } else {
        (&first, &second)
    };
    bifid_squares(text, p.period, from, to, decode, op)
}
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let parameters = json!({"input_square":"EXTRAKLMPOHWZQDGVUSIFCBYN","output_square":"NCDRSOBFQUVAGPWEYHMXLTIKZ","period":7});
    let (input, expected) = if decode {
        ("FANXZEXFENUKKRBYNKAK", "ODDPERIODSAREPOPULAR")
    } else {
        ("ODDPERIODSAREPOPULAR", "FANXZEXFENUKKRBYNKAK")
    };
    let text = cm_bifid(
        input,
        &serde_json::from_value(parameters.clone()).expect("typed fixture"),
        decode,
    )?;
    if text != expected {
        return Err(invalid(
            "cipher.cm_bifid.encode",
            "fixture",
            "published CM Bifid answer mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"text":input}),
        parameters,
        outputs: json!({"text":text}),
    })
}
pub(crate) fn fixture_encode() -> Result<Fixture, OperationError> {
    fixture(false)
}
pub(crate) fn fixture_decode() -> Result<Fixture, OperationError> {
    fixture(true)
}
