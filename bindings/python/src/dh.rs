//! Native transport only; Core owns named groups and private arithmetic.
use cube_core::asymmetric::dh;
use pyo3::{prelude::*, types::PyBytes};
use zeroize::Zeroizing;
#[pyfunction]
fn dh_agree<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    parameters_json: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let settings = dh::Settings::parse(parameters_json, key).map_err(super::file_error)?;
    let output = Zeroizing::new(dh::agree(data, &settings).map_err(super::file_error)?);
    Ok(PyBytes::new(py, &output))
}
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(dh_agree, module)?)?;
    Ok(())
}
