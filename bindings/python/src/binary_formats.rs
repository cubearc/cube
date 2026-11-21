//! Native byte and exact-integer adapters for the binary-format contracts.
use cube_core::{
    binary::{bcd, bit_units, reed_solomon, representation, varint},
    encoding::rfc4648,
    errors::{ErrorCode, OperationError},
    hashing::fingerprints,
};
use pyo3::{
    prelude::*,
    types::{PyBool, PyBytes, PyDict, PyInt, PyList, PyTuple},
};
use serde::{Serialize, de::DeserializeOwned};

pub(super) fn error(code: ErrorCode, op: &'static str, message: &str) -> PyErr {
    super::file_error(OperationError::new(code, message).operation(op))
}
pub(super) fn result<T>(value: Result<T, OperationError>, op: &'static str) -> PyResult<T> {
    value.map_err(|err| super::file_error(err.operation(op)))
}
pub(super) fn settings<T: DeserializeOwned>(
    options: &Bound<'_, PyDict>,
    op: &'static str,
) -> PyResult<T> {
    let json =
        super::discrete_transport::document(options.py(), options.as_any(), 4096, 64, 6, op)?;
    serde_json::from_str(&json).map_err(|_| {
        error(
            ErrorCode::InvalidParameter,
            op,
            "parameters do not match the selected binary format",
        )
    })
}
pub(super) fn report(
    py: Python<'_>,
    value: impl Serialize,
    op: &'static str,
) -> PyResult<Py<PyAny>> {
    let value = serde_json::to_value(value).map_err(|_| {
        error(
            ErrorCode::InvalidInput,
            op,
            "could not serialize the native report",
        )
    })?;
    super::structured(py, &value)
}
pub(super) fn count(value: usize, maximum: usize, op: &'static str) -> PyResult<()> {
    if value > maximum {
        return Err(error(
            ErrorCode::ResourceLimit,
            op,
            "too many values for the selected binary format",
        ));
    }
    Ok(())
}
fn integers(
    values: &Bound<'_, PyList>,
    op: &'static str,
) -> PyResult<Vec<representation::Integer>> {
    count(values.len(), 65_536, op)?;
    values
        .iter()
        .map(|v| {
            if v.is_exact_instance_of::<PyInt>() {
                if let Ok(value) = v.extract::<i64>() {
                    return Ok(representation::Integer::Signed(value));
                }
                if let Ok(value) = v.extract::<u64>() {
                    return Ok(representation::Integer::Unsigned(value));
                }
            }
            Err(error(
                ErrorCode::InvalidInput,
                op,
                "expected an exact builtin 64-bit integer",
            ))
        })
        .collect()
}

macro_rules! with_bytes {
    ($name:ident, $op:literal, $run:path) => {
        #[pyfunction]
        fn $name<'py>(
            py: Python<'py>,
            data: &[u8],
            options: &Bound<'_, PyDict>,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let data = result($run(data, settings(options, $op)?), $op)?;
            Ok(PyBytes::new(py, &data))
        }
    };
}
macro_rules! with_report {
    ($name:ident, $op:literal, $run:path) => {
        #[pyfunction]
        fn $name(py: Python<'_>, data: &[u8], options: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
            report(py, result($run(data, settings(options, $op)?), $op)?, $op)
        }
    };
}
macro_rules! with_bits {
    ($name:ident, $op:literal, $run:path) => {
        #[pyfunction]
        fn $name<'py>(
            py: Python<'py>,
            data: &[u8],
            options: &Bound<'_, PyDict>,
        ) -> PyResult<(Bound<'py, PyBytes>, usize)> {
            let (data, bits) = result($run(data, settings(options, $op)?), $op)?;
            Ok((PyBytes::new(py, &data), bits))
        }
    };
}
macro_rules! with_ints {
    ($name:ident, $op:literal, $run:path) => {
        #[pyfunction]
        fn $name<'py>(
            py: Python<'py>,
            values: &Bound<'_, PyList>,
            options: &Bound<'_, PyDict>,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let data = result($run(&integers(values, $op)?, settings(options, $op)?), $op)?;
            Ok(PyBytes::new(py, &data))
        }
    };
}

with_bits!(pack_bits, "binary.pack_bits", bit_units::pack);
with_bits!(unpack_bits, "binary.unpack_bits", bit_units::unpack);
with_bytes!(mask_bytes, "binary.mask", bit_units::mask);
with_report!(count_bits, "binary.count_bits", bit_units::count);
with_ints!(
    fixed_integer_encode,
    "binary.fixed_integer.encode",
    representation::encode
);
with_report!(
    fixed_integer_decode,
    "binary.fixed_integer.decode",
    representation::decode
);
with_report!(bcd_decode, "binary.bcd.decode", bcd::decode);
with_ints!(varint_encode, "binary.varint.encode", varint::encode);
with_report!(varint_decode, "binary.varint.decode", varint::decode);
with_report!(
    ieee754_inspect,
    "binary.ieee754.inspect",
    representation::inspect_floats
);

#[pyfunction]
fn bcd_encode<'py>(
    py: Python<'py>,
    digits: &str,
    negative: &Bound<'_, PyBool>,
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.bcd.encode";
    let bytes = result(
        bcd::encode(digits, negative.is_true(), settings(options, ID)?),
        ID,
    )?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
fn noncrypto_hash(data: &[u8], options: &Bound<'_, PyDict>) -> PyResult<String> {
    const ID: &str = "hash.noncrypto";
    Ok(result(fingerprints::noncrypto(data, settings(options, ID)?), ID)?.hex)
}

