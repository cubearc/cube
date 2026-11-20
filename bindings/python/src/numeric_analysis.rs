//! Native Python bindings preserve the exact integers carried as decimal JSON.
use cube_core::numbers::analysis;
use pyo3::{
    prelude::*,
    types::{PyDict, PyInt, PyList},
};
use serde_json::Value;

fn integer_text(value: &Bound<'_, PyAny>, operation: &'static str) -> PyResult<String> {
    let signed = value.extract::<i128>().map(|n| n.to_string());
    signed
        .or_else(|_| value.extract::<u128>().map(|n| n.to_string()))
        .or_else(|error| {
            if operation == "number.balanced_ternary" && value.lt(0)? {
                return value
                    .call_method0("__abs__")?
                    .extract::<u128>()
                    .map(|n| format!("-{n}"));
            }
            Err(error)
        })
        .map_err(|_| {
            super::discrete_transport::invalid(
                operation,
                "request",
                "integer exceeds the exact 128-bit transport",
            )
        })
}

fn exact_request<'py>(
    value: &Bound<'py, PyAny>,
    field: &str,
    depth: usize,
    remaining: &mut usize,
    operation: &'static str,
) -> PyResult<Bound<'py, PyAny>> {
    *remaining = remaining
        .checked_sub(1)
        .filter(|_| depth <= 8)
        .ok_or_else(|| {
            super::discrete_transport::invalid(
                operation,
                "request",
                "exact scalar document exceeds its structural limits",
            )
        })?;
    let py = value.py();
    if value.is_exact_instance_of::<PyInt>()
        && (matches!(field, "numerator" | "denominator") || value.extract::<i64>().is_err())
    {
        return Ok(pyo3::types::PyString::new(py, &integer_text(value, operation)?).into_any());
    }
    if value.is_exact_instance_of::<PyDict>() {
        let result = PyDict::new(py);
        for (key, item) in value.cast::<PyDict>()?.iter() {
            let name = key.extract::<&str>()?;
            result.set_item(
                &key,
                exact_request(&item, name, depth + 1, remaining, operation)?,
            )?;
        }
        return Ok(result.into_any());
    }
    if value.is_exact_instance_of::<PyList>() {
        let result = PyList::empty(py);
        for item in value.cast::<PyList>()?.iter() {
            result.append(exact_request(
                &item,
                field,
                depth + 1,
                remaining,
                operation,
            )?)?;
        }
        return Ok(result.into_any());
    }
    Ok(value.clone())
}

fn request<'py>(value: &Bound<'py, PyAny>, operation: &'static str) -> PyResult<Bound<'py, PyAny>> {
    if matches!(operation, "number.matrix" | "number.interpolation") {
        return exact_request(value, "", 0, &mut 8192, operation);
    }
    let field = match operation {
        "number.gray" | "number.factoradic" | "number.balanced_ternary" => "value",
        "number.lehmer" | "number.lexicographic_rank" => "rank",
        _ => return Ok(value.clone()),
    };
    if !value.is_exact_instance_of::<PyDict>() {
        return Ok(value.clone());
    }
    let dict = value.cast::<PyDict>()?;
    if let Some(n) = dict
        .get_item(field)?
        .filter(Bound::is_exact_instance_of::<PyInt>)
    {
        if dict.len() > 32 {
            return Err(super::discrete_transport::invalid(
                operation,
                "request",
                "too many request fields",
            ));
        }
        let copy = dict.copy()?;
        copy.set_item(field, integer_text(&n, operation)?)?;
        return Ok(copy.into_any());
    }
    Ok(value.clone())
}

// Integer tokens at these fields carry exact numeric results in the Core wire
// contract. Text such as state IDs, scalars, numeral text and binary strings
// stays text even when its content happens to be decimal digits.
fn numeric_field(operation: &str, field: &str) -> bool {
    match operation {
        "number.gray" | "number.factoradic" | "number.balanced_ternary" => field == "value",
        "number.lfsr" => field == "final_state",
        "graph.weighted_path" => field == "cost",
        "graph.all_pairs_paths" => matches!(field, "cost" | "distances"),
        "number.lehmer" => field == "rank",
        "number.lexicographic_rank" => matches!(field, "rank" | "total"),
        "number.partitions" | "number.permutation_list" => field == "count",
        "number.entropy" => field == "samples",
        "number.crt" => matches!(field, "residue" | "modulus" | "gcd"),
        "number.factor" => field == "factor",
        "number.recurrence" => field == "values",
        "number.matrix" | "number.interpolation" => matches!(field, "numerator" | "denominator"),
        "number.letter_values" => matches!(field, "value" | "sum"),
        _ => false,
    }
}

pub(super) fn restore(
    value: &Bound<'_, PyAny>,
    wire: &Value,
    field: &str,
    operation: &'static str,
    integer: &Bound<'_, PyAny>,
) -> PyResult<()> {
    match wire {
        Value::Object(fields) => {
            let object = value.cast::<PyDict>()?;
            for (key, child) in fields {
                if let Some(text) = child.as_str().filter(|_| numeric_field(operation, key)) {
                    object.set_item(key, integer.call1((text,))?)?;
                } else if let Some(item) = object.get_item(key)? {
                    restore(&item, child, key, operation, integer)?;
                }
            }
        }
        Value::Array(items) => {
            let list = value.cast::<PyList>()?;
            for (index, child) in items.iter().enumerate() {
                if let Some(text) = child.as_str().filter(|_| numeric_field(operation, field)) {
                    list.set_item(index, integer.call1((text,))?)?;
                } else {
                    restore(&list.get_item(index)?, child, field, operation, integer)?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn call(py: Python<'_>, input: &Bound<'_, PyAny>, operation: &'static str) -> PyResult<Py<PyAny>> {
    let request = request(input, operation)?;
    let document = super::discrete_transport::typed_document(
        py,
        &request,
        analysis::MAX_DOCUMENT,
        1_100_000,
        16,
        operation,
        true,
    )?;
    let input = analysis::parse(&document).map_err(super::file_error)?;
    let wire = analysis::invoke_transport(operation, input).map_err(super::file_error)?;
    let report = super::structured(py, &wire)?;
    restore(
        report.bind(py),
        &wire,
        "",
        operation,
        py.get_type::<PyInt>().as_any(),
    )?;
    Ok(report)
}

macro_rules! native_analysis {
    ($($name:ident => $id:literal,)*) => {
        $(
            #[pyfunction]
            fn $name(py: Python<'_>, request: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
                call(py, request, concat!("number.", $id))
            }
        )*
        pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
            $(module.add_function(wrap_pyfunction!($name, module)?)?;)*
            Ok(())
        }
    };
}

native_analysis! {
    number_roman => "roman",
    number_balanced_ternary => "balanced_ternary",
    number_gray => "gray",
    number_decimal_check => "decimal_check",
    number_crt => "crt",
    number_primality => "primality",
    number_sieve => "sieve",
    number_factor => "factor",
    number_recurrence => "recurrence",
    number_lehmer => "lehmer",
    number_factoradic => "factoradic",
    number_lexicographic_rank => "lexicographic_rank",
    number_partitions => "partitions",
    number_de_bruijn => "de_bruijn",
    number_permutation_list => "permutation_list",
    number_matrix => "matrix",
    number_interpolation => "interpolation",
    number_entropy => "entropy",
    number_correlation => "correlation",
    number_runs => "runs",
    number_benford => "benford",
    number_markov => "markov",
    number_letter_values => "letter_values",
}
