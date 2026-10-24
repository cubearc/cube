//! Native image boundary; all decoding, sample work and validation stay in Rust.
use cube_core::{
    forensics,
    image::{ColorModel, Image},
};
use pyo3::{prelude::*, types::PyBytes};
use std::{io::Cursor, path::PathBuf};

/// Immutable PNG8 samples. Transformations return another Image.
#[pyclass(name = "Image", module = "cube._native", frozen)]
pub struct PyImage {
    pub(super) inner: Image,
}

#[pymethods]
impl PyImage {
    #[staticmethod]
    fn decode(data: &[u8]) -> PyResult<Self> {
        Ok(Self {
            inner: Image::decode(data).map_err(super::file_error)?,
        })
    }
    #[staticmethod]
    fn open(path: PathBuf) -> PyResult<Self> {
        Ok(Self {
            inner: Image::open(&path).map_err(super::file_error)?,
        })
    }
    #[getter]
    fn width(&self) -> u32 {
        self.inner.width()
    }
    #[getter]
    fn height(&self) -> u32 {
        self.inner.height()
    }
    #[getter]
    fn samples<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.inner.samples())
    }
    fn info(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        super::structured(py, &serde_json::json!(self.inner.info()))
    }
    fn exif(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        super::structured(
            py,
            &serde_json::json!(self.inner.exif().map_err(super::file_error)?),
        )
    }
    fn expand_palette(&self) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.expand_palette().map_err(super::file_error)?,
        })
    }
    fn encode_png<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(
            py,
            &self.inner.encode_png().map_err(super::file_error)?,
        ))
    }
    fn save_png(&self, path: PathBuf) -> PyResult<()> {
        self.inner.save_png(&path).map_err(super::file_error)
    }
    fn __repr__(&self) -> String {
        format!(
            "Image({}x{}, {:?})",
            self.inner.width(),
            self.inner.height(),
            self.inner.model()
        )
    }
    #[staticmethod]
    #[pyo3(signature = (width, height, model, samples, palette=None, transparency=None))]
    fn from_samples(
        width: u32,
        height: u32,
        model: &str,
        samples: &[u8],
        palette: Option<Vec<[u8; 3]>>,
        transparency: Option<&[u8]>,
    ) -> PyResult<Self> {
        let model = ColorModel::parse(model).map_err(super::file_error)?;
        Ok(Self {
            inner: Image::from_bytes(
                width,
                height,
                model,
                samples,
                &palette.unwrap_or_default(),
                transparency.unwrap_or_default(),
            )
            .map_err(super::file_error)?,
        })
    }
}

/// Inspect strict PNG chunks, or explicitly report CRC mismatches under safe framing.
#[pyfunction]
#[pyo3(signature = (data, *, strict=true, payload_budget=0))]
pub fn png_chunks(
    py: Python<'_>,
    data: &[u8],
    strict: bool,
    payload_budget: usize,
) -> PyResult<Py<PyAny>> {
    let report = forensics::png::inspect(&mut Cursor::new(data), strict, payload_budget)
        .map_err(super::file_error)?;
    super::structured(py, &serde_json::json!(report))
}
