//! The five independent known answers printed by the ACA Headlines sheet.
use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn published_headlines_construct_the_exact_alphabet_and_all_five_rows() {
    let p = json!({"profile":"aca_base","hat":"APOTHECARY","key":"CHEMIST","setting":"DRUGS","punctuation":"aca_printed"});
    let plain = json!([
        "*Bush Signs Intelligence Overhaul Legislation",
        "*Bin *Laden Urges Fighters to Strike Oil Facilities",
        "*Pfizer: Painkiller may pose increased cardiovascular risk",
        "*Carrey masters disguises in '*Lemony *Snicket'",
        "*Martinez blasts ex-teammate *Schilling"
    ]);
    let expected = json!([
        "*GCTJ TNWOT NOALZZNWLODL PHLXJFCZ ZLWNTZFANPO",
        "*VZS *IUXYS FDHYO CZHWPYDO PT OPDZMY TZI CURZIZPZYO",
        "*OAYLWF: OTYIDYEEWF XTJ ONZW YIUFWTZWC UTFCYNQTZUPETF FYZD",
        "*GQHHPA YQDKPHD WFDNBFDPD FR '*UPYXRA *DRFGEPK'",
        "*UIOYGWQH CVIJYJ QP=YQIUUIYQ *JSXGVVGWD"
    ]);
    let result = execution::execute(
        "cipher.headlines.encode",
        1,
        &json!({"headlines":plain}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result["alphabet"], "CFUAPTOSNZILYEJWHGVBQMKXDR");
    assert_eq!(result["headlines"], expected);
    let result = execution::execute(
        "cipher.headlines.decode",
        1,
        &json!({"headlines":expected}),
        &p,
        Budget::default(),
    )
    .unwrap();
    let canonical = plain
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_ascii_uppercase())
        .collect::<Vec<_>>();
    assert_eq!(result["headlines"], json!(canonical));
}

fn parameters() -> serde_json::Value {
    json!({"profile":"aca_base","hat":"APOTHECARY","key":"CHEMIST","setting":"DRUGS","punctuation":"aca_printed"})
}
fn call(input: serde_json::Value, p: serde_json::Value) -> execution::Result<serde_json::Value> {
    execution::execute("cipher.headlines.encode", 1, &input, &p, Budget::default())
}
#[test]
fn empty_headlines_and_punctuation_are_explicit() {
    let result = call(json!({"headlines":["","","","",""]}), parameters()).unwrap();
    assert_eq!(result["headlines"], json!(["", "", "", "", ""]));
    let mut p = parameters();
    p["punctuation"] = json!("preserve");
    assert_eq!(
        call(json!({"headlines":["- = * 123","","","",""]}), p).unwrap()["headlines"][0],
        "- = * 123"
    );
    for text in ["literal=", "é", "🐈", "\n", "\0"] {
        assert!(call(json!({"headlines":[text,"","","",""]}), parameters()).is_err());
    }
}
#[test]
fn profile_key_and_identity_variants_reject_instead_of_guessing() {
    for patch in [
        json!({"profile":"up"}),
        json!({"hat":""}),
        json!({"hat":"A".repeat(27)}),
        json!({"key":"Chemist"}),
        json!({"setting":"DRUG"}),
        json!({"setting":"DRUGSS"}),
        json!({"setting":"CRUGS"}),
    ] {
        let mut p = parameters();
        p.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert!(
            call(json!({"headlines":["","","","",""]}), p).is_err(),
            "{patch}"
        );
    }
    for input in [
        json!({"headlines":[]}),
        json!({"headlines":["a","b","c","d"]}),
        json!({"headlines":["","","","",""],"extra":true}),
    ] {
        assert!(call(input, parameters()).is_err());
    }
}
#[test]
fn aggregate_scalar_budget_is_checked_at_the_exact_boundary() {
    assert!(
        call(
            json!({"headlines":["A".repeat(65536),"","","",""]}),
            parameters()
        )
        .is_ok()
    );
    assert!(
        call(
            json!({"headlines":["A".repeat(65536),"A","","",""]}),
            parameters()
        )
        .is_err()
    );
}
