//! Native image boundary; all decoding, sample work and validation stay in Rust.
use cube_core::{
    forensics,
    image::{ColorModel, Image},
};
use pyo3::{prelude::*, types::PyBytes};
use std::{io::Cursor, path::PathBuf};

fn region(value: Option<(u32, u32, u32, u32)>) -> Option<cube_core::image::Region> {
    value.map(|(x, y, width, height)| cube_core::image::Region {
        x,
        y,
        width,
        height,
    })
}
fn channels(names: Option<Vec<String>>) -> PyResult<Option<Vec<cube_core::image::Channel>>> {
    names
        .map(|names| {
            names
                .iter()
                .map(|name| cube_core::image::Channel::parse(name).map_err(super::file_error))
                .collect()
        })
        .transpose()
}
fn pixel_data(py: Python<'_>, data: &cube_core::image::PixelData) -> PyResult<Py<PyAny>> {
    let result = super::structured(py, &serde_json::json!(data))?;
    result
        .bind(py)
        .set_item("data", PyBytes::new(py, &data.data))?;
    Ok(result)
}

/// Immutable PNG8 samples. Transformations return another Image.
#[pyclass(name = "Image", module = "cube._native", frozen)]
pub struct PyImage {
    pub(super) inner: Image,
}

#[pymethods]
impl PyImage {
    #[pyo3(signature = (x, y, *, expand_palette=false))]
    fn pixel<'py>(
        &self,
        py: Python<'py>,
        x: u32,
        y: u32,
        expand_palette: bool,
    ) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let values = self
            .inner
            .view(expand_palette)
            .map_err(super::file_error)?
            .pixel(x, y)
            .map_err(super::file_error)?;
        pyo3::types::PyList::new(py, values)
    }
    #[pyo3(signature = (*, region=None, channels=None, expand_palette=false))]
    fn pixels(
        &self,
        py: Python<'_>,
        region: Option<(u32, u32, u32, u32)>,
        channels: Option<Vec<String>>,
        expand_palette: bool,
    ) -> PyResult<Py<PyAny>> {
        let selected = crate::image::channels(channels)?;
        let selected_region = crate::image::region(region);
        let data = self
            .inner
            .view(expand_palette)
            .map_err(super::file_error)?
            .pixels(selected_region, selected.as_deref())
            .map_err(super::file_error)?;
        pixel_data(py, &data)
    }
    #[pyo3(signature = (channel, *, region=None, expand_palette=false))]
    fn channel(
        &self,
        py: Python<'_>,
        channel: &str,
        region: Option<(u32, u32, u32, u32)>,
        expand_palette: bool,
    ) -> PyResult<Py<PyAny>> {
        let name = cube_core::image::Channel::parse(channel).map_err(super::file_error)?;
        let data = self
            .inner
            .view(expand_palette)
            .map_err(super::file_error)?
            .channel(name, crate::image::region(region))
            .map_err(super::file_error)?;
        pixel_data(py, &data)
    }
    #[pyo3(signature = (*, region=None, channels=None, max_colors=1000, expand_palette=false))]
    fn histogram(
        &self,
        py: Python<'_>,
        region: Option<(u32, u32, u32, u32)>,
        channels: Option<Vec<String>>,
        max_colors: usize,
        expand_palette: bool,
    ) -> PyResult<Py<PyAny>> {
        let selected = crate::image::channels(channels)?;
        let result = self
            .inner
            .view(expand_palette)
            .map_err(super::file_error)?
            .histogram(
                crate::image::region(region),
                selected.as_deref(),
                max_colors,
            )
            .map_err(super::file_error)?;
        super::structured(py, &serde_json::json!(result))
    }
    #[pyo3(signature = (other, *, region=None, channels=None, max_results=1000, mask=false))]
    fn compare(
        &self,
        py: Python<'_>,
        other: &Self,
        region: Option<(u32, u32, u32, u32)>,
        channels: Option<Vec<String>>,
        max_results: usize,
        mask: bool,
    ) -> PyResult<Py<PyAny>> {
        let selected = crate::image::channels(channels)?;
        let result = self
            .inner
            .compare(
                &other.inner,
                crate::image::region(region),
                selected.as_deref(),
                max_results,
                mask,
            )
            .map_err(super::file_error)?;
        let output = super::structured(py, &serde_json::json!(result))?;
        output.bind(py).set_item(
            "mask",
            result
                .mask
                .map(|inner| Py::new(py, Self { inner }))
                .transpose()?,
        )?;
        Ok(output)
    }
    fn rotate(&self, degrees: u32) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.rotate(degrees).map_err(super::file_error)?,
        })
    }
    fn mirror(&self, axis: &str) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.mirror(axis).map_err(super::file_error)?,
        })
    }
    fn crop(&self, region: (u32, u32, u32, u32)) -> PyResult<Self> {
        Ok(Self {
            inner: self
                .inner
                .crop(crate::image::region(Some(region)).expect("present region"))
                .map_err(super::file_error)?,
        })
    }
    #[pyo3(signature=(*, channels=None, expand_palette=false))]
    fn invert(&self, channels: Option<Vec<String>>, expand_palette: bool) -> PyResult<Self> {
        let selected = crate::image::channels(channels)?;
        Ok(Self {
            inner: self
                .inner
                .view(expand_palette)
                .map_err(super::file_error)?
                .invert(selected.as_deref())
                .map_err(super::file_error)?,
        })
    }
    #[pyo3(signature=(channels, *, expand_palette=false))]
    fn reorder(&self, channels: Vec<String>, expand_palette: bool) -> PyResult<Self> {
        let selected = crate::image::channels(Some(channels))?.expect("present channels");
        Ok(Self {
            inner: self
                .inner
                .view(expand_palette)
                .map_err(super::file_error)?
                .reorder(&selected)
                .map_err(super::file_error)?,
        })
    }
    #[pyo3(signature=(*, expand_palette=false))]
    fn grayscale(&self, expand_palette: bool) -> PyResult<Self> {
        Ok(Self {
            inner: self
                .inner
                .view(expand_palette)
                .map_err(super::file_error)?
                .grayscale()
                .map_err(super::file_error)?,
        })
    }
    #[pyo3(signature=(channel, bit, *, region=None, expand_palette=false, visualization=false))]
    fn bit_plane(
        &self,
        py: Python<'_>,
        channel: &str,
        bit: u8,
        region: Option<(u32, u32, u32, u32)>,
        expand_palette: bool,
        visualization: bool,
    ) -> PyResult<Py<PyAny>> {
        let channel = cube_core::image::Channel::parse(channel).map_err(super::file_error)?;
        let result = self
            .inner
            .view(expand_palette)
            .map_err(super::file_error)?
            .bit_plane(channel, bit, crate::image::region(region))
            .map_err(super::file_error)?;
        let output = super::structured(py, &serde_json::json!(result))?;
        output
            .bind(py)
            .get_item("pixels")?
            .set_item("data", PyBytes::new(py, &result.pixels.data))?;
        output
            .bind(py)
            .set_item("packed", PyBytes::new(py, &result.packed))?;
        output.bind(py).set_item(
            "image",
            if visualization {
                Some(Py::new(
                    py,
                    Self {
                        inner: result.visualization().map_err(super::file_error)?,
                    },
                )?)
            } else {
                None
            },
        )?;
        Ok(output)
    }
    #[pyo3(signature=(**kwargs))]
    fn lsb(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        use cube_core::image::{BitOrder, Incomplete, LsbOptions, Traversal};
        let empty = pyo3::types::PyDict::new(py);
        let kwargs = kwargs.unwrap_or(&empty);
        for key in kwargs.keys() {
            let name: String = key.extract()?;
            if ![
                "region",
                "channels",
                "bits",
                "traversal",
                "reverse",
                "bit_order",
                "incomplete",
                "expand_palette",
            ]
            .contains(&name.as_str())
            {
                return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                    "unknown LSB option '{name}'"
                )));
            }
        }
        let region = kwargs
            .get_item("region")?
            .map(|v| v.extract::<Option<(u32, u32, u32, u32)>>())
            .transpose()?
            .flatten();
        let names = kwargs
            .get_item("channels")?
            .map(|v| v.extract::<Option<Vec<String>>>())
            .transpose()?
            .flatten();
        let bits = kwargs
            .get_item("bits")?
            .map(|v| v.extract::<Vec<u8>>())
            .transpose()?
            .unwrap_or_else(|| vec![0]);
        let traversal = kwargs
            .get_item("traversal")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .unwrap_or_else(|| "row".into());
        let order = kwargs
            .get_item("bit_order")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .unwrap_or_else(|| "msb".into());
        let incomplete = kwargs
            .get_item("incomplete")?
            .map(|v| v.extract::<String>())
            .transpose()?
            .unwrap_or_else(|| "reject".into());
        let reverse = kwargs
            .get_item("reverse")?
            .map(|v| v.extract::<bool>())
            .transpose()?
            .unwrap_or(false);
        let expand = kwargs
            .get_item("expand_palette")?
            .map(|v| v.extract::<bool>())
            .transpose()?
            .unwrap_or(false);
        let options = LsbOptions {
            region: crate::image::region(region),
            channels: crate::image::channels(names)?,
            bits,
            traversal: Traversal::parse(&traversal).map_err(super::file_error)?,
            reverse,
            bit_order: BitOrder::parse(&order).map_err(super::file_error)?,
            incomplete: Incomplete::parse(&incomplete).map_err(super::file_error)?,
        };
        let result = self
            .inner
            .view(expand)
            .map_err(super::file_error)?
            .lsb(&options)
            .map_err(super::file_error)?;
        let output = super::structured(py, &serde_json::json!(result))?;
        output
            .bind(py)
            .set_item("data", PyBytes::new(py, &result.data))?;
        Ok(output)
    }
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
