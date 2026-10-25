//! Native exact numeric records, without float conversion or Python algorithms.
use cube_core::{
    errors::{ErrorCode, OperationError},
    text::values,
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyList},
};
#[pyfunction]
fn word_lengths(py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
    let report = values::word_lengths(text).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
#[pyfunction]
#[pyo3(signature=(text,*,format="decimal"))]
fn text_codepoints(py: Python<'_>, text: &str, format: &str) -> PyResult<Py<PyAny>> {
    let report = values::codepoints(text, format).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
fn table(value: &Bound<'_, PyList>) -> PyResult<Vec<(char, u64)>> {
    let invalid = || {
        super::file_error(
            OperationError::new(
                ErrorCode::InvalidParameter,
                "table requires <=4096 exact scalar/u64 pairs, not booleans or floats",
            )
            .operation("text.word.values")
            .parameter("table"),
        )
    };
    if value.len() > 4096 {
        return Err(invalid());
    }
    value
        .iter()
        .map(|entry| {
            let mut pair = entry.try_iter()?;
            let scalar = pair.next().ok_or_else(invalid)??;
            let number = pair.next().ok_or_else(invalid)??;
            if pair.next().is_some() || number.is_instance_of::<PyBool>() {
                return Err(invalid());
            }
            Ok((
                scalar.extract().map_err(|_| invalid())?,
                number.extract().map_err(|_| invalid())?,
            ))
        })
        .collect()
}
#[pyfunction]
#[pyo3(signature=(text,*,table=None,unmapped="error",sums=false,max_sum=u64::MAX))]
fn word_values(
    py: Python<'_>,
    text: &str,
    table: Option<&Bound<'_, PyList>>,
    unmapped: &str,
    sums: bool,
    max_sum: u64,
) -> PyResult<Py<PyAny>> {
    let table = table.map(self::table).transpose()?;
    let report = values::word_values(text, table.as_deref(), unmapped, sums, max_sum)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(word_lengths, module)?)?;
    module.add_function(wrap_pyfunction!(text_codepoints, module)?)?;
    module.add_function(wrap_pyfunction!(word_values, module)?)?;
    Ok(())
}
