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
pub(crate) fn period_estimate() -> Result<Fixture, OperationError> {
    let request = json!({"text":"ABABAB","periods":[2,1],"ngram":2,"expected_ic":0.5});
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