#[pyfunction]
fn siphash24(data: &[u8], key: &[u8]) -> PyResult<String> {
    Ok(result(fingerprints::siphash24(data, key), "hash.siphash24")?.hex)
}

#[pyfunction]
fn base32hex_encode(data: &[u8]) -> PyResult<String> {
    result(rfc4648::encode_base32hex(data), "encoding.base32hex.encode")
}
#[pyfunction]
fn base32hex_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let data = result(rfc4648::decode_base32hex(text), "encoding.base32hex.decode")?;
    Ok(PyBytes::new(py, &data))
}

#[pyfunction(signature = (data, options=None))]
fn base64url_encode(data: &[u8], options: Option<&Bound<'_, PyDict>>) -> PyResult<String> {
    const ID: &str = "encoding.base64url.encode";
    let options = options
        .map(|options| settings(options, ID))
        .transpose()?
        .unwrap_or_default();
    result(rfc4648::encode_base64url(data, options), ID)
}
#[pyfunction(signature = (text, options=None))]
fn base64url_decode<'py>(
    py: Python<'py>,
    text: &str,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "encoding.base64url.decode";
    let options = options
        .map(|options| settings(options, ID))
        .transpose()?
        .unwrap_or_default();
    let data = result(rfc4648::decode_base64url(text, options), ID)?;
    Ok(PyBytes::new(py, &data))
}

#[pyfunction]
fn reed_solomon_encode<'py>(
    py: Python<'py>,
    blocks: &Bound<'_, PyList>,
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyList>> {
    const ID: &str = "binary.reed_solomon.encode";
    count(blocks.len(), 4096, ID)?;
    let blocks = blocks
        .iter()
        .map(Bound::cast_into::<PyBytes>)
        .collect::<Result<Vec<_>, _>>()?;
    let slices: Vec<_> = blocks.iter().map(|block| block.as_bytes()).collect();
    let values = result(reed_solomon::encode(&slices, settings(options, ID)?), ID)?;
    let output = PyList::empty(py);
    for value in values {
        let row = PyDict::new(py);
        row.set_item("codeword", PyBytes::new(py, &value.codeword))?;
        row.set_item("parity", PyBytes::new(py, &value.parity))?;
        output.append(row)?;
    }
    Ok(output)
}

#[pyfunction]
fn reed_solomon_decode<'py>(
    py: Python<'py>,
    blocks: &Bound<'_, PyList>,
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyList>> {
    const ID: &str = "binary.reed_solomon.decode";
    count(blocks.len(), 4096, ID)?;
    let blocks = blocks
        .iter()
        .map(|block| {
            let block = block.cast::<PyTuple>()?;
            if block.len() != 2 {
                return Err(error(
                    ErrorCode::InvalidInput,
                    ID,
                    "expected a (codeword, erasures) pair",
                ));
            }
            let bytes = block.get_item(0)?.cast_into::<PyBytes>()?;
            let erasures = block.get_item(1)?.cast_into::<PyList>()?;
            count(erasures.len(), 64, ID)?;
            let erasures = erasures
                .iter()
                .map(|value| {
                    if value.is_exact_instance_of::<PyInt>()
                        && let Ok(position) = value.extract::<u8>()
                    {
                        return Ok(usize::from(position));
                    }
                    Err(error(
                        ErrorCode::InvalidInput,
                        ID,
                        "expected a byte index for each erasure",
                    ))
                })
                .collect::<PyResult<Vec<_>>>()?;
            Ok((bytes, erasures))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let blocks: Vec<_> = blocks
        .iter()
        .map(|(bytes, erasures)| reed_solomon::Damaged {
            codeword: bytes.as_bytes(),
            erasures,
        })
        .collect();
    let values = result(reed_solomon::decode(&blocks, settings(options, ID)?), ID)?;
    let output = PyList::empty(py);
    for value in values {
        let row = PyDict::new(py);
        row.set_item("codeword", PyBytes::new(py, &value.codeword))?;
        row.set_item("message", PyBytes::new(py, &value.message))?;
        row.set_item("positions", value.positions)?;
        row.set_item(
            "initial_syndromes",
            PyList::new(py, value.initial_syndromes)?,
        )?;
        row.set_item(
            "verified_syndromes",
            PyList::new(py, value.verified_syndromes)?,
        )?;
        output.append(row)?;
    }
    Ok(output)
}

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(pack_bits, module)?)?;
    module.add_function(wrap_pyfunction!(unpack_bits, module)?)?;
    module.add_function(wrap_pyfunction!(mask_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(count_bits, module)?)?;
    module.add_function(wrap_pyfunction!(fixed_integer_encode, module)?)?;
    module.add_function(wrap_pyfunction!(fixed_integer_decode, module)?)?;
    module.add_function(wrap_pyfunction!(bcd_encode, module)?)?;
    module.add_function(wrap_pyfunction!(bcd_decode, module)?)?;
    module.add_function(wrap_pyfunction!(varint_encode, module)?)?;
    module.add_function(wrap_pyfunction!(varint_decode, module)?)?;
    module.add_function(wrap_pyfunction!(ieee754_inspect, module)?)?;
    module.add_function(wrap_pyfunction!(reed_solomon_encode, module)?)?;
    module.add_function(wrap_pyfunction!(reed_solomon_decode, module)?)?;
    module.add_function(wrap_pyfunction!(noncrypto_hash, module)?)?;
    module.add_function(wrap_pyfunction!(siphash24, module)?)?;
    module.add_function(wrap_pyfunction!(base32hex_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base32hex_decode, module)?)?;
    module.add_function(wrap_pyfunction!(base64url_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base64url_decode, module)?)?;
    Ok(())
}
