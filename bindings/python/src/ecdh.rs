//! Native shared-secret transport; Core owns curve validation and agreement.
use cube_core::asymmetric::ecdh;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn ecdh_agree<'py>(
    py: Python<'py>,
    data: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let s = ecdh::Settings::parse(parameters_json).map_err(super::file_error)?;
    let output = Zeroizing::new(ecdh::agree(data, &s).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(ecdh_agree, module)?)
}
