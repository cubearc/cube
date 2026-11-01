//! Native transport for the fixed Core Solresol profile.
use cube_core::{ciphers::poly, symbols::solresol as s};
use pyo3::{prelude::*, types::PyDict};
fn call(
    py: Python<'_>,
    input: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
    encode: bool,
) -> PyResult<Py<PyAny>> {
    let op = if encode {
        "symbol.solresol.encode"
    } else {
        "symbol.solresol.decode"
    };
    let field = if encode { "terms" } else { "tokens" };
    let p: s::Parameters = poly::parse_parameters(&super::mono::document(
        py,
        parameters.as_any(),
        65536,
        op,
        "parameters",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))?;
    let input = super::mono::input::<Vec<String>>(py, input, 1048576, op, field)?;
    let result = if encode {
        serde_json::to_value(s::encode(&input, &p).map_err(super::file_error)?)
    } else {
        serde_json::to_value(s::decode(&input, &p).map_err(super::file_error)?)
    }
    .expect("typed report");
    super::structured(py, &result)
}
#[pyfunction]
fn solresol_encode(
    py: Python<'_>,
    terms: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    call(py, terms, parameters, true)
}
#[pyfunction]
fn solresol_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    call(py, tokens, parameters, false)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(solresol_encode, module)?)?;
    module.add_function(wrap_pyfunction!(solresol_decode, module)?)?;
    Ok(())
}
