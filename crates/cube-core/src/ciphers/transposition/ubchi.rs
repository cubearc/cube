//! LANAKI's 1997 Ubchi profile with exact caller-owned between-pass nulls.
use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ubchi {
    #[schemars(length(min = 1, max = 8191), regex(pattern = r"^[A-Z]+( [A-Z]+)*$"))]
    pub key_phrase: String,
    #[schemars(length(min = 1, max = 4096), extend("x-cube-max-utf8-bytes" = 16384))]
    pub nulls: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UbchiReport {
    #[schemars(length(max = 65536), extend("x-cube-max-utf8-bytes" = 262144))]
    pub text: String,
    #[schemars(range(min = 0, max = 65535))]
    pub original_length: usize,
    #[schemars(range(min = 1, max = 4096))]
    pub null_count: usize,
}

/// Use the same stable phrase key twice. Spaces delimit actual key words only;
/// no plaintext normalization, guessed null count or arbitrary unpadding occurs.
pub fn transform(text: &str, p: &Ubchi, decode: bool) -> Result<UbchiReport, OperationError> {
    let op = if decode {
        "cipher.ubchi.decode"
    } else {
        "cipher.ubchi.encode"
    };
    bound(op, "key_phrase", p.key_phrase.len(), 8191)?;
    bound(op, "nulls", p.nulls.len(), 16384)?;
    let words: Vec<_> = p.key_phrase.split(' ').collect();
    if words
        .iter()
        .any(|w| w.is_empty() || !w.bytes().all(|c| c.is_ascii_uppercase()))
    {
        return Err(invalid(
            op,
            "key_phrase",
            "use nonempty uppercase A..Z words separated by one ASCII space",
        ));
    }
    let letters = words.concat();
    bound(op, "key_phrase", letters.len(), MAX_KEY)?;
    let null_count = p.nulls.chars().count();
    if null_count != words.len() {
        return Err(invalid(
            op,
            "nulls",
            "provide exactly one null scalar per key phrase word",
        ));
    }
    let length = scalars(text, op)?.len();
    let key = Columnar {
        key: ColumnKey::Keyword { text: letters },
    };
    let pass = |text: &str| columnar(text, &key, decode).map_err(|e| e.operation(op));
    let (result, original_length) = if decode {
        let mut between = pass(text)?;
        if !between.ends_with(&p.nulls) {
            return Err(invalid(
                op,
                "nulls",
                "declared between-pass null suffix does not match",
            ));
        }
        between.truncate(between.len() - p.nulls.len());
        (pass(&between)?, length - null_count)
    } else {
        bound(op, "cells", length + null_count, MAX_SYMBOLS)?;
        let mut between = pass(text)?;
        between.push_str(&p.nulls);
        (pass(&between)?, length)
    };
    Ok(UbchiReport {
        text: result,
        original_length,
        null_count,
    })
}

fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let parameters = json!({"key_phrase":"HERRSCHAFT","nulls":"Z"});
    let (input, expected) = if decode {
        (
            "RARHZIADAREIOSAUFRTMLTVTECNMTXSSTOEERSXAYVNCIPAF",
            "FIRSTARMYXPLANFIVEACTIVATEDXCROSSMARNEATSETHOUR",
        )
    } else {
        (
            "FIRSTARMYXPLANFIVEACTIVATEDXCROSSMARNEATSETHOUR",
            "RARHZIADAREIOSAUFRTMLTVTECNMTXSSTOEERSXAYVNCIPAF",
        )
    };
    let report = transform(
        input,
        &serde_json::from_value(parameters.clone()).expect("typed source"),
        decode,
    )?;
    let outputs = serde_json::to_value(report).expect("typed report");
    if outputs != json!({"text":expected,"original_length":47,"null_count":1}) {
        return Err(invalid(
            "cipher.ubchi.encode",
            "fixture",
            "independent source answer mismatch",
        ));
    }
    Ok(Fixture {
        inputs: json!({"text":input}),
        parameters,
        outputs,
    })
}
pub(crate) fn fixture_encode() -> Result<Fixture, OperationError> {
    fixture(false)
}
pub(crate) fn fixture_decode() -> Result<Fixture, OperationError> {
    fixture(true)
}
