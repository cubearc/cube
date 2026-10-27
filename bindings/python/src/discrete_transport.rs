//! Bounded builtin JSON/configuration transport for concrete discrete APIs.
use cube_core::errors::{ErrorCode, OperationError};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyInt, PyList, PyString},
};
pub(super) fn invalid(op: &'static str, name: &'static str, msg: &str) -> PyErr {
    super::file_error(
        OperationError::new(ErrorCode::InvalidParameter, msg)
            .operation(op)
            .parameter(name),
    )
}
fn input(op: &'static str, name: &'static str, msg: &str) -> PyErr {
    super::file_error(
        OperationError::new(ErrorCode::InvalidInput, msg)
            .operation(op)
            .parameter(name),
    )
}
fn resource(op: &'static str, name: &'static str, msg: &str) -> PyErr {
    super::file_error(
        OperationError::new(ErrorCode::ResourceLimit, msg)
            .operation(op)
            .parameter(name),
    )
}
pub(super) fn config<'a, 'py>(
    v: Option<&'a Bound<'py, PyDict>>,
    names: &[&str],
    op: &'static str,
) -> PyResult<Option<&'a Bound<'py, PyDict>>> {
    if let Some(v) = v {
        if v.len() > names.len() {
            return Err(invalid(op, "options", "too many configuration fields"));
        }
        for k in v.keys() {
            if k.len()? > 64 {
                return Err(invalid(op, "options", "configuration key is too long"));
            }
            let key = k.extract::<&str>()?;
            if !names.contains(&key) {
                return Err(invalid(op, "options", "unknown configuration field"));
            }
        }
    }
    Ok(v)
}
pub(super) fn number(
    v: Option<&Bound<'_, PyDict>>,
    name: &'static str,
    default: u64,
    op: &'static str,
) -> PyResult<u64> {
    v.and_then(|v| v.get_item(name).transpose())
        .transpose()?
        .map(|v| super::modular::operand(&v, op, name))
        .transpose()
        .map(|v| v.unwrap_or(default))
}
pub(super) fn size(
    v: Option<&Bound<'_, PyDict>>,
    name: &'static str,
    default: usize,
    op: &'static str,
) -> PyResult<usize> {
    usize::try_from(number(v, name, default as u64, op)?)
        .map_err(|_| invalid(op, name, "integer exceeds platform bounds"))
}
pub(super) fn text(
    v: Option<&Bound<'_, PyDict>>,
    name: &'static str,
    default: &str,
    op: &'static str,
) -> PyResult<String> {
    let value = v.and_then(|v| v.get_item(name).transpose()).transpose()?;
    if let Some(value) = value {
        let s = value.cast::<PyString>()?;
        if s.as_any().len()? > 32 {
            return Err(invalid(op, name, "profile choice exceeds 32 characters"));
        }
        let s = s.to_str()?;
        if s.len() > 32 {
            return Err(invalid(op, name, "profile choice exceeds 32 UTF-8 bytes"));
        }
        Ok(s.to_owned())
    } else {
        Ok(default.into())
    }
}
fn string_size(s: &str) -> usize {
    2 + s
        .chars()
        .map(|c| match c {
            '"' | '\\' => 2,
            c if (c as u32) < 32 => 6,
            c => c.len_utf8(),
        })
        .sum::<usize>()
}
struct JsonBudget {
    bytes: usize,
    nodes: usize,
    max_bytes: usize,
    max_nodes: usize,
    max_depth: usize,
    op: &'static str,
}
impl JsonBudget {
    fn walk(&mut self, v: &Bound<'_, PyAny>, depth: usize) -> PyResult<()> {
        self.nodes += 1;
        if self.nodes > self.max_nodes || depth > self.max_depth {
            return Err(resource(
                self.op,
                "document",
                "JSON node/depth budget exceeded",
            ));
        }
        let add;
        if v.is_exact_instance_of::<PyString>() {
            if v.len()? > self.max_bytes {
                return Err(resource(
                    self.op,
                    "document",
                    "JSON string exceeds byte budget",
                ));
            }
            let s = v.extract::<&str>()?;
            if s.len() > self.max_bytes {
                return Err(resource(
                    self.op,
                    "document",
                    "JSON string exceeds byte budget",
                ));
            }
            add = string_size(s);
        } else if v.is_none() {
            add = 4;
        } else if v.is_exact_instance_of::<PyBool>() {
            add = 5;
        } else if v.is_exact_instance_of::<PyInt>() {
            if v.extract::<i64>().is_err() && v.extract::<u64>().is_err() {
                return Err(input(
                    self.op,
                    "document",
                    "JSON integer exceeds fixed-width transport",
                ));
            }
            add = 21;
        } else if v.is_exact_instance_of::<PyList>() {
            let a = v.cast::<PyList>()?;
            if a.len() > self.max_nodes - self.nodes {
                return Err(resource(
                    self.op,
                    "document",
                    "JSON array exceeds node budget",
                ));
            }
            add = 2 + a.len();
            for v in a.iter() {
                self.walk(&v, depth + 1)?;
            }
        } else if v.is_exact_instance_of::<PyDict>() {
            let m = v.cast::<PyDict>()?;
            if m.len() > self.max_nodes - self.nodes {
                return Err(resource(
                    self.op,
                    "document",
                    "JSON object exceeds node budget",
                ));
            }
            add = 2 + 2 * m.len();
            for (k, v) in m.iter() {
                if !k.is_exact_instance_of::<PyString>() {
                    return Err(input(
                        self.op,
                        "document",
                        "JSON object keys must be builtin strings",
                    ));
                }
                if k.len()? > self.max_bytes {
                    return Err(resource(
                        self.op,
                        "document",
                        "JSON key exceeds byte budget",
                    ));
                }
                let key = k.extract::<&str>()?;
                if key.len() > self.max_bytes {
                    return Err(resource(
                        self.op,
                        "document",
                        "JSON key exceeds byte budget",
                    ));
                }
                self.bytes = self
                    .bytes
                    .checked_add(string_size(key))
                    .filter(|n| *n <= self.max_bytes)
                    .ok_or_else(|| resource(self.op, "document", "JSON byte budget exceeded"))?;
                self.walk(&v, depth + 1)?;
            }
        } else {
            return Err(input(
                self.op,
                "document",
                "expected builtin JSON dict/list/string/Boolean/integer/null",
            ));
        }
        self.bytes = self
            .bytes
            .checked_add(add)
            .filter(|n| *n <= self.max_bytes)
            .ok_or_else(|| resource(self.op, "document", "JSON byte budget exceeded"))?;
        Ok(())
    }
}
/// Accept a JSON text (preserving duplicate-key errors) or bounded builtin
/// objects. Prewalk limits depth, cardinality and encoding size before dumps.
pub(super) fn document(
    py: Python<'_>,
    v: &Bound<'_, PyAny>,
    max_bytes: usize,
    max_nodes: usize,
    max_depth: usize,
    op: &'static str,
) -> PyResult<String> {
    if let Ok(s) = v.cast::<PyString>() {
        if s.as_any().len()? > max_bytes {
            return Err(resource(op, "document", "JSON text exceeds input limit"));
        }
        let s = s.to_str()?;
        if s.len() > max_bytes {
            return Err(super::file_error(
                OperationError::new(
                    ErrorCode::ResourceLimit,
                    "JSON exceeds operation input limit",
                )
                .operation(op)
                .parameter("document"),
            ));
        }
        return Ok(s.to_owned());
    }
    JsonBudget {
        bytes: 0,
        nodes: 0,
        max_bytes,
        max_nodes,
        max_depth,
        op,
    }
    .walk(v, 0)?;
    let options = PyDict::new(py);
    options.set_item("ensure_ascii", false)?;
    options.set_item("allow_nan", false)?;
    options.set_item("separators", (",", ":"))?;
    let s: String = py
        .import("json")?
        .call_method("dumps", (v,), Some(&options))?
        .extract()?;
    if s.len() > max_bytes {
        return Err(resource(op, "document", "JSON exceeds input limit"));
    }
    Ok(s)
}
