//! Fixed synthetic known answers; every fixture calls the public typed operation.
use super::*;
use crate::operations::contracts::Fixture;
use serde_json::{Value, json};
fn fixture(request: Value, report: impl Serialize) -> Fixture {
    Fixture {
        inputs: json!({"request":request}),
        parameters: json!({}),
        outputs: json!({"report":report}),
    }
}
fn dictionary(word: &str) -> Value {
    json!({"kind":"dictionary","entries":[{"text":word,"weight":2.0}],"unseen_weight":-1.0})
}
pub(crate) fn caesar_rank() -> Result<Fixture, OperationError> {
    let request = json!({"text":"B","shifts":[0,1,1],"model":dictionary("A")});
    let r = classical::caesar(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.candidates[0].plaintext != "A" || r.candidates[0].aliases.len() != 2 || r.attempted != 3 {
        return Err(invalid("Caesar known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn affine_rank() -> Result<Fixture, OperationError> {
    let request = json!({"text":"IHHWVCSWFRCP","keys":[{"a":2,"b":0},{"a":5,"b":8}],"model":dictionary("AFFINECIPHER")});
    let r = classical::affine(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.candidates[0].plaintext != "AFFINECIPHER" || r.rejected.len() != 1 {
        return Err(invalid("Affine known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn vigenere_rank() -> Result<Fixture, OperationError> {
    let request = json!({"text":"BCBC","lengths":[2],"beam_width":1,"model":{"kind":"counts","n":1,"entries":[{"text":"A","weight":26.0}],"smoothing":1.0,"normalization":"ascii_letters"}});
    let r = vigenere::rank(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.ranking.candidates[0].plaintext != "AAAA" || r.ranking.status.stop != Stop::Pruned {
        return Err(invalid("Vigenere known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn transposition_rank() -> Result<Fixture, OperationError> {
    let request = json!({"text":"ACEBDF","configurations":[{"kind":"columnar","order":[0,1]},{"kind":"columnar","order":[0,1]}],"model":dictionary("ABCDEF")});
    let r = transposition::rank(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.candidates[0].plaintext != "ABCDEF"
        || r.candidates[0].aliases[0].source_positions != [0, 3, 1, 4, 2, 5]
        || r.candidates[0].aliases.len() != 2
    {
        return Err(invalid("transposition known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn homophonic_inspect() -> Result<Fixture, OperationError> {
    let request =
        json!({"tokens":["12","雪","12","éé"],"groups":[{"symbol":"A","tokens":["12","雪"]}]});
    let r = homophonic::inspect(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.observed[0].count != 2
        || r.hypotheses[0].count != 3
        || r.unmapped != 1
        || r.adjacency.len() != 3
    {
        return Err(invalid("token evidence known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn crib_verify() -> Result<Fixture, OperationError> {
    let request = json!({"candidates":[{"data":{"kind":"text","text":"é Aa!"},"key":{"kind":"caesar","shift":1}}],"ciphertext":{"kind":"text","text":"é Bb!"},"crib":{"kind":"text","text":"aa"},"offset":0,"view":"ascii_letters"});
    let r = crib::verify(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.candidates[0].all_match != Some(true)
        || r.candidates[0].reencode_matches != Some(true)
        || r.candidates[0].positions[0].original_byte_offset != Some(3)
    {
        return Err(invalid("crib known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn period_estimate() -> Result<Fixture, OperationError> {
    let request = json!({"text":"ABABAB","periods":["2","1"],"ngram":"2","expected_ic":0.5});
    let r = period::estimate(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.ic != Some(0.4) || r.periods[0].divisor_hits != 4 || r.repeats[0].positions != [0, 2, 4] {
        return Err(invalid("period known answer mismatch"));
    }
    Ok(fixture(request, r))
}
pub(crate) fn language_score() -> Result<Fixture, OperationError> {
    let request = json!({"candidates":["B","A",""],"model":{"kind":"dictionary","entries":[{"text":"A","weight":2.0}],"unseen_weight":-1.0}});
    let r = scoring::score(
        &serde_json::from_value(request.clone()).expect("typed fixture"),
        None,
    )?;
    if r.ranking != [1, 0, 2]
        || r.scores[0].total_nanos != Some(-1_000_000_000)
        || r.scores[1].total_nanos != Some(2_000_000_000)
        || r.scores[2].total_nanos.is_some()
    {
        return Err(invalid("language scoring known answer mismatch"));
    }
    Ok(fixture(request, r))
}
