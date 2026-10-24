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
    module.add_class::<NativeSession>()?;
    module.add_function(wrap_pyfunction!(plan_investigation, module)?)?;
    module.add_function(wrap_pyfunction!(investigate, module)?)?;
    Ok(())
}

use crate::pipeline::{self as binding, CancellationToken};
use cube_core::investigation::session::{Capture, Export, RunSpec, Session, SessionLimits};
use pyo3::types::{PyDict, PyList};
fn export(results: Option<Vec<u32>>, parameters: Option<Vec<u32>>, hypotheses: bool) -> Export {
    Export {
        include_results: results.unwrap_or_default(),
        include_parameters: parameters.unwrap_or_default(),
        include_hypotheses: hypotheses,
    }
}
/// Caller-owned in-memory provenance, without automatic saving or raw input retention.
#[pyclass(name = "InvestigationSession", module = "cube._native")]
struct NativeSession {
    inner: Session,
}
#[pymethods]
impl NativeSession {
    #[new]
    #[pyo3(signature=(data, *, identify=false))]
    fn new(py: Python<'_>, data: &[u8], identify: bool) -> PyResult<Self> {
        let mut inner = Session::new(data.len() as u64, SessionLimits::default())
            .map_err(|e| binding::error(py, e))?;
        if identify {
            inner
                .execute(data, RunSpec::Identity, Capture::default(), None)
                .map_err(|e| binding::error(py, e))?;
        }
        Ok(Self { inner })
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, document: &str) -> PyResult<Self> {
        Ok(Self {
            inner: Session::from_json(document).map_err(|e| binding::error(py, e))?,
        })
    }
    #[pyo3(signature=(data, definition, *, record_parameters=false, retain_result=false, cancellation=None))]
    fn run(
        &mut self,
        py: Python<'_>,
        data: &[u8],
        definition: &str,
        record_parameters: bool,
        retain_result: bool,
        cancellation: Option<&CancellationToken>,
    ) -> PyResult<u32> {
        let value =
            cube_core::pipeline::json::parse(definition).map_err(|e| binding::error(py, e))?;
        let spec: RunSpec = serde_json::from_value(value)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid run contract schema"))?;
        let token = cancellation.map(|c| &c.flag);
        py.detach(|| {
            self.inner.execute(
                data,
                spec,
                Capture {
                    parameters: record_parameters,
                    result: retain_result,
                },
                token,
            )
        })
        .map_err(|e| binding::error(py, e))
    }
    #[pyo3(signature=(data, steps, *, record_parameters=false, retain_result=false, cancellation=None))]
    fn execute_pipeline(
        &mut self,
        py: Python<'_>,
        data: &[u8],
        steps: &Bound<'_, PyList>,
        record_parameters: bool,
        retain_result: bool,
        cancellation: Option<&CancellationToken>,
    ) -> PyResult<u32> {
        let spec = RunSpec::pipeline(
            binding::steps(py, steps)?,
            cube_core::pipeline::Limits::default(),
        );
        let token = cancellation.map(|c| &c.flag);
        py.detach(|| {
            self.inner.execute(
                data,
                spec,
                Capture {
                    parameters: record_parameters,
                    result: retain_result,
                },
                token,
            )
        })
        .map_err(|e| binding::error(py, e))
    }
    #[pyo3(signature=(text, findings=None))]
    fn add_hypothesis(
        &mut self,
        py: Python<'_>,
        text: &str,
        findings: Option<Vec<u32>>,
    ) -> PyResult<u32> {
        self.inner
            .add_hypothesis(text, &findings.unwrap_or_default())
            .map_err(|e| binding::error(py, e))
    }
    fn redact_parameters(&mut self, py: Python<'_>, run_id: u32) -> PyResult<()> {
        self.inner
            .redact_parameters(run_id)
            .map_err(|e| binding::error(py, e))
    }
    fn discard_result(&mut self, py: Python<'_>, run_id: u32) -> PyResult<()> {
        self.inner
            .discard_result(run_id)
            .map_err(|e| binding::error(py, e))
    }
    #[pyo3(signature=(*, include_results=None, include_parameters=None, include_hypotheses=false))]
    fn to_json(
        &self,
        py: Python<'_>,
        include_results: Option<Vec<u32>>,
        include_parameters: Option<Vec<u32>>,
        include_hypotheses: bool,
    ) -> PyResult<String> {
        self.inner
            .to_json(&export(
                include_results,
                include_parameters,
                include_hypotheses,
            ))
            .map_err(|e| binding::error(py, e))
    }
    fn report(&self) -> String {
        self.inner.human()
    }
    #[pyo3(signature=(data, *, cancellation=None))]
    fn replay(
        &self,
        py: Python<'_>,
        data: &[u8],
        cancellation: Option<&CancellationToken>,
    ) -> PyResult<Self> {
        let token = cancellation.map(|c| &c.flag);
        Ok(Self {
            inner: py
                .detach(|| self.inner.replay(data, token))
                .map_err(|e| binding::error(py, e))?,
        })
    }
}

use cube_core::investigation::profile::Profile;
fn profile(py: Python<'_>, document: Option<&str>) -> PyResult<Profile> {
    document
        .map(Profile::parse)
        .transpose()
        .map(Option::unwrap_or_default)
        .map_err(|e| binding::error(py, e))
}
#[pyfunction]
#[pyo3(signature=(input_size, profile_json=None))]
fn plan_investigation(
    py: Python<'_>,
    input_size: u64,
    profile_json: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let plan = profile(py, profile_json)?
        .dry_run(input_size)
        .map_err(|e| binding::error(py, e))?;
    super::structured(py, &serde_json::json!(plan))
}
#[pyfunction]
#[pyo3(signature=(data, profile_json=None, *, cancellation=None))]
fn investigate(
    py: Python<'_>,
    data: &[u8],
    profile_json: Option<&str>,
    cancellation: Option<&CancellationToken>,
) -> PyResult<Py<PyAny>> {
    let profile = profile(py, profile_json)?;
    let token = cancellation.map(|c| &c.flag);
    let result = py
        .detach(|| profile.run_bytes(data, token))
        .map_err(|e| binding::error(py, e))?;
    let output = PyDict::new(py);
    output.set_item(
        "plan",
        super::structured(py, &serde_json::json!(result.plan))?,
    )?;
    output.set_item("metrics",super::structured(py,&serde_json::json!({"input_read_bytes":result.input_read_bytes,"inspected_bytes":result.inspected_bytes,"operation_units":result.operation_units,"elapsed_ms":result.elapsed_ms}))?)?;
    output.set_item(
        "session",
        Py::new(
            py,
            NativeSession {
                inner: result.session,
            },
        )?,
    )?;
    Ok(output.into_any().unbind())
}
