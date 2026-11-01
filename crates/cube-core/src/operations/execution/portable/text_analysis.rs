//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::field;
use crate::{
    errors::{ErrorCode, OperationError},
    text::{
        anagram, combinations, grid, lexical, multiset, positions, statistics, transform,
        traversal, word, word_ladder,
    },
};
use serde::Serialize;
use serde_json::{Value, json};

enum Adapter {
    Anagram,
    Combinations,
    Josephus,
    Compare,
    Subtract,
    Keyed,
    Masked,
    Words,
    Shortest,
    Validate,
    Grid,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "text.anagram.search" => Adapter::Anagram,
        "text.combinations" => Adapter::Combinations,
        "text.josephus.extract" => Adapter::Josephus,
        "text.multiset.compare" => Adapter::Compare,
        "text.multiset.subtract" => Adapter::Subtract,
        "text.skip.keyed" => Adapter::Keyed,
        "text.transform.masked" => Adapter::Masked,
        "text.word.extract" => Adapter::Words,
        "text.word_ladder.shortest" => Adapter::Shortest,
        "text.word_ladder.validate" => Adapter::Validate,
        "text.grid.render" => Adapter::Grid,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact text inputs and explicit selection fields",
    )
}
fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str, OperationError> {
    v.get(name).and_then(Value::as_str).ok_or_else(invalid)
}
fn unsigned(v: &Value) -> Result<u64, OperationError> {
    v.as_u64()
        .or_else(|| v.as_str()?.parse().ok())
        .ok_or_else(invalid)
}
fn optional_number(p: &Value, name: &str) -> Result<Option<u64>, OperationError> {
    p.get(name).map(unsigned).transpose()
}
fn optional_size(p: &Value, name: &str) -> Result<Option<usize>, OperationError> {
    optional_number(p, name)?
        .map(|v| usize::try_from(v).map_err(|_| invalid()))
        .transpose()
}
fn sizes(p: &Value, name: &str) -> Result<Option<Vec<usize>>, OperationError> {
    p.get(name)
        .map(|v| {
            v.as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|v| usize::try_from(unsigned(v)?).map_err(|_| invalid()))
                .collect()
        })
        .transpose()
}
fn profile(p: &Value) -> Result<multiset::Profile, OperationError> {
    Ok(multiset::Profile {
        normalization: statistics::Normalization::parse(string(p, "normalization")?)?,
        case: statistics::CaseMode::parse(string(p, "case")?)?,
        filter: p
            .get("filter")
            .map(|_| statistics::TextFilter::parse(string(p, "filter")?))
            .transpose()?
            .unwrap_or(statistics::TextFilter::All),
    })
}
fn output(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|v| json!({"output":v}))
        .map_err(|_| invalid())
}
pub(super) fn execute(
    op: &str,
    inputs: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let identity = crate::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| match selected {
        Adapter::Grid => {
            let grid = grid::render(
                string(&inputs, "input")?,
                field(&p, "width")?,
                field(&p, "drop_whitespace")?,
                crate::image::Traversal::parse(string(&p, "traversal")?)?,
            )?;
            let mut result = json!({"output":grid});
            if let Some(colors) = p.get("colors") {
                let colors = grid::parse_colors(colors.as_str().ok_or_else(invalid)?)?;
                result["image"] = super::images::image(&grid.image(&colors)?);
            }
            Ok(result)
        }
        Adapter::Combinations => output(
            combinations::Combinations::new(
                field(&inputs, "input")?,
                field(&p, "separator")?,
                field(&p, "max_results")?,
            )?
            .collect::<Vec<_>>(),
        ),
        Adapter::Compare | Adapter::Subtract => {
            let limits = multiset::Limits {
                max_work: field(&p, "max_work")?,
                max_memory_bytes: field(&p, "max_memory_bytes")?,
                max_report_bytes: field(&p, "max_report_bytes")?,
            };
            if matches!(selected, Adapter::Compare) {
                output(multiset::compare(
                    string(&inputs, "input")?,
                    string(&p, "requested")?,
                    profile(&p)?,
                    limits,
                )?)
            } else {
                output(multiset::subtract(
                    string(&inputs, "input")?,
                    string(&p, "requested")?,
                    profile(&p)?,
                    limits,
                )?)
            }
        }
        Adapter::Anagram => {
            let dictionary = lexical::parse_dictionary(
                string(&inputs, "dictionary")?,
                anagram::MAX_DICTIONARY_BYTES,
                anagram::MAX_ENTRIES,
            )?;
            output(anagram::search(
                string(&inputs, "input")?,
                &dictionary,
                profile(&p)?,
                anagram::Options {
                    max_words: field(&p, "max_words")?,
                    reuse: field(&p, "reuse")?,
                    max_results: field(&p, "max_results")?,
                    max_work: field(&p, "max_work")?,
                    max_memory_bytes: field(&p, "max_memory_bytes")?,
                    max_report_bytes: field(&p, "max_report_bytes")?,
                },
            )?)
        }
        Adapter::Shortest | Adapter::Validate => {
            let dictionary = lexical::parse_dictionary(
                string(&inputs, "dictionary")?,
                word_ladder::MAX_DICTIONARY_BYTES,
                anagram::MAX_ENTRIES,
            )?;
            let options = word_ladder::Options {
                max_work: field(&p, "max_work")?,
                max_memory_bytes: field(&p, "max_memory_bytes")?,
                max_report_bytes: field(&p, "max_report_bytes")?,
            };
            if matches!(selected, Adapter::Shortest) {
                output(word_ladder::shortest(
                    &dictionary,
                    string(&inputs, "start")?,
                    string(&inputs, "end")?,
                    profile(&p)?,
                    options,
                )?)
            } else {
                let path = lexical::parse_dictionary(
                    string(&inputs, "path")?,
                    word_ladder::MAX_DICTIONARY_BYTES,
                    crate::graph::MAX_VERTICES,
                )?;
                output(word_ladder::validate(
                    &dictionary,
                    &path,
                    profile(&p)?,
                    options,
                )?)
            }
        }
        Adapter::Josephus | Adapter::Keyed => {
            let steps = p["steps"]
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(unsigned)
                .collect::<Result<Vec<_>, _>>()?;
            let options = traversal::Options {
                start: optional_number(&p, "start")?,
                index_base: field(&p, "index_base")?,
                direction: field(&p, "direction")?,
                wrap: field(&p, "wrap")?,
                max_work: field(&p, "max_work")?,
                max_memory: field(&p, "max_memory")?,
                max_output: field(&p, "max_output")?,
            };
            if matches!(selected, Adapter::Keyed) {
                output(traversal::keyed(
                    string(&inputs, "input")?,
                    &steps,
                    field(&p, "count")?,
                    &options,
                )?)
            } else {
                output(traversal::josephus(
                    string(&inputs, "input")?,
                    &steps,
                    optional_size(&p, "count")?,
                    &options,
                )?)
            }
        }
        Adapter::Masked => {
            let selector = positions::Selector::from_options(
                string(&p, "selector")?,
                sizes(&p, "indices")?,
                optional_size(&p, "nth")?,
                sizes(&p, "exclude")?,
            )?;
            let overrides: Vec<String> = p
                .get("overrides")
                .map(|_| field(&p, "overrides"))
                .transpose()?
                .unwrap_or_default();
            let config = transform::Config {
                index_base: positions::IndexBase::try_from(field::<u8>(&p, "index_base")?)?,
                selector,
                selected: transform::Transformation::parse(
                    string(&p, "selected")?,
                    p.get("selected_shift")
                        .map(|_| field(&p, "selected_shift"))
                        .transpose()?,
                )?,
                otherwise: transform::Transformation::parse(
                    string(&p, "otherwise")?,
                    p.get("otherwise_shift")
                        .map(|_| field(&p, "otherwise_shift"))
                        .transpose()?,
                )?,
                overrides: overrides
                    .iter()
                    .map(|s| transform::Override::parse(s))
                    .collect::<Result<Vec<_>, _>>()?,
            };
            output(transform::masked(string(&inputs, "input")?, &config)?.final_text)
        }
        Adapter::Words => {
            let shared = p
                .get("positions")
                .map(|_| word::parse_positions(string(&p, "positions")?))
                .transpose()?;
            let per_word = p
                .get("per_word")
                .map(|_| word::parse_per_word(string(&p, "per_word")?))
                .transpose()?;
            let positions = match (shared.as_deref(), per_word.as_deref()) {
                (Some(p), None) => word::Positions::Shared(p),
                (None, Some(p)) => word::Positions::PerWord(p),
                _ => return Err(invalid()),
            };
            output(word::extract(
                string(&inputs, "input")?,
                positions,
                field(&p, "index_base")?,
                string(&p, "short_word")?,
            )?)
        }
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
