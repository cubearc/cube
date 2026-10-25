//! Native wrappers for bounded video metadata; no decoder or network backend.
use pyo3::prelude::*;
#[pyfunction]
fn inspect_video(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let info = cube_core::media::video::inspect(&mut std::io::Cursor::new(data))
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(inspect_video, module)?)?;
    Ok(())
}
