//! Explicit transposition configurations only; no factorial key enumeration.
use super::{candidates::*, *};
use crate::ciphers::transposition as t;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Configuration {
    RailFence {
        #[serde(deserialize_with = "wire::integer")]
        rails: usize,
        #[serde(deserialize_with = "wire::integer")]
        initial_rail: usize,
        direction: t::Direction,
        #[serde(deserialize_with = "wire::indices")]
        readout: Vec<usize>,
    },
    Columnar {
        #[serde(deserialize_with = "wire::indices")]
        order: Vec<usize>,
    },
    Route {
        #[serde(deserialize_with = "wire::integer")]
        rows: usize,
        #[serde(deserialize_with = "wire::integer")]
        columns: usize,
        fill: t::routes::Route,
        read: t::routes::Route,
    },
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub text: String,
    pub model: scoring::Model,
    pub configurations: Vec<Configuration>,
    #[serde(default)]
    pub limits: Limits,
}
pub(crate) fn encode(text: &str, configuration: &Configuration) -> Result<String, OperationError> {
    text_bound(text, 65536)?;
    let chars: Vec<char> = text.chars().collect();
    Ok(t::apply(
        &chars,
        &permutation(chars.len(), configuration)?,
        false,
    ))
}
fn permutation(n: usize, c: &Configuration) -> Result<Vec<usize>, OperationError> {
    let op = "analysis.transposition.rank";
    match c {
        Configuration::RailFence {
            rails,
            initial_rail,
            direction,
            readout,
        } => t::rail_order(
            n,
            &t::Rails {
                rails: *rails,
                initial_rail: *initial_rail,
                direction: *direction,
            },
            readout,
            op,
        ),
        Configuration::Columnar { order } => {
            bound(order.len(), 4096)?;
            if order.is_empty() {
                return Err(invalid("empty column permutation"));
            }
            t::validate_order(order, order.len(), op)?;
            Ok(t::column_order(n, order))
        }
        Configuration::Route {
            rows,
            columns,
            fill,
            read,
        } => t::routes::order(
            n,
            &t::routes::Routes {
                rows: *rows,
                columns: *columns,
                fill: *fill,
                read: *read,
            },
            op,
        ),
    }
}
/// Reuses Core permutation construction/application. Source positions index exact
/// input scalars for each output scalar; every configuration alias is retained.
pub fn rank(
    r: &Request,
    cancel: Option<&AtomicBool>,
) -> Result<candidates::Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    text_bound(&r.text, 65536)?;
    bound(r.configurations.len(), 1024)?;
    let model = scoring::Prepared::new(&r.model)?;
    let mut b = Budget::new(&r.limits, cancel)?;
    let mut out = Retained::new(r.limits.max_results);
    let mut attempted = 0;
    let chars: Vec<char> = r.text.chars().collect();
    for (i, configuration) in r.configurations.iter().enumerate() {
        let config_work = match configuration {
            Configuration::RailFence { readout, .. } => readout.len(),
            Configuration::Columnar { order } => order.len(),
            Configuration::Route { .. } => 1,
        };
        if !b.take((chars.len() * 12 + config_work).max(1)) {
            break;
        }
        attempted += 1;
        let Ok(order) = permutation(chars.len(), configuration) else {
            out.reject(i);
            continue;
        };
        let plain = t::apply(&chars, &order, true);
        let mut positions = vec![0; chars.len()];
        for (source, &target) in order.iter().enumerate() {
            positions[target] = source;
        }
        let Some(score) = model.score(&plain, i, &mut b) else {
            break;
        };
        if !out.accept(
            plain,
            score,
            Alias {
                input_index: i,
                key: Key::Transposition {
                    configuration: configuration.clone(),
                },
                source_positions: positions,
            },
            &mut b,
        ) {
            break;
        }
    }
    out.report(model, b, attempted)
}
