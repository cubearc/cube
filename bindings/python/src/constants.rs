//! Native boundary for the bounded exact constant-digit core API.
use cube_core::{
    errors::{ErrorCode, OperationError},
    numbers::constants::{self, Constant, DigitOptions},
};
use pyo3::{prelude::*, types::PyList};

#[pyfunction]
#[pyo3(signature = (constant, *, start=None, count=None, indices=None, max_digits=256))]
fn constant_digits(
    py: Python<'_>,
    constant: &str,
    start: Option<usize>,
    count: Option<usize>,
    indices: Option<&Bound<'_, PyList>>,
    max_digits: usize,
) -> PyResult<Py<PyAny>> {
    let constant = Constant::parse(constant).map_err(super::file_error)?;
    let indices = indices
        .map(|list| -> PyResult<Vec<usize>> {
            if list.len() > constants::MAX_INDICES {
                return Err(super::file_error(
                    OperationError::new(
                        ErrorCode::ResourceLimit,
                        "index selection exceeds 1024 entries",
                    )
                    .operation("number.constant_digits")
                    .parameter("indices"),
                ));
            }
            list.iter().map(|i| i.extract()).collect()
        })
        .transpose()?;
    let result = constants::digits(
        constant,
        DigitOptions {
            start,
            count,
            indices: indices.as_deref(),
            max_digits,
        },
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(constant_digits, module)?)?;
    Ok(())
}
