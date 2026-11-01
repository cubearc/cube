use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;
pub(super) fn parameters() -> VicEnglish2010 {
    VicEnglish2010 {
        phrase: "ALLTHEPEOPLEAREDEADB".into(),
        mnemonic: "ASINTOER".into(),
        date_digits: "391752".into(),
        personal_number: 15,
        message_indicator: "60115".into(),
        split_at: 14,
        padding: String::new(),
    }
}
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let p = parameters();
    let plain = "ASSIGNEDOBJECTIVESINVALIDATED.REPORTIMMEDIATELYTOSAFEHOUSE.AWAITEXTRACTIONINSTRUCTIONSWITHINWEEK";
    let cipher = "439836529332548692543593224039193759123412656674021687356133550753351136926066081424184749148634617711450653261143330085374956011529149";
    let (input, expected) = if decode {
        (cipher, plain)
    } else {
        (plain, cipher)
    };
    let actual = transform(input, &p, decode)?;
    if actual != expected {
        return Err(invalid(
            "cipher.vic.encode",
            "fixture",
            "published complete VIC answer mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"text":input}),
        parameters: serde_json::to_value(p).expect("typed source fixture"),
        outputs: json!({"text":actual}),
    })
}
pub(crate) fn fixture_encode() -> Result<Fixture, OperationError> {
    fixture(false)
}
pub(crate) fn fixture_decode() -> Result<Fixture, OperationError> {
    fixture(true)
}
