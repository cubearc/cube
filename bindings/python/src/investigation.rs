//! Native investigation boundary; all rules and execution remain in the Rust core.
use cube_core::{investigation::triage, operations};
use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature=(data, *, max_read_bytes=12))]
fn triage_input(py: Python<'_>, data: &[u8], max_read_bytes: usize) -> PyResult<Py<PyAny>> {
    let result =
        triage::bytes(data, max_read_bytes, &operations::catalog()).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(triage_input, module)?)?;
    Ok(())
}
