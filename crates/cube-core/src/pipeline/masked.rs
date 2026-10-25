//! One concrete masked-text adapter and its conservative charges, not rule dispatch.
use super::{Error, Parameter, Step, invalid};
use crate::{
    errors::ErrorCode,
    text::{
        self,
        positions::{IndexBase, Selector},
        transform::{Config, Override, Report, Transformation},
    },
};

pub(super) fn prepare(step: &Step) -> Result<Config, Error> {
    let number = |name: &str| -> Result<Option<usize>, Error> {
        match step.parameters.get(name) {
            Some(Parameter::Integer(n)) => usize::try_from(*n)
                .map(Some)
                .map_err(|_| invalid("integer exceeds position bounds").parameter(name)),
            None => Ok(None),
            _ => Err(invalid("expected position integer").parameter(name)),
        }
    };
    let list = |name: &str| -> Result<Option<Vec<usize>>, Error> {
        match step.parameters.get(name) {
            Some(Parameter::UnsignedList { values }) => values
                .iter()
                .map(|n| {
                    usize::try_from(*n)
                        .map_err(|_| invalid("position exceeds platform bounds").parameter(name))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Some),
            None => Ok(None),
            _ => Err(invalid("expected tagged unsigned list").parameter(name)),
        }
    };
    let text = |name: &str, default: &str| match step.parameters.get(name) {
        Some(Parameter::Text(value)) => value.clone(),
        _ => default.to_owned(),
    };
    let transform = |name: &'static str, shift_name| {
        let shift = match step.parameters.get(shift_name) {
            Some(Parameter::Integer(n)) => Some(*n as i32),
            _ => None,
        };
        Transformation::parse(&text(name, "identity"), shift)
            .map_err(|e| Error::from_operation(e.parameter(name)))
    };
    let overrides = match step.parameters.get("overrides") {
        Some(Parameter::TextList { values }) => values
            .iter()
            .map(|v| Override::parse(v).map_err(Error::from_operation))
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    Ok(Config {
        index_base: IndexBase::try_from(number("index_base")?.unwrap_or(0) as u8)
            .map_err(Error::from_operation)?,
        selector: Selector::from_options(
            &text("selector", ""),
            list("indices")?,
            number("nth")?,
            list("exclude")?,
        )
        .map_err(Error::from_operation)?,
        selected: transform("selected", "selected_shift")?,
        otherwise: transform("otherwise", "otherwise_shift")?,
        overrides,
    })
}
fn limit() -> Error {
    Error::new(
        ErrorCode::ResourceLimit,
        "masked text bound exceeds pipeline budget",
    )
}
fn selector_entries(config: &Config) -> usize {
    match &config.selector {
        Selector::Indices { indices } => indices.len(),
        Selector::Fibonacci { exclude } => exclude.len(),
        Selector::EveryNth { .. } => 0,
    }
}
/// Charge native list storage as well as UTF-8 payloads; reports are retained in
/// addition to the composable text output, even when no intermediate was requested.
fn audit_bytes(
    config: &Config,
    original: usize,
    deterministic: usize,
    final_text: usize,
    positions: usize,
) -> Result<usize, Error> {
    [
        original,
        deterministic,
        final_text,
        positions.checked_mul(8).ok_or_else(limit)?,
        selector_entries(config).checked_mul(8).ok_or_else(limit)?,
        config.overrides.len().checked_mul(48).ok_or_else(limit)?,
    ]
    .into_iter()
    .try_fold(512usize, |sum, n| sum.checked_add(n).ok_or_else(limit))
}
pub(super) fn output_bound(config: &Config, size: usize) -> Result<usize, Error> {
    if size > text::MAX_INPUT {
        return Err(limit());
    }
    size.checked_add(
        config
            .overrides
            .len()
            .min(size)
            .checked_mul(3)
            .ok_or_else(limit)?,
    )
    .ok_or_else(limit)
}
pub(super) fn audit_bound(config: &Config, size: usize) -> Result<usize, Error> {
    let positions = match &config.selector {
        Selector::Indices { indices } => indices.len(),
        Selector::EveryNth { n } => (size / n).min(text::MAX_INDICES),
        Selector::Fibonacci { .. } => config
            .selector
            .positions(size, config.index_base)
            .map_err(Error::from_operation)?
            .len(),
    };
    audit_bytes(config, size, size, output_bound(config, size)?, positions)
}
pub(super) fn audit_size(report: &Report) -> Result<usize, Error> {
    audit_bytes(
        &report.config,
        report.original_input.len(),
        report.deterministic_text.len(),
        report.final_text.len(),
        report.selected_positions.len(),
    )
}
pub(super) fn work_bound(config: &Config, size: usize) -> Result<u64, Error> {
    let entries = selector_entries(config) as u64;
    // Both selectors sort bounded lists. This charge includes comparisons, copied
    // indices, partition/cipher/recombination buffers and report construction.
    let sort = entries
        .checked_mul(16 * (u64::from(entries.max(1).ilog2()) + 2))
        .ok_or_else(limit)?;
    (size as u64)
        .checked_mul(32)
        .and_then(|n| n.checked_add(sort))
        .and_then(|n| n.checked_add((config.overrides.len() as u64) * 64))
        .and_then(|n| n.checked_add(audit_bound(config, size).ok()? as u64))
        .ok_or_else(limit)
}
