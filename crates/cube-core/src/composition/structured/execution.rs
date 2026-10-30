use crate::operations::execution::{self as e, Entry, Failure, Representation::*};
use serde_json::json;
fn failure(error: crate::errors::OperationError) -> Failure {
    use crate::errors::ErrorCode;
    Failure(match error.code {
        ErrorCode::ResourceLimit
            if matches!(
                error.operation,
                Some("number.add" | "number.maximum" | "text.integer")
            ) =>
        {
            "numeric_overflow"
        }
        ErrorCode::ResourceLimit => "input_limit",
        ErrorCode::InvalidParameter => "invalid_pointer",
        _ => match error.operation {
            Some("json.select") => "missing_field",
            Some("json.parse") => "invalid_json",
            Some("json.text") => "invalid_type",
            _ => "invalid_number",
        },
    })
}
pub(crate) const ENTRIES: &[Entry] = &[
    Entry::pure("json.parse", &[Text], &[Json], |i, _, _| {
        Ok(json!({"output":super::parse_json(e::text(i,"input")?).map_err(failure)?}))
    }),
    Entry::pure("json.select", &[Json], &[Json], |i, p, _| {
        Ok(
            json!({"output":super::select_field(&i["input"],e::text(p,"pointer")?).map_err(failure)?}),
        )
    }),
    Entry::pure("json.text", &[Json], &[Text], |i, _, _| {
        Ok(json!({"output":super::json_text(&i["input"]).map_err(failure)?}))
    }),
    Entry::pure("text.integer", &[Text], &[Json], |i, _, _| {
        Ok(json!({"output":super::parse_integer(e::text(i,"input")?).map_err(failure)?}))
    }),
    Entry::pure("number.maximum", &[Json], &[Json], |i, _, _| {
        let v: Vec<i64> =
            serde_json::from_value(i["input"].clone()).map_err(|_| Failure("invalid_number"))?;
        Ok(json!({"output":super::maximum(&v).map_err(failure)?}))
    }),
    Entry::pure("number.add", &[Json, Json], &[Json], |i, _, _| {
        Ok(
            json!({"output":super::add(i["left"].as_i64().ok_or(Failure("invalid_number"))?,i["right"].as_i64().ok_or(Failure("invalid_number"))?).map_err(failure)?}),
        )
    }),
];
