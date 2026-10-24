//! Native audio adapters; framing and all sample algorithms live in cube-core.
use pyo3::prelude::*;
use std::{io::Cursor, path::PathBuf};
#[pyfunction]
fn inspect_wave(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let info =
        cube_core::media::wave::inspect(&mut Cursor::new(data)).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
#[pyfunction]
fn inspect_wave_file(py: Python<'_>, path: PathBuf) -> PyResult<Py<PyAny>> {
    let info = cube_core::media::wave::inspect_file(&path).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(inspect_wave, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_wave_file, module)?)?;
    Ok(())
}
