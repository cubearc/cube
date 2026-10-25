//! Native iterator over bounded caller-provided keys, with no automatic persistence.
use cube_core::blockchain::{
    Network,
    candidates::{self, Candidate, CandidateBatch, Format},
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyString},
};
#[pyclass(module = "cube._native")]
struct CandidateValidation {
    inner: CandidateBatch,
}
#[pymethods]
impl CandidateValidation {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        self.inner
            .next()
            .map(|entry| super::structured(py, &serde_json::json!(entry)))
            .transpose()
    }
    fn __len__(&self) -> usize {
        self.inner.len()
    }
    fn plan(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        super::structured(py, &serde_json::json!(self.inner.plan()))
    }
    fn progress(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        super::structured(py, &serde_json::json!(self.inner.progress()))
    }
}
fn error(code: cube_core::errors::ErrorCode, message: &str) -> PyErr {
    super::file_error(
        cube_core::errors::OperationError::new(code, message)
            .operation("blockchain.candidates.validate"),
    )
}
fn text<'a>(value: &'a Bound<'_, PyAny>, max: usize) -> PyResult<&'a str> {
    use cube_core::errors::ErrorCode;
    let value = value
        .cast::<PyString>()
        .map_err(|_| error(ErrorCode::InvalidInput, "candidate fields must be strings"))?;
    // Check character count before UTF-8 caching, then exact encoded byte count.
    if value.len()? > max {
        return Err(error(
            ErrorCode::ResourceLimit,
            "candidate field exceeds byte limit",
        ));
    }
    let text = value.to_str().map_err(|_| {
        error(
            ErrorCode::InvalidInput,
            "candidate fields must be valid UTF-8",
        )
    })?;
    if text.len() > max {
        return Err(error(
            ErrorCode::ResourceLimit,
            "candidate field exceeds byte limit",
        ));
    }
    Ok(text)
}
#[pyfunction]
#[pyo3(signature=(candidates, *, derive=None, network=None, compressed=None, checksum=None))]
fn validate_key_candidates(
    candidates: &Bound<'_, PyList>,
    derive: Option<&str>,
    network: Option<&str>,
    compressed: Option<bool>,
    checksum: Option<&str>,
) -> PyResult<CandidateValidation> {
    use cube_core::errors::ErrorCode;
    let mode = candidates::derivation(
        derive,
        network
            .map(Network::parse)
            .transpose()
            .map_err(super::file_error)?,
        compressed,
        checksum,
    )
    .map_err(super::file_error)?;
    if candidates.len() > candidates::MAX_ENTRIES {
        return Err(error(
            ErrorCode::ResourceLimit,
            "candidate count exceeds 256",
        ));
    }
    let mut entries = Vec::with_capacity(candidates.len());
    let mut bytes = 0usize;
    for entry in candidates.iter() {
        let entry = entry.cast::<PyDict>().map_err(|_| {
            error(
                ErrorCode::InvalidInput,
                "candidate entry must contain only format/value strings",
            )
        })?;
        if entry.len() != 2 {
            return Err(error(
                ErrorCode::InvalidInput,
                "candidate entry must contain only format/value strings",
            ));
        }
        let format = entry
            .get_item("format")?
            .ok_or_else(|| error(ErrorCode::InvalidInput, "candidate requires format/value"))?;
        let value = entry
            .get_item("value")?
            .ok_or_else(|| error(ErrorCode::InvalidInput, "candidate requires format/value"))?;
        // Unknown labels map to a fixed classification, never become output data.
        let format = text(&format, 128)?;
        let value = text(&value, candidates::MAX_ENTRY_BYTES)?;
        bytes = bytes
            .checked_add(value.len())
            .ok_or_else(|| error(ErrorCode::ResourceLimit, "candidate byte count overflow"))?;
        if bytes > candidates::MAX_INPUT_BYTES {
            return Err(error(
                ErrorCode::ResourceLimit,
                "candidate input exceeds 32 KiB",
            ));
        }
        entries.push(Candidate::new(Format::parse(format), value).map_err(super::file_error)?);
    }
    Ok(CandidateValidation {
        inner: CandidateBatch::new(entries, mode).map_err(super::file_error)?,
    })
}
pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<CandidateValidation>()?;
    module.add_function(wrap_pyfunction!(validate_key_candidates, module)?)?;
    Ok(())
}
