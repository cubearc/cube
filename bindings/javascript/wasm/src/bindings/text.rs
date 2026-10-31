//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use cube_core::{
    errors::{ErrorCode, OperationError},
    text::{self, classes, grid, statistics as stats, unicode, values},
};
use serde::Serialize;
use serde_json::{Value, json};

enum Adapter {
    Reverse,
    Normalize,
    Extract,
    Statistics,
    Factors,
    Lengths,
    Codepoints,
    Values,
    Inspect,
    Compare,
    Classes,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "text.reverse" => Adapter::Reverse,
        "text.normalize" => Adapter::Normalize,
        "text.extract" => Adapter::Extract,
        "text.statistics" => Adapter::Statistics,
        "text.grid.factors" => Adapter::Factors,
        "text.word.lengths" => Adapter::Lengths,
        "text.codepoints" => Adapter::Codepoints,
        "text.word.values" => Adapter::Values,
        "text.unicode.inspect" => Adapter::Inspect,
        "text.unicode.compare" => Adapter::Compare,
        "text.class.map" => Adapter::Classes,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact typed text input and parameter fields",
    )
}
fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, OperationError> {
    value.get(name).and_then(Value::as_str).ok_or_else(invalid)
}
fn unsigned(value: &Value) -> Result<u64, OperationError> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .ok_or_else(invalid)
}
fn number(value: &Value, name: &str) -> Result<u64, OperationError> {
    unsigned(value.get(name).ok_or_else(invalid)?)
}
fn size(value: &Value, name: &str) -> Result<usize, OperationError> {
    number(value, name)?.try_into().map_err(|_| invalid())
}
fn optional_size(value: &Value, name: &str) -> Result<Option<usize>, OperationError> {
    value.get(name).map(|_| size(value, name)).transpose()
}
fn flag(value: &Value, name: &str) -> Result<bool, OperationError> {
    value.get(name).and_then(Value::as_bool).ok_or_else(invalid)
}
fn output(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|v| json!({"output":v}))
        .map_err(|_| invalid())
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let source = string(&inputs, "input")?;
    let selected = adapter(op).ok_or_else(invalid)?;
    match selected {
        Adapter::Reverse => output(text::reverse(source)?),
        Adapter::Normalize => output(text::normalize(source, string(&p, "form")?)?),
        Adapter::Extract => {
            let indices = p
                .get("indices")
                .map(|value| {
                    value
                        .as_array()
                        .ok_or_else(invalid)?
                        .iter()
                        .map(|v| unsigned(v)?.try_into().map_err(|_| invalid()))
                        .collect::<Result<Vec<usize>, OperationError>>()
                })
                .transpose()?;
            output(text::extract(
                source,
                indices.as_deref(),
                optional_size(&p, "start")?,
                optional_size(&p, "stride")?,
                optional_size(&p, "stop")?,
            )?)
        }
        Adapter::Statistics => output(stats::analyze(
            source,
            stats::Options {
                ngram_size: size(&p, "ngram_size")?,
                normalization: stats::Normalization::parse(string(&p, "normalization")?)?,
                case: stats::CaseMode::parse(string(&p, "case")?)?,
                filter: stats::TextFilter::parse(string(&p, "filter")?)?,
                max_results: size(&p, "max_results")?,
            },
        )?),
        Adapter::Factors => output(grid::factors(
            source,
            flag(&p, "drop_whitespace")?,
            size(&p, "max_candidates")?,
        )?),
        Adapter::Lengths => output(values::word_lengths(source)?),
        Adapter::Codepoints => output(values::codepoints(source, string(&p, "format")?)?),
        Adapter::Values => {
            let table = p
                .get("table")
                .map(|_| values::parse_table(string(&p, "table")?))
                .transpose()?;
            output(values::word_values(
                source,
                table.as_deref(),
                string(&p, "unmapped")?,
                flag(&p, "sums")?,
                number(&p, "max_sum")?,
            )?)
        }
        Adapter::Inspect | Adapter::Compare => {
            let options = unicode::Options {
                max_results: size(&p, "max_results")?,
                max_work: number(&p, "max_work")?,
                max_output: size(&p, "max_output")?,
            };
            if matches!(selected, Adapter::Inspect) {
                output(unicode::inspect(source, &options)?)
            } else {
                output(unicode::compare(
                    source,
                    string(&inputs, "other")?,
                    string(&p, "normalization")?,
                    &options,
                )?)
            }
        }
        Adapter::Classes => {
            let definition = p
                .get("definition")
                .map(|_| string(&p, "definition"))
                .transpose()?;
            let y = p.get("y").map(|_| string(&p, "y")).transpose()?;
            if let Some(definition) = definition {
                if y.is_some() {
                    return Err(invalid());
                }
                output(
                    classes::Definition::parse(definition)?.map(source, string(&p, "unmapped")?)?,
                )
            } else {
                output(classes::map(source, y, string(&p, "unmapped")?)?)
            }
        }
    }
}
