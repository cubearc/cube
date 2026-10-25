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
fn selection(
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
) -> cube_core::media::pcm::Selection {
    cube_core::media::pcm::Selection {
        channels,
        start_frame,
        frames,
    }
}
fn sample_output(
    py: Python<'_>,
    samples: &cube_core::media::pcm::Samples,
    include_values: bool,
) -> PyResult<Py<PyAny>> {
    let mut info = serde_json::json!(samples.info());
    if include_values {
        info["values"] = serde_json::json!(samples.bounded_values().map_err(super::file_error)?);
    }
    let result = super::structured(py, &info)?;
    result
        .bind(py)
        .set_item("data", pyo3::types::PyBytes::new(py, samples.raw()))?;
    Ok(result)
}
#[pyfunction]
#[pyo3(signature=(data, *, channels=None, start_frame=0, frames=None, include_values=false))]
fn wave_samples(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    include_values: bool,
) -> PyResult<Py<PyAny>> {
    let samples = cube_core::media::pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    sample_output(py, &samples, include_values)
}
#[pyfunction]
#[pyo3(signature=(path, *, channels=None, start_frame=0, frames=None, include_values=false))]
fn wave_file_samples(
    py: Python<'_>,
    path: PathBuf,
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    include_values: bool,
) -> PyResult<Py<PyAny>> {
    let samples =
        cube_core::media::pcm::extract_file(&path, &selection(channels, start_frame, frames))
            .map_err(super::file_error)?;
    sample_output(py, &samples, include_values)
}
#[pyfunction]
#[pyo3(signature=(data, *, channels=None, start_frame=0, frames=None, bucket_frames=1024, max_buckets=1000))]
fn wave_envelope(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    bucket_frames: u64,
    max_buckets: usize,
) -> PyResult<Py<PyAny>> {
    let result = cube_core::media::envelope::from_reader(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
        bucket_frames,
        max_buckets,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
#[pyfunction]
#[pyo3(signature=(path, *, channels=None, start_frame=0, frames=None, bucket_frames=1024, max_buckets=1000))]
fn wave_file_envelope(
    py: Python<'_>,
    path: PathBuf,
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    bucket_frames: u64,
    max_buckets: usize,
) -> PyResult<Py<PyAny>> {
    let result = cube_core::media::envelope::from_file(
        &path,
        &selection(channels, start_frame, frames),
        bucket_frames,
        max_buckets,
    )
    .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
/// Bounded exact-N spectrum of selected integer PCM, peak full-scale amplitudes.
#[pyfunction]
#[pyo3(signature=(data,*,channels=None,start_frame=0,fft_size=1024,window="hann"))]
fn wave_spectrum(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    fft_size: usize,
    window: &str,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::{
        pcm,
        spectrum::{self, Options, Window},
    };
    let options = Options {
        fft_size,
        window: Window::parse(window).map_err(super::file_error)?,
    };
    options.validate().map_err(super::file_error)?;
    let samples = pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, Some(fft_size as u64)),
    )
    .map_err(super::file_error)?;
    let result = spectrum::analyze(&samples, options).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(wave_spectrum, module)?)?;
    module.add_function(wrap_pyfunction!(wave_envelope, module)?)?;
    module.add_function(wrap_pyfunction!(wave_file_envelope, module)?)?;
    module.add_function(wrap_pyfunction!(wave_samples, module)?)?;
    module.add_function(wrap_pyfunction!(wave_file_samples, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_wave, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_wave_file, module)?)?;
    Ok(())
}
