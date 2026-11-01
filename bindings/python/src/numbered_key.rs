//! Thin native transport for the Core Numbered Key constructor and mapper.
use cube_core::ciphers::{mono::numbered_key as n, poly};
use pyo3::{prelude::*, types::PyDict};
fn parse_parameters(
    py: Python<'_>,
    value: &Bound<'_, PyDict>,
    op: &'static str,
) -> PyResult<n::Parameters> {
    let document = super::mono::document(py, value.as_any(), 65536, op, "parameters")?;
    poly::parse_parameters(&document).map_err(|e| super::file_error(e.operation(op)))
}
#[pyfunction]
fn numbered_key_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let p = parse_parameters(py, parameters, "cipher.numbered_key.encode")?;
    let report = n::encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(report).expect("typed report"))
}
#[pyfunction]
fn numbered_key_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.numbered_key.decode";
    let p = parse_parameters(py, parameters, op)?;
    let tokens = super::mono::input::<Vec<String>>(py, tokens, 1048576, op, "tokens")?;
    let report = n::decode(&tokens, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(report).expect("typed report"))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(numbered_key_encode, module)?)?;
    module.add_function(wrap_pyfunction!(numbered_key_decode, module)?)?;
    Ok(())
}
