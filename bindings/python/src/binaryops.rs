//! Binary-safe native adapters for bounded byte transformations.
use cube_core::binary::transform as t;
use pyo3::{prelude::*, types::PyBytes};
fn result<'py>(
    py: Python<'py>,
    bytes: Result<Vec<u8>, cube_core::errors::OperationError>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(py, &bytes.map_err(super::file_error)?))
}
#[pyfunction]
fn and_bytes<'py>(py: Python<'py>, data: &[u8], key: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::and(data, key))
}
#[pyfunction]
fn or_bytes<'py>(py: Python<'py>, data: &[u8], key: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::or(data, key))
}
#[pyfunction]
fn not_bytes<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::not(data))
}
#[pyfunction]
fn reverse_bytes<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::reverse(data))
}
#[pyfunction]
#[pyo3(signature=(data,bits,*,unit_bytes=1))]
fn shift_left<'py>(
    py: Python<'py>,
    data: &[u8],
    bits: u8,
    unit_bytes: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::shift_left(data, bits, unit_bytes))
}
#[pyfunction]
#[pyo3(signature=(data,bits,*,unit_bytes=1))]
fn shift_right<'py>(
    py: Python<'py>,
    data: &[u8],
    bits: u8,
    unit_bytes: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::shift_right(data, bits, unit_bytes))
}
#[pyfunction]
#[pyo3(signature=(data,*,unit_bytes=1))]
fn swap_endian<'py>(
    py: Python<'py>,
    data: &[u8],
    unit_bytes: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::swap_endian(data, unit_bytes))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(and_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(or_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(not_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(reverse_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(shift_left, module)?)?;
    module.add_function(wrap_pyfunction!(shift_right, module)?)?;
    module.add_function(wrap_pyfunction!(swap_endian, module)?)?;
    Ok(())
}
