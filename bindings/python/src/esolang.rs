//! Native configuration/byte/report transport; execution belongs exclusively to Core.
use cube_core::{
    errors::{ErrorCode, OperationError},
    esolang::brainfuck as bf,
};
use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict, PyList},
};

fn config<'a, 'py>(
    value: Option<&'a Bound<'py, PyDict>>,
    allowed: &[&str],
) -> PyResult<Option<&'a Bound<'py, PyDict>>> {
    if let Some(value) = value {
        for key in value.keys() {
            let key: String = key.extract()?;
            if !allowed.contains(&key.as_str()) {
                return Err(super::file_error(
                    OperationError::new(
                        ErrorCode::InvalidParameter,
                        "unknown Brainfuck configuration field",
                    )
                    .operation("esolang.brainfuck.execute"),
                ));
            }
        }
    }
    Ok(value)
}
fn unsigned(value: Option<&Bound<'_, PyDict>>, key: &'static str, default: u64) -> PyResult<u64> {
    value
        .and_then(|d| d.get_item(key).transpose())
        .transpose()?
        .map(|v| super::modular::operand(&v, "esolang.brainfuck.execute", key))
        .transpose()
        .map(|v| v.unwrap_or(default))
}
fn size(value: Option<&Bound<'_, PyDict>>, key: &'static str, default: usize) -> PyResult<usize> {
    usize::try_from(unsigned(value, key, default as u64)?).map_err(|_| {
        super::file_error(
            OperationError::new(
                ErrorCode::InvalidParameter,
                "configuration integer exceeds platform bounds",
            )
            .operation("esolang.brainfuck.execute")
            .parameter(key),
        )
    })
}
fn eof(value: Option<&Bound<'_, PyDict>>) -> PyResult<bf::EofPolicy> {
    let value = value
        .and_then(|d| d.get_item("eof").transpose())
        .transpose()?
        .map(|v| v.extract::<String>())
        .transpose()?
        .unwrap_or_else(|| "error".into());
    bf::EofPolicy::parse(&value).map_err(super::file_error)
}
fn options(value: Option<&Bound<'_, PyDict>>) -> PyResult<bf::Options> {
    let value = config(value, &["cell_bits", "eof"])?;
    let bits = unsigned(value, "cell_bits", 8)?;
    let cell_bits = u8::try_from(bits).map_err(|_| {
        super::file_error(
            OperationError::new(ErrorCode::InvalidParameter, "cell_bits must be 8, 16 or 32")
                .operation("esolang.brainfuck.execute")
                .parameter("cell_bits"),
        )
    })?;
    Ok(bf::Options {
        cell_bits,
        eof: eof(value)?,
    })
}
fn limits(value: Option<&Bound<'_, PyDict>>) -> PyResult<bf::Limits> {
    let value = config(
        value,
        &["max_instructions", "max_output_bytes", "max_memory_bytes"],
    )?;
    Ok(bf::Limits {
        max_instructions: unsigned(value, "max_instructions", bf::MAX_INSTRUCTIONS)?,
        max_output_bytes: size(value, "max_output_bytes", bf::MAX_OUTPUT_BYTES)?,
        max_memory_bytes: size(value, "max_memory_bytes", bf::MAX_MEMORY_BYTES)?,
    })
}
fn diagnostics(value: Option<&Bound<'_, PyDict>>) -> PyResult<bf::Diagnostics> {
    let value = config(
        value,
        &[
            "tape_start",
            "tape_len",
            "max_trace_records",
            "max_trace_bytes",
            "max_report_bytes",
        ],
    )?;
    Ok(bf::Diagnostics {
        tape_start: size(value, "tape_start", 0)?,
        tape_len: size(value, "tape_len", 0)?,
        max_trace_records: size(value, "max_trace_records", 0)?,
        max_trace_bytes: size(value, "max_trace_bytes", 0)?,
        max_report_bytes: size(value, "max_report_bytes", bf::MAX_REPORT_BYTES)?,
    })
}
fn report(py: Python<'_>, r: &bf::Report) -> PyResult<Py<PyAny>> {
    let value = super::structured(
        py,
        &serde_json::json!({
            "completed":r.completed,"budget_scope":r.budget_scope,"overflow_policy":r.overflow_policy,"tape_cells":r.tape_cells,"stop_reason":r.stop_reason,"error":r.error,
            "options":r.options,"limits":r.limits,"diagnostics":r.diagnostics,
            "instructions":r.instructions,"input_consumed":r.input_consumed,"final_pointer":r.final_pointer,
            "tape":r.tape,"trace":r.trace,"trace_complete":r.trace_complete,
            "memory_bytes_upper_bound":r.memory_bytes_upper_bound,"report_bytes_upper_bound":r.report_bytes_upper_bound
        }),
    )?;
    value
        .bind(py)
        .cast::<PyDict>()?
        .set_item("output", PyBytes::new(py, &r.output))?;
    Ok(value)
}
#[pyfunction]
#[pyo3(signature=(source,input=&b""[..],*,options=None,limits=None))]
fn brainfuck_execute<'py>(
    py: Python<'py>,
    source: &str,
    input: &[u8],
    options: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
) -> PyResult<Bound<'py, PyBytes>> {
    let options = self::options(options)?;
    let limits = self::limits(limits)?;
    let bytes = bf::Program::parse(source)
        .and_then(|p| p.execute(input, options, limits))
        .map_err(super::file_error)?;
    Ok(PyBytes::new(py, &bytes))
}
#[pyfunction]
#[pyo3(signature=(source,input=&b""[..],*,options=None,limits=None,diagnostics=None))]
fn brainfuck_report(
    py: Python<'_>,
    source: &str,
    input: &[u8],
    options: Option<&Bound<'_, PyDict>>,
    limits: Option<&Bound<'_, PyDict>>,
    diagnostics: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let options = self::options(options)?;
    let limits = self::limits(limits)?;
    let diagnostics = self::diagnostics(diagnostics)?;
    let result = bf::Program::parse(source)
        .and_then(|p| p.report(input, options, limits, diagnostics))
        .map_err(super::file_error)?;
    report(py, &result)
}
#[pyfunction]
#[pyo3(signature=(source,input=&b""[..],*,options,limits=None,aggregate=None))]
fn brainfuck_compare(
    py: Python<'_>,
    source: &str,
    input: &[u8],
    options: &Bound<'_, PyDict>,
    limits: Option<&Bound<'_, PyDict>>,
    aggregate: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    let options = config(Some(options), &["widths", "eof"])?;
    let widths = if let Some(values) = options
        .and_then(|d| d.get_item("widths").transpose())
        .transpose()?
    {
        let values = values.cast::<PyList>()?;
        if values.len() > 3 {
            return Err(super::file_error(
                OperationError::new(
                    ErrorCode::InvalidParameter,
                    "at most three widths are supported",
                )
                .operation("esolang.brainfuck.compare")
                .parameter("widths"),
            ));
        }
        values
            .iter()
            .map(|v| {
                let n = super::modular::operand(&v, "esolang.brainfuck.compare", "widths")?;
                u8::try_from(n).map_err(|_| {
                    super::file_error(
                        OperationError::new(ErrorCode::InvalidParameter, "invalid cell width")
                            .operation("esolang.brainfuck.compare")
                            .parameter("widths"),
                    )
                })
            })
            .collect::<PyResult<Vec<_>>>()?
    } else {
        return Err(super::file_error(
            OperationError::new(
                ErrorCode::InvalidParameter,
                "comparison requires explicit widths",
            )
            .operation("esolang.brainfuck.compare")
            .parameter("widths"),
        ));
    };
    let limits = self::limits(limits)?;
    let aggregate = self::limits(aggregate)?;
    let result = bf::compare(
        source,
        input,
        &widths,
        eof(options)?,
        limits,
        bf::ComparisonLimits {
            max_instructions: aggregate.max_instructions,
            max_output_bytes: aggregate.max_output_bytes,
            max_memory_bytes: aggregate.max_memory_bytes,
        },
    )
    .map_err(super::file_error)?;
    let value = super::structured(
        py,
        &serde_json::json!({"all_completed":result.all_completed,"equal_completed_outputs":result.equal_completed_outputs,"limits":result.limits,"per_run_limits":result.per_run_limits,"instructions":result.instructions,"output_bytes":result.output_bytes,"memory_bytes_upper_bound":result.memory_bytes_upper_bound}),
    )?;
    let runs = PyList::empty(py);
    for run in &result.runs {
        runs.append(report(py, run)?)?;
    }
    value.bind(py).cast::<PyDict>()?.set_item("runs", runs)?;
    Ok(value)
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(brainfuck_execute, module)?)?;
    module.add_function(wrap_pyfunction!(brainfuck_report, module)?)?;
    module.add_function(wrap_pyfunction!(brainfuck_compare, module)?)?;
    Ok(())
}
