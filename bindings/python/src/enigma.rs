//! Native settings/report transport only; no cipher implementation in the SDK.
use cube_core::ciphers::enigma;
use pyo3::{prelude::*, types::PyDict};

#[pyfunction(name = "enigma")]
fn transform(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        enigma::MAX_PARAMETERS,
        enigma::ID,
        "parameters",
    )?;
    let p = enigma::parse_parameters(&document).map_err(super::file_error)?;
    let report = enigma::transform(text, &p).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report).expect("typed Enigma report"),
    )
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(transform, module)?)
}
