//! Tagged bounded Boolean inputs; all logical evaluation stays in Rust.
use super::discrete_transport as t;
use cube_core::logic::boolean as b;
use pyo3::{prelude::*, types::PyDict};
fn limits(v: Option<&Bound<'_, PyDict>>, op: &'static str) -> PyResult<b::Limits> {
    let v = t::config(
        v,
        &[
            "max_work",
            "max_rows",
            "max_memory_bytes",
            "max_report_bytes",
        ],
        op,
    )?;
    Ok(b::Limits {
        max_work: t::number(v, "max_work", b::MAX_WORK, op)?,
        max_rows: t::size(v, "max_rows", 32, op)?,
        max_memory_bytes: t::size(v, "max_memory_bytes", b::MAX_MEMORY, op)?,
        max_report_bytes: t::size(v, "max_report_bytes", b::MAX_REPORT, op)?,
    })
}
#[pyfunction]
#[pyo3(signature=(model,assignment,*,limits=None))]
fn boolean_evaluate(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    assignment: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "logic.boolean.evaluate";
    let model = b::parse(&t::document(py, model, b::MAX_INPUT, 4096, 96, OP)?)
        .map_err(super::file_error)?;
    let assignment = b::parse_assignment(&t::document(py, assignment, b::MAX_INPUT, 128, 8, OP)?)
        .map_err(super::file_error)?;
    let r =
        b::evaluate(&model, &assignment, self::limits(limits, OP)?).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
#[pyfunction]
#[pyo3(signature=(model,*,limits=None))]
fn boolean_analyze(
    py: Python<'_>,
    model: &Bound<'_, PyAny>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    const OP: &str = "logic.boolean.analyze";
    let model = b::parse(&t::document(py, model, b::MAX_INPUT, 4096, 96, OP)?)
        .map_err(super::file_error)?;
    let r = b::analyze(&model, self::limits(limits, OP)?).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(r))
}
pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(boolean_evaluate, m)?)?;
    m.add_function(wrap_pyfunction!(boolean_analyze, m)?)?;
    Ok(())
}
