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
/// Windowed spectra; keyword extraction keeps the Rust adapter small and explicit.
#[pyfunction]
#[pyo3(signature=(data,*,channels=None,start_frame=0,frames=None,**options))]
fn wave_spectrogram(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    options: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::{
        pcm,
        spectrogram::{self, FinalWindow, Options},
        spectrum::Window,
    };
    let mut opts = Options::default();
    if let Some(options) = options {
        for (key, value) in options.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "fft_size" => opts.fft_size = value.extract()?,
                "hop_size" => opts.hop_size = value.extract()?,
                "window" => {
                    opts.window = Window::parse(value.extract()?).map_err(super::file_error)?
                }
                "final_window" => {
                    opts.final_window =
                        FinalWindow::parse(value.extract()?).map_err(super::file_error)?
                }
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "unknown spectrogram option",
                    ));
                }
            }
        }
    }
    opts.validate().map_err(super::file_error)?;
    let samples = pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    let report = spectrogram::analyze(&samples, opts).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
/// Native storage bits, binary-safe output and explicit packing metadata.
#[pyfunction]
#[pyo3(signature=(data,*,channels=None,start_frame=0,frames=None,**options))]
fn wave_bits(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    options: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::{
        bits::{self, BitOrder, Incomplete, Options},
        pcm,
    };
    let mut opts = Options::default();
    if let Some(options) = options {
        for (key, value) in options.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "bits" => opts.bits = value.extract()?,
                "reverse_frames" => opts.reverse_frames = value.extract()?,
                "bit_order" => {
                    opts.bit_order = BitOrder::parse(value.extract()?).map_err(super::file_error)?
                }
                "incomplete" => {
                    opts.incomplete =
                        Incomplete::parse(value.extract()?).map_err(super::file_error)?
                }
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "unknown audio bit extraction option",
                    ));
                }
            }
        }
    }
    let samples = pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    let report = bits::extract(&samples, &opts).map_err(super::file_error)?;
    let result = super::structured(py, &serde_json::json!(report))?;
    result
        .bind(py)
        .set_item("data", pyo3::types::PyBytes::new(py, &report.data))?;
    Ok(result)
}
/// Return a new minimal WAV, reversing time frames without changing sample bytes.
#[pyfunction]
#[pyo3(signature=(data,*,channels=None,start_frame=0,frames=None))]
fn reverse_wave<'py>(
    py: Python<'py>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
    let samples = cube_core::media::pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    let output = cube_core::media::reverse::wave(&samples).map_err(super::file_error)?;
    Ok(pyo3::types::PyBytes::new(py, &output))
}
/// Timed nominal DTMF candidates, retaining uncertain windows/runs and evidence.
#[pyfunction]
#[pyo3(signature=(data,*,channels=None,start_frame=0,frames=None,**options))]
fn detect_dtmf(
    py: Python<'_>,
    data: &[u8],
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    options: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::{
        dtmf::{self, Options},
        pcm,
    };
    let mut opts = Options::default();
    if let Some(options) = options {
        for (key, value) in options.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "window_ms" => opts.window_ms = value.extract()?,
                "min_tone_ms" => opts.min_tone_ms = value.extract()?,
                "min_gap_ms" => opts.min_gap_ms = value.extract()?,
                "min_level_dbfs" => opts.min_level_dbfs = value.extract()?,
                "dominance_db" => opts.dominance_db = value.extract()?,
                "max_twist_db" => opts.max_twist_db = value.extract()?,
                "purity_percent" => opts.purity_percent = value.extract()?,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "unknown DTMF option",
                    ));
                }
            }
        }
    }
    opts.validate().map_err(super::file_error)?;
    let samples = pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    let result = dtmf::detect(&samples, opts).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
/// Inspect tone intervals at a caller-supplied frequency without guessing Morse.
#[pyfunction]
#[pyo3(signature=(data,*,frequency_hz,channels=None,start_frame=0,frames=None,**options))]
fn detect_morse_intervals(
    py: Python<'_>,
    data: &[u8],
    frequency_hz: u32,
    channels: Option<Vec<u16>>,
    start_frame: u64,
    frames: Option<u64>,
    options: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::{
        morse::{self, Options},
        pcm,
    };
    let mut opts = Options {
        frequency_hz,
        window_ms: 10,
        min_level_dbfs: -40,
        purity_percent: 80,
    };
    if let Some(options) = options {
        for (key, value) in options.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "window_ms" => opts.window_ms = value.extract()?,
                "min_level_dbfs" => opts.min_level_dbfs = value.extract()?,
                "purity_percent" => opts.purity_percent = value.extract()?,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "unknown Morse interval option",
                    ));
                }
            }
        }
    }
    let samples = pcm::extract(
        &mut Cursor::new(data),
        &selection(channels, start_frame, frames),
    )
    .map_err(super::file_error)?;
    let result = morse::detect(&samples, opts).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
/// Separately interpret a bounded explicit list of interval dictionaries.
#[pyfunction]
#[pyo3(signature=(intervals,*,sample_rate,unit_ms,tolerance_percent=20))]
fn transcribe_morse(
    py: Python<'_>,
    intervals: &Bound<'_, pyo3::types::PyList>,
    sample_rate: u32,
    unit_ms: u32,
    tolerance_percent: u32,
) -> PyResult<Py<PyAny>> {
    use cube_core::media::morse::{self, Interval, State};
    if intervals.len() > morse::MAX_INTERVALS {
        return Err(super::file_error(
            cube_core::errors::OperationError::new(
                cube_core::errors::ErrorCode::ResourceLimit,
                "transcription exceeds 4096 intervals",
            )
            .operation("audio.morse.transcribe"),
        ));
    }
    let mut selected = Vec::with_capacity(intervals.len());
    for item in intervals.iter() {
        let item = item.cast::<pyo3::types::PyDict>()?;
        let get = |name: &str| {
            item.get_item(name)?.ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(
                    "interval requires state, start_frame and frames",
                )
            })
        };
        let state = get("state")?;
        selected.push(Interval {
            state: State::parse(state.extract()?).map_err(super::file_error)?,
            start_frame: get("start_frame")?.extract()?,
            frames: get("frames")?.extract()?,
        });
    }
    let result = morse::transcribe(&selected, sample_rate, unit_ms, tolerance_percent)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
/// Inspect the bounded native FLAC metadata profile; never decode audio/artwork.
#[pyfunction]
fn inspect_audio_metadata(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let result =
        cube_core::media::metadata::inspect(&mut Cursor::new(data)).map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(result))
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(wave_spectrum, module)?)?;
    module.add_function(wrap_pyfunction!(wave_bits, module)?)?;
    module.add_function(wrap_pyfunction!(reverse_wave, module)?)?;
    module.add_function(wrap_pyfunction!(detect_dtmf, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_audio_metadata, module)?)?;
    module.add_function(wrap_pyfunction!(detect_morse_intervals, module)?)?;
    module.add_function(wrap_pyfunction!(transcribe_morse, module)?)?;
    module.add_function(wrap_pyfunction!(wave_spectrogram, module)?)?;
    module.add_function(wrap_pyfunction!(wave_envelope, module)?)?;
    module.add_function(wrap_pyfunction!(wave_file_envelope, module)?)?;
    module.add_function(wrap_pyfunction!(wave_samples, module)?)?;
    module.add_function(wrap_pyfunction!(wave_file_samples, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_wave, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_wave_file, module)?)?;
    Ok(())
}
