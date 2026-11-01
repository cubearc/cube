use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;
pub(crate) fn parameters() -> Parameters {
    serde_json::from_value(json!({"profile":"aca_base","hat":"APOTHECARY","key":"CHEMIST","setting":"DRUGS","punctuation":"aca_printed"})).expect("parameters")
}
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let p = parameters();
    let plain = [
        "*Bush Signs Intelligence Overhaul Legislation",
        "*Bin *Laden Urges Fighters to Strike Oil Facilities",
        "*Pfizer: Painkiller may pose increased cardiovascular risk",
        "*Carrey masters disguises in '*Lemony *Snicket'",
        "*Martinez blasts ex-teammate *Schilling",
    ];
    let cipher = [
        "*GCTJ TNWOT NOALZZNWLODL PHLXJFCZ ZLWNTZFANPO",
        "*VZS *IUXYS FDHYO CZHWPYDO PT OPDZMY TZI CURZIZPZYO",
        "*OAYLWF: OTYIDYEEWF XTJ ONZW YIUFWTZWC UTFCYNQTZUPETF FYZD",
        "*GQHHPA YQDKPHD WFDNBFDPD FR '*UPYXRA *DRFGEPK'",
        "*UIOYGWQH CVIJYJ QP=YQIUUIYQ *JSXGVVGWD",
    ];
    let input: Vec<String> = if decode {
        cipher.iter().map(|s| s.to_string()).collect()
    } else {
        plain.iter().map(|s| s.to_string()).collect()
    };
    let result = transform(&input, &p, decode)?;
    let expected: Vec<String> = if decode {
        plain.iter().map(|s| s.to_ascii_uppercase()).collect()
    } else {
        cipher.iter().map(|s| s.to_string()).collect()
    };
    if result.headlines.0 != expected || result.alphabet != "CFUAPTOSNZILYEJWHGVBQMKXDR" {
        return Err(invalid("fixture"));
    }
    Ok(Fixture {
        inputs: json!({"headlines":input}),
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
