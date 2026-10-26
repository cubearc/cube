//! Exact native integer transport; no Python arithmetic algorithms.
use cube_core::{
    errors::{ErrorCode, OperationError},
    numbers::modular,
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyInt},
};
fn operand(value: &Bound<'_, PyAny>, op: &'static str, parameter: &'static str) -> PyResult<u64> {
    let invalid = |code| {
        super::file_error(
            OperationError::new(
                code,
                "operand requires a builtin integer in 0..2^64-1 (no bool/float)",
            )
            .operation(op)
            .parameter(parameter),
        )
    };
    if !value.is_exact_instance_of::<PyInt>() {
        return Err(invalid(ErrorCode::InvalidParameter));
    }
    if value.lt(0)? {
        return Err(invalid(ErrorCode::InvalidParameter));
    }
    value
        .extract()
        .map_err(|_| invalid(ErrorCode::ResourceLimit))
}
#[pyfunction]
fn gcd(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>) -> PyResult<u64> {
    const OP: &str = "number.gcd";
    modular::gcd(operand(a, OP, "a")?, operand(b, OP, "b")?).map_err(super::file_error)
}
#[pyfunction]
fn extended_gcd(py: Python<'_>, a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    const OP: &str = "number.extended_gcd";
    let r = modular::extended_gcd(operand(a, OP, "a")?, operand(b, OP, "b")?)
        .map_err(super::file_error)?;
    let result = PyDict::new(py);
    result.set_item("gcd", r.gcd)?;
    result.set_item("x", r.x)?;
    result.set_item("y", r.y)?;
    Ok(result.into_any().unbind())
}
#[pyfunction]
fn mod_inverse(a: &Bound<'_, PyAny>, modulus: &Bound<'_, PyAny>) -> PyResult<u64> {
    const OP: &str = "number.mod_inverse";
    modular::mod_inverse(operand(a, OP, "a")?, operand(modulus, OP, "modulus")?)
        .map_err(super::file_error)
}
#[pyfunction]
fn mod_pow(
    base: &Bound<'_, PyAny>,
    exponent: &Bound<'_, PyAny>,
    modulus: &Bound<'_, PyAny>,
) -> PyResult<u64> {
    const OP: &str = "number.mod_pow";
    modular::mod_pow(
        operand(base, OP, "base")?,
        operand(exponent, OP, "exponent")?,
        operand(modulus, OP, "modulus")?,
    )
    .map_err(super::file_error)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(gcd, module)?)?;
    module.add_function(wrap_pyfunction!(extended_gcd, module)?)?;
    module.add_function(wrap_pyfunction!(mod_inverse, module)?)?;
    module.add_function(wrap_pyfunction!(mod_pow, module)?)?;
    Ok(())
}
