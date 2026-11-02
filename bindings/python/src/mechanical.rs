//! Native transport over Core-owned mechanical cipher implementations.
use cube_core::ciphers::mechanical::{self, alberti, disks, kryha, wheatstone};
use pyo3::{prelude::*, types::PyDict};
fn transform<P: serde::de::DeserializeOwned, R: serde::Serialize>(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
    op: &'static str,
    run: fn(&str, &P) -> Result<R, cube_core::errors::OperationError>,
) -> PyResult<Py<PyAny>> {
    let document = super::mono::document(
        py,
        parameters.as_any(),
        mechanical::MAX_PARAMETERS,
        op,
        "parameters",
    )?;
    let p = mechanical::parse_parameters(&document, op).map_err(super::file_error)?;
    let report = run(text, &p).map_err(super::file_error)?;
    super::structured(
        py,
        &serde_json::to_value(report).expect("typed mechanical report"),
    )
}
#[pyfunction]
fn m94_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.m94.encode", disks::m94_encode)
}
#[pyfunction]
fn m94_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.m94.decode", disks::m94_decode)
}
#[pyfunction]
fn jefferson_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.jefferson.encode",
        disks::jefferson_encode,
    )
}
#[pyfunction]
fn jefferson_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.jefferson.decode",
        disks::jefferson_decode,
    )
}
#[pyfunction]
fn m138a_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.m138a.encode",
        disks::m138a_encode,
    )
}
#[pyfunction]
fn m138a_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.m138a.decode",
        disks::m138a_decode,
    )
}
#[pyfunction]
fn wheatstone_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.wheatstone.encode",
        wheatstone::encode,
    )
}
#[pyfunction]
fn wheatstone_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.wheatstone.decode",
        wheatstone::decode,
    )
}
#[pyfunction]
fn kryha_encode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kryha.encode", kryha::encode)
}
#[pyfunction]
fn kryha_decode(py: Python<'_>, text: &str, parameters: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    transform(py, text, parameters, "cipher.kryha.decode", kryha::decode)
}
#[pyfunction]
fn alberti_encode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.alberti.encode",
        alberti::encode,
    )
}
#[pyfunction]
fn alberti_decode(
    py: Python<'_>,
    text: &str,
    parameters: &Bound<'_, PyDict>,
) -> PyResult<Py<PyAny>> {
    transform(
        py,
        text,
        parameters,
        "cipher.alberti.decode",
        alberti::decode,
    )
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(wheatstone_encode, module)?)?;
    module.add_function(wrap_pyfunction!(wheatstone_decode, module)?)?;
    module.add_function(wrap_pyfunction!(kryha_encode, module)?)?;
    module.add_function(wrap_pyfunction!(kryha_decode, module)?)?;
    module.add_function(wrap_pyfunction!(alberti_encode, module)?)?;
    module.add_function(wrap_pyfunction!(alberti_decode, module)?)?;
    module.add_function(wrap_pyfunction!(m94_encode, module)?)?;
    module.add_function(wrap_pyfunction!(m94_decode, module)?)?;
    module.add_function(wrap_pyfunction!(jefferson_encode, module)?)?;
    module.add_function(wrap_pyfunction!(jefferson_decode, module)?)?;
    module.add_function(wrap_pyfunction!(m138a_encode, module)?)?;
    module.add_function(wrap_pyfunction!(m138a_decode, module)?)
}
