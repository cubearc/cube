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
fn unit(value: Option<&Bound<'_, PyAny>>, op: &'static str) -> PyResult<usize> {
    let value = value
        .map(|v| super::modular::operand(v, op, "unit_bytes"))
        .transpose()?
        .unwrap_or(1);
    if ![1, 2, 4, 8].contains(&value) {
        return Err(super::file_error(
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::InvalidParameter,
                "unit_bytes must be 1, 2, 4 or 8",
            )
            .operation(op)
            .parameter("unit_bytes"),
        ));
    }
    Ok(value as usize)
}
#[pyfunction]
#[pyo3(signature=(data,count,*,unit_bytes=None))]
fn rotate_left<'py>(
    py: Python<'py>,
    data: &[u8],
    count: &Bound<'_, PyAny>,
    unit_bytes: Option<&Bound<'_, PyAny>>,
) -> PyResult<Bound<'py, PyBytes>> {
    let op = "binary.rotate_left";
    result(
        py,
        t::rotate_left(
            data,
            super::modular::operand(count, op, "count")?,
            unit(unit_bytes, op)?,
        ),
    )
}
#[pyfunction]
#[pyo3(signature=(data,count,*,unit_bytes=None))]
fn rotate_right<'py>(
    py: Python<'py>,
    data: &[u8],
    count: &Bound<'_, PyAny>,
    unit_bytes: Option<&Bound<'_, PyAny>>,
) -> PyResult<Bound<'py, PyBytes>> {
    let op = "binary.rotate_right";
    result(
        py,
        t::rotate_right(
            data,
            super::modular::operand(count, op, "count")?,
            unit(unit_bytes, op)?,
        ),
    )
}
#[pyfunction]
#[pyo3(signature=(data,key,*,unit_bytes=None))]
fn add_wrapping<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    unit_bytes: Option<&Bound<'_, PyAny>>,
) -> PyResult<Bound<'py, PyBytes>> {
    result(
        py,
        t::add_wrapping(data, key, unit(unit_bytes, "binary.add_wrapping")?),
    )
}
#[pyfunction]
#[pyo3(signature=(data,key,*,unit_bytes=None))]
fn sub_wrapping<'py>(
    py: Python<'py>,
    data: &[u8],
    key: &[u8],
    unit_bytes: Option<&Bound<'_, PyAny>>,
) -> PyResult<Bound<'py, PyBytes>> {
    result(
        py,
        t::sub_wrapping(data, key, unit(unit_bytes, "binary.sub_wrapping")?),
    )
}
#[pyfunction]
fn xor_exact<'py>(py: Python<'py>, data: &[u8], key: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    result(py, t::xor_exact(data, key))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(and_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(or_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(not_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(reverse_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(shift_left, module)?)?;
    module.add_function(wrap_pyfunction!(shift_right, module)?)?;
    module.add_function(wrap_pyfunction!(swap_endian, module)?)?;
    module.add_function(wrap_pyfunction!(rotate_left, module)?)?;
    module.add_function(wrap_pyfunction!(rotate_right, module)?)?;
    module.add_function(wrap_pyfunction!(xor_exact, module)?)?;
    module.add_function(wrap_pyfunction!(add_wrapping, module)?)?;
    module.add_function(wrap_pyfunction!(sub_wrapping, module)?)?;
    Ok(())
}
