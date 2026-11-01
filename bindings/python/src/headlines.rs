//! Native transport for the five-headline Core profile.
use cube_core::ciphers::{mono::headlines as h, poly};
use pyo3::{prelude::*, types::PyDict};
fn call(
    py: Python<'_>,
    headlines: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
    decode: bool,
) -> PyResult<Py<PyAny>> {
    let op = if decode {
        "cipher.headlines.decode"
    } else {
        "cipher.headlines.encode"
    };
    let p = poly::parse_parameters::<h::Parameters>(&super::mono::document(
        py,
        parameters.as_any(),
        65536,
        op,
        "parameters",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))?;
    let input = super::mono::input::<Vec<String>>(py, headlines, 1048576, op, "headlines")?;
    let result = h::transform(&input, &p, decode).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn headlines_encode(
    py: Python<'_>,
    headlines: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    call(py, headlines, parameters, false)
}
#[pyfunction]
fn headlines_decode(
    py: Python<'_>,
    headlines: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    call(py, headlines, parameters, true)
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(headlines_encode, module)?)?;
    module.add_function(wrap_pyfunction!(headlines_decode, module)?)?;
    Ok(())
}
