use super::*;
use crate::operations::contracts::Fixture;
use serde_json::json;
fn fixture(decode: bool) -> Result<Fixture, OperationError> {
    let p:Parameters=serde_json::from_value(json!({"profile":"arnold_paged_words_v1","source_id":"synthetic:two-pages-v1","index_base":1,"choice":"round_robin"})).expect("parameters");
    let pages = vec!["THE ROAD\nTHE CUBE".into(), "CUBE 🐈\r\nROAD".into()];
    let words = vec![
        "THE".into(),
        "THE".into(),
        "CUBE".into(),
        "CUBE".into(),
        "🐈".into(),
    ];
    let coordinates:Vec<Coordinate>=serde_json::from_value(json!([{"page":1,"line":1,"word":1},{"page":1,"line":2,"word":1},{"page":1,"line":2,"word":2},{"page":2,"line":1,"word":1},{"page":2,"line":1,"word":2}])).expect("independent coordinates");
    let result = if decode {
        super::decode(&pages, &coordinates, &p)?
    } else {
        super::encode(&pages, &words, &p)?
    };
    if result.words != words || result.coordinates != coordinates {
        return Err(invalid("fixture"));
    }
    Ok(Fixture {
        inputs: if decode {
            json!({"pages":pages,"coordinates":coordinates})
        } else {
            json!({"pages":pages,"words":words})
        },
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
