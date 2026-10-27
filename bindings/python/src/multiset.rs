//! Shared Rust scalar preparation, complete inventories and typed reports.
use super::discrete_transport as t;
use cube_core::text::{
    multiset as m,
    statistics::{CaseMode, Normalization, TextFilter},
};
use pyo3::{prelude::*, types::PyDict};
pub(super) fn profile(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<m::Profile> {
    let v = t::config(v, &["normalization", "case", "filter"], op)?;
    Ok(m::Profile {
        normalization: Normalization::parse(&t::text(v, "normalization", "none", op)?)
            .map_err(|e| super::file_error(e.operation(op)))?,
        case: CaseMode::parse(&t::text(v, "case", "preserve", op)?)
            .map_err(|e| super::file_error(e.operation(op)))?,
        filter: TextFilter::parse(&t::text(v, "filter", "all", op)?)
            .map_err(|e| super::file_error(e.operation(op)))?,
    })
}
fn limits(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<m::Limits> {
    let v = t::config(v, &["max_work", "max_memory_bytes", "max_report_bytes"], op)?;
    Ok(m::Limits {
        max_work: t::size(v, "max_work", m::MAX_WORK, op)?,
        max_memory_bytes: t::size(v, "max_memory_bytes", m::MAX_MEMORY, op)?,
        max_report_bytes: t::size(v, "max_report_bytes", m::MAX_REPORT, op)?,
    })
}
#[pyfunction]
#[pyo3(signature=(source,requested,*,profile=None,limits=None))]
fn multiset_compare(
    py: Python<'_>,
    source: &str,
    requested: &str,
    profile: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "text.multiset.compare";
    let r = m::compare(
        source,
        requested,
        self::profile(profile, OP)?,
        self::limits(limits, OP)?,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(source,requested,*,profile=None,limits=None))]
fn multiset_subtract(
    py: Python<'_>,
    source: &str,
    requested: &str,
    profile: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "text.multiset.subtract";
    let r = m::subtract(
        source,
        requested,
        self::profile(profile, OP)?,
        self::limits(limits, OP)?,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(multiset_compare, m)?)?;
    m.add_function(wrap_pyfunction!(multiset_subtract, m)?)?;
    Ok(())
}
