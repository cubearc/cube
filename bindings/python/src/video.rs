//! Native wrappers for bounded video metadata; no decoder or network backend.
use pyo3::prelude::*;
#[pyfunction]
fn extract_video_frames(
    py: Python<'_>,
    data: &[u8],
    track_id: u64,
    indexes: &Bound<'_, pyo3::types::PyList>,
) -> PyResult<Vec<Py<PyAny>>> {
    if indexes.is_empty() || indexes.len() > 8 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "select 1..=8 unique frame indexes",
        ));
    }
    let indexes = indexes.extract::<Vec<usize>>()?;
    let frames = cube_core::media::video::frames::extract(
        &mut std::io::Cursor::new(data),
        track_id,
        &indexes,
    )
    .map_err(super::file_error)?;
    frames
        .into_iter()
        .map(|frame| {
            let result = super::structured(py, &serde_json::json!(frame))?;
            result.bind(py).set_item(
                "image",
                Py::new(py, super::image::PyImage { inner: frame.image })?,
            )?;
            Ok(result)
        })
        .collect()
}
#[pyfunction]
fn inspect_video(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let info = cube_core::media::video::inspect(&mut std::io::Cursor::new(data))
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(info))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(extract_video_frames, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_video, module)?)?;
    Ok(())
}
