//! Thin native transports for Core-owned bounded numeric profiles.
use cube_core::ciphers::{mono::numeric as n, poly};
use pyo3::{prelude::*, types::PyDict};
fn parameters<P: serde::de::DeserializeOwned>(
    py: Python<'_>,
    value: &Bound<'_, PyDict>,
    op: &'static str,
) -> PyResult<P> {
    poly::parse_parameters(&super::mono::document(
        py,
        value.as_any(),
        65536,
        op,
        "parameters",
    )?)
    .map_err(|e| super::file_error(e.operation(op)))
}
#[pyfunction]
fn prime_number_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.prime_number.encode";
    let p: n::prime::Parameters = self::parameters(py, parameters, op)?;

    let result = n::prime::encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn prime_number_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.prime_number.decode";
    let p: n::prime::Parameters = self::parameters(py, parameters, op)?;
    let tokens = super::mono::input::<Vec<String>>(py, tokens, 1048576, op, "tokens")?;
    let result = n::prime::decode(&tokens, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn prime_multiplication_encode(
    py: Python<'_>,
    groups: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.prime_multiplication.encode";
    let p: n::product::Parameters = self::parameters(py, parameters, op)?;
    let groups = super::mono::input::<Vec<String>>(py, groups, 1048576, op, "groups")?;
    let result = n::product::encode(&groups, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn prime_multiplication_decode(
    py: Python<'_>,
    products: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.prime_multiplication.decode";
    let p: n::product::Parameters = self::parameters(py, parameters, op)?;
    let products = super::mono::input::<Vec<String>>(py, products, 1048576, op, "products")?;
    let result = n::product::decode(&products, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn modulo_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.modulo.encode";
    let p: n::modulo::Parameters = self::parameters(py, parameters, op)?;

    let result = n::modulo::encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn modulo_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.modulo.decode";
    let p: n::modulo::Parameters = self::parameters(py, parameters, op)?;
    let tokens = super::mono::input::<Vec<String>>(py, tokens, 1048576, op, "tokens")?;
    let result = n::modulo::decode(&tokens, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn triliteral_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.triliteral.encode";
    let p: n::triliteral::Parameters = self::parameters(py, parameters, op)?;

    let result = n::triliteral::encode(text, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
#[pyfunction]
fn triliteral_decode(
    py: Python<'_>,
    tokens: &Bound<'_, PyAny>,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    let op = "cipher.triliteral.decode";
    let p: n::triliteral::Parameters = self::parameters(py, parameters, op)?;
    let tokens = super::mono::input::<Vec<String>>(py, tokens, 1048576, op, "tokens")?;
    let result = n::triliteral::decode(&tokens, &p).map_err(super::file_error)?;
    super::structured(py, &serde_json::to_value(result).expect("typed report"))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(prime_number_encode, module)?)?;
    module.add_function(wrap_pyfunction!(prime_number_decode, module)?)?;
    module.add_function(wrap_pyfunction!(prime_multiplication_encode, module)?)?;
    module.add_function(wrap_pyfunction!(prime_multiplication_decode, module)?)?;
    module.add_function(wrap_pyfunction!(modulo_encode, module)?)?;
    module.add_function(wrap_pyfunction!(modulo_decode, module)?)?;
    module.add_function(wrap_pyfunction!(triliteral_encode, module)?)?;
    module.add_function(wrap_pyfunction!(triliteral_decode, module)?)?;
    Ok(())
}
