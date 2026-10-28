//! Native structured adapters, without scoring or cipher logic.
use crate::pipeline::CancellationToken;
use cube_core::cryptanalysis as a;
use pyo3::{prelude::*, types::PyDict};
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn language_score(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.language.score",
        "request",
    )?;
    let r: a::scoring::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::scoring::score(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.language.score")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn period_estimate(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.period.estimate",
        "request",
    )?;
    let r: a::period::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::period::estimate(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.period.estimate")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn caesar_rank(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.caesar.rank",
        "request",
    )?;
    let r: a::classical::Caesar = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::classical::caesar(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.caesar.rank")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn affine_rank(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.affine.rank",
        "request",
    )?;
    let r: a::classical::Affine = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::classical::affine(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.affine.rank")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn vigenere_rank(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.vigenere.rank",
        "request",
    )?;
    let r: a::vigenere::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::vigenere::rank(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.vigenere.rank")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn transposition_rank(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.transposition.rank",
        "request",
    )?;
    let r: a::transposition::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::transposition::rank(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.transposition.rank")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn homophonic_inspect(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.homophonic.inspect",
        "request",
    )?;
    let r: a::homophonic::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::homophonic::inspect(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.homophonic.inspect")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
#[pyfunction]
#[pyo3(signature=(request, *, cancellation=None))]
fn crib_verify(
    py: Python<'_>,
    request: &Bound<'_, PyDict>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        request.as_any(),
        a::MAX_REQUEST,
        "analysis.crib.verify",
        "request",
    )?;
    let r: a::crib::Request = a::parse_request(&document).map_err(super::file_error)?;
    let token = cancellation.map(|c| &c.flag);
    let report = py
        .detach(|| a::crib::verify(&r, token))
        .map_err(|e| super::file_error(e.operation("analysis.crib.verify")))?;
    super::structured(
        py,
        &serde_json::to_value(report)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid report"))?,
    )
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(language_score, module)?)?;
    module.add_function(wrap_pyfunction!(period_estimate, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_rank, module)?)?;
    module.add_function(wrap_pyfunction!(affine_rank, module)?)?;
    module.add_function(wrap_pyfunction!(vigenere_rank, module)?)?;
    module.add_function(wrap_pyfunction!(transposition_rank, module)?)?;
    module.add_function(wrap_pyfunction!(homophonic_inspect, module)?)?;
    module.add_function(wrap_pyfunction!(crib_verify, module)?)?;
    Ok(())
}
