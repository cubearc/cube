//! Native structured adapters, without scoring or cipher logic.
use cube_core::cryptanalysis as a;
use pyo3::{prelude::*, types::PyDict};
#[pyfunction]
fn language_score(py: Python<'_>, request: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.language.score",
        "request",
    )?;
    let r: a::scoring::Request = a::parse_request(&document).map_err(super::file_error)?;
    let report = a::scoring::score(&r, None).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
fn period_estimate(py: Python<'_>, request: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.period.estimate",
        "request",
    )?;
    let r: a::period::Request = a::parse_request(&document).map_err(super::file_error)?;
    let report = a::period::estimate(&r, None).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(language_score, module)?)?;
    module.add_function(wrap_pyfunction!(period_estimate, module)?)?;
    Ok(())
}
