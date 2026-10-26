//! Exact native signed-magnitude transport; digital-root mathematics stays in core.
use cube_core::{
    errors::{ErrorCode, OperationError},
    numbers::digital,
};
use pyo3::{prelude::*, types::PyInt};
#[pyfunction]
#[pyo3(signature=(value,*,radix=10))]
fn digital_root(value: &Bound<'_, PyAny>, radix: u32) -> PyResult<u8> {
    let invalid = |code| {
        super::file_error(
            OperationError::new(
                code,
                "value requires a builtin integer with magnitude <=2^128-1 (no bool/float)",
            )
            .operation("number.digital_root")
            .parameter("value"),
        )
    };
    if !value.is_exact_instance_of::<PyInt>() {
        return Err(invalid(ErrorCode::InvalidParameter));
    }
    // Check the exact builtin's width before negation can copy caller-sized data.
    if value.call_method0("bit_length")?.extract::<usize>()? > 128 {
        return Err(invalid(ErrorCode::ResourceLimit));
    }
    let magnitude = if value.lt(0)? {
        value.neg()?
    } else {
        value.clone()
    };
    let magnitude = magnitude
        .extract::<u128>()
        .map_err(|_| invalid(ErrorCode::ResourceLimit))?;
    digital::digital_root(magnitude, radix).map_err(super::file_error)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(digital_root, module)?)?;
    Ok(())
}
