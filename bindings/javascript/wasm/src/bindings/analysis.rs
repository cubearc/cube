//! Explicit analysis requests; scoring and search remain bounded Core algorithms.
use cube_core::{
    cryptanalysis as a,
    errors::{ErrorCode, OperationError},
};
use serde::Serialize;
use serde_json::{Value, json};
type Handler = fn(&str) -> Result<Value, OperationError>;
fn report(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|report| json!({"report":report}))
        .map_err(|_| {
            OperationError::new(
                ErrorCode::InvalidInput,
                "typed analysis result serialization failed",
            )
        })
}
macro_rules! handler {
    ($name:ident,$f:path) => {
        fn $name(doc: &str) -> Result<Value, OperationError> {
            report($f(&a::parse_request(doc)?, None)?)
        }
    };
}
handler!(score, a::scoring::score);
handler!(period, a::period::estimate);
handler!(caesar, a::classical::caesar);
handler!(affine, a::classical::affine);
handler!(vigenere, a::vigenere::rank);
handler!(transposition, a::transposition::rank);
handler!(homophonic, a::homophonic::inspect);
handler!(crib, a::crib::verify);
handler!(frequency, a::frequency::compare);
fn resolve(op: &str) -> Option<Handler> {
    Some(match op {
        "analysis.language.score" => score,
        "analysis.period.estimate" => period,
        "analysis.caesar.rank" => caesar,
        "analysis.affine.rank" => affine,
        "analysis.vigenere.rank" => vigenere,
        "analysis.transposition.rank" => transposition,
        "analysis.homophonic.inspect" => homophonic,
        "analysis.crib.verify" => crib,
        "analysis.frequency.compare" => frequency,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    resolve(op).is_some()
}
pub(super) fn execute(op: &str, i: Value, _: Value) -> Result<Value, OperationError> {
    resolve(op).ok_or_else(|| {
        OperationError::new(ErrorCode::InvalidInput, "no installed analysis binding")
    })?(&i["request"].to_string())
}
