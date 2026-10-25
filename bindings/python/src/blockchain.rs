//! Native validation reports do not serialize or retain private input material.
use cube_core::blockchain::{self, Network, PrivateInput};
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyBytes, PyString},
};
fn material<'a>(
    value: &'a Bound<'_, PyAny>,
    format: &str,
    operation: &'static str,
) -> PyResult<PrivateInput<'a>> {
    let data = if format == "raw" {
        value
            .cast::<PyBytes>()
            .map_err(|_| PyTypeError::new_err("raw private material must be bytes"))?
            .as_bytes()
    } else {
        value
            .cast::<PyString>()
            .map_err(|_| PyTypeError::new_err("hex/WIF private material must be str"))?
            .to_str()
            .map_err(|_| {
                super::file_error(
                    cube_core::errors::OperationError::new(
                        cube_core::errors::ErrorCode::InvalidInput,
                        "key material must be valid UTF-8",
                    )
                    .operation(operation)
                    .parameter("material"),
                )
            })?
            .as_bytes()
    };
    PrivateInput::parse(format, data).map_err(|e| super::file_error(e.operation(operation)))
}
#[pyfunction]
#[pyo3(signature=(text, *, expected_network=None))]
fn validate_wif(
    py: Python<'_>,
    text: &Bound<'_, PyAny>,
    expected_network: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let network = expected_network
        .map(Network::parse)
        .transpose()
        .map_err(super::file_error)?;
    let PrivateInput::Wif(text) = material(text, "wif", "blockchain.wif.validate")? else {
        unreachable!("WIF transport selects WIF input");
    };
    let info = blockchain::wif::validate(text, network).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
#[pyfunction]
#[pyo3(signature=(data, *, format="raw"))]
fn validate_private_scalar(
    py: Python<'_>,
    data: &Bound<'_, PyAny>,
    format: &str,
) -> PyResult<Py<PyAny>> {
    let info = blockchain::validate_scalar(material(data, format, "blockchain.scalar.validate")?)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(validate_wif, module)?)?;
    module.add_function(wrap_pyfunction!(validate_private_scalar, module)?)?;
    Ok(())
}
