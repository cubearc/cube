//! Thin native interfaces for the bounded codecs and file-report API.
use super::binary_formats::{count, error, report, result, settings};
use cube_core::{
    binary::compression::{
        bzip2, dictionary, huffman, lz4, lzw, packbits, probability, transforms,
    },
    errors::ErrorCode,
    forensics::formats::{bmp, executable, mpeg, ogg, tiff, webp},
    operations::bounded_formats as spec,
};
use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict, PyList},
};
use serde::de::DeserializeOwned;

fn list<T: DeserializeOwned>(
    values: &Bound<'_, PyList>,
    max: usize,
    op: &'static str,
) -> PyResult<Vec<T>> {
    count(values.len(), max, op)?;
    let text =
        super::discrete_transport::document(values.py(), values.as_any(), 4 << 20, 400_000, 8, op)?;
    serde_json::from_str(&text).map_err(|_| {
        error(
            ErrorCode::InvalidInput,
            op,
            "records do not match the selected format",
        )
    })
}

fn frequencies(values: &Bound<'_, PyList>, op: &'static str) -> PyResult<[u64; 256]> {
    result(spec::frequencies(list(values, 256, op)?), op)
}

macro_rules! plain_report {
    ($name:ident, $op:literal, $operation:path) => {
        #[pyfunction]
        fn $name(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
            report(py, result($operation(data), $op)?, $op)
        }
    };
}

macro_rules! options_bytes {
    ($name:ident, $op:literal, $operation:path) => {
        #[pyfunction]
        fn $name<'py>(
            py: Python<'py>,
            data: &[u8],
            options: &Bound<'_, PyDict>,
        ) -> PyResult<Bound<'py, PyBytes>> {
            let bytes = result($operation(data, settings(options, $op)?), $op)?;
            Ok(PyBytes::new(py, &bytes))
        }
    };
}

#[pyfunction]
fn packbits_encode<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(packbits::encode(data), spec::PACKBITS_ENCODE)?,
    ))
}

#[pyfunction]
#[pyo3(signature=(data, *, expected_length=None))]
fn packbits_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    expected_length: Option<usize>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(
            packbits::decode(data, expected_length),
            spec::PACKBITS_DECODE,
        )?,
    ))
}

#[pyfunction]
#[pyo3(signature=(data, *, window=4096))]
fn lz77_encode(py: Python<'_>, data: &[u8], window: u16) -> PyResult<Py<PyAny>> {
    const ID: &str = "binary.lz77.encode";
    report(py, result(dictionary::lz77_encode(data, window), ID)?, ID)
}

#[pyfunction]
#[pyo3(signature=(tokens, *, window=4096))]
fn lz77_decode<'py>(
    py: Python<'py>,
    tokens: &Bound<'_, PyList>,
    window: u16,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.lz77.decode";
    let values = list(tokens, dictionary::MAX_TOKENS, ID)?;
    Ok(PyBytes::new(
        py,
        &result(dictionary::lz77_decode(&values, window), ID)?,
    ))
}

plain_report!(lz78_encode, "binary.lz78.encode", dictionary::lz78_encode);
#[pyfunction]
fn lz78_decode<'py>(py: Python<'py>, tokens: &Bound<'_, PyList>) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.lz78.decode";
    let values = list(tokens, dictionary::MAX_TOKENS, ID)?;
    Ok(PyBytes::new(
        py,
        &result(dictionary::lz78_decode(&values), ID)?,
    ))
}

#[pyfunction]
fn bwt_encode<'py>(py: Python<'py>, data: &[u8]) -> PyResult<(Bound<'py, PyBytes>, usize)> {
    let (bytes, primary) = result(transforms::bwt(data), "binary.bwt.encode")?;
    Ok((PyBytes::new(py, &bytes), primary))
}

#[pyfunction]
fn bwt_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    primary_index: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = result(
        transforms::inverse_bwt(data, primary_index),
        "binary.bwt.decode",
    )?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
#[pyo3(signature=(data, *, alphabet=None))]
fn mtf_encode<'py>(
    py: Python<'py>,
    data: &[u8],
    alphabet: Option<&[u8]>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(transforms::mtf(data, alphabet, false), "binary.mtf.encode")?,
    ))
}

#[pyfunction]
#[pyo3(signature=(data, *, alphabet=None))]
fn mtf_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    alphabet: Option<&[u8]>,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(transforms::mtf(data, alphabet, true), "binary.mtf.decode")?,
    ))
}

#[pyfunction]
fn lz4_encode<'py>(
    py: Python<'py>,
    data: &[u8],
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.lz4.encode";
    let options: spec::Lz4Encode = settings(options, ID)?;
    Ok(PyBytes::new(
        py,
        &result(lz4::encode(data, options.mode, options.limits), ID)?,
    ))
}

#[pyfunction]
fn lz4_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.lz4.decode";
    let (mode, limits) = settings::<spec::Lz4Decode>(options, ID)?.parts();
    Ok(PyBytes::new(
        py,
        &result(lz4::decode(data, mode, limits), ID)?,
    ))
}

#[pyfunction]
#[pyo3(signature=(data, *, level=1))]
fn bzip2_encode<'py>(py: Python<'py>, data: &[u8], level: u8) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(bzip2::encode(data, level), "binary.bzip2.encode")?,
    ))
}

#[pyfunction]
#[pyo3(signature=(data, *, concatenate=false))]
fn bzip2_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    concatenate: bool,
) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(
        py,
        &result(bzip2::decode(data, concatenate), "binary.bzip2.decode")?,
    ))
}

options_bytes!(lzw_encode, "binary.lzw.encode", lzw::encode);
options_bytes!(lzw_decode, "binary.lzw.decode", lzw::decode);

#[pyfunction]
fn huffman_construct(py: Python<'_>, table: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    const ID: &str = "binary.huffman.construct";
    report(
        py,
        result(huffman::construct(&frequencies(table, ID)?), ID)?,
        ID,
    )
}

#[pyfunction]
fn huffman_encode<'py>(
    py: Python<'py>,
    data: &[u8],
    table: &Bound<'_, PyList>,
) -> PyResult<(Bound<'py, PyBytes>, usize, Py<PyAny>)> {
    const ID: &str = "binary.huffman.encode";
    let encoded = result(huffman::encode(data, &frequencies(table, ID)?), ID)?;
    Ok((
        PyBytes::new(py, &encoded.data),
        encoded.bit_count,
        report(py, encoded.codes, ID)?,
    ))
}

#[pyfunction]
fn huffman_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    lengths: &Bound<'_, PyList>,
    bit_count: usize,
    symbol_count: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.huffman.decode";
    let model = list(lengths, 256, ID)?;
    Ok(PyBytes::new(
        py,
        &result(huffman::decode(data, &model, bit_count, symbol_count), ID)?,
    ))
}

#[pyfunction]
fn entropy_encode<'py>(
    py: Python<'py>,
    data: &[u8],
    table: &Bound<'_, PyList>,
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.entropy.encode";
    let opts: spec::EntropyMode = settings(options, ID)?;
    Ok(PyBytes::new(
        py,
        &result(
            probability::encode(data, &frequencies(table, ID)?, opts.profile),
            ID,
        )?,
    ))
}

#[pyfunction]
fn entropy_decode<'py>(
    py: Python<'py>,
    data: &[u8],
    table: &Bound<'_, PyList>,
    options: &Bound<'_, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    const ID: &str = "binary.entropy.decode";
    let opts: spec::EntropyDecode = settings(options, ID)?;
    Ok(PyBytes::new(
        py,
        &result(
            probability::decode(
                data,
                &frequencies(table, ID)?,
                opts.symbol_count,
                opts.profile,
            ),
            ID,
        )?,
    ))
}

plain_report!(inspect_bmp, "file.bmp.inspect", bmp::inspect);
plain_report!(inspect_webp, "file.webp.inspect", webp::inspect);
plain_report!(inspect_tiff, "file.tiff.inspect", tiff::inspect);
plain_report!(
    inspect_executable,
    "file.executable.inspect",
    executable::inspect
);
plain_report!(inspect_mpeg_audio, "file.mpeg_audio.inspect", mpeg::inspect);
plain_report!(inspect_ogg, "file.ogg.inspect", ogg::inspect);

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(bwt_decode, module)?)?;
    module.add_function(wrap_pyfunction!(bwt_encode, module)?)?;
    module.add_function(wrap_pyfunction!(bzip2_decode, module)?)?;
    module.add_function(wrap_pyfunction!(bzip2_encode, module)?)?;
    module.add_function(wrap_pyfunction!(entropy_decode, module)?)?;
    module.add_function(wrap_pyfunction!(entropy_encode, module)?)?;
    module.add_function(wrap_pyfunction!(huffman_construct, module)?)?;
    module.add_function(wrap_pyfunction!(huffman_decode, module)?)?;
    module.add_function(wrap_pyfunction!(huffman_encode, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_bmp, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_executable, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_mpeg_audio, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_ogg, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_tiff, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_webp, module)?)?;
    module.add_function(wrap_pyfunction!(lz4_decode, module)?)?;
    module.add_function(wrap_pyfunction!(lz4_encode, module)?)?;
    module.add_function(wrap_pyfunction!(lz77_decode, module)?)?;
    module.add_function(wrap_pyfunction!(lz77_encode, module)?)?;
    module.add_function(wrap_pyfunction!(lz78_decode, module)?)?;
    module.add_function(wrap_pyfunction!(lz78_encode, module)?)?;
    module.add_function(wrap_pyfunction!(lzw_decode, module)?)?;
    module.add_function(wrap_pyfunction!(lzw_encode, module)?)?;
    module.add_function(wrap_pyfunction!(mtf_decode, module)?)?;
    module.add_function(wrap_pyfunction!(mtf_encode, module)?)?;
    module.add_function(wrap_pyfunction!(packbits_decode, module)?)?;
    module.add_function(wrap_pyfunction!(packbits_encode, module)?)?;
    Ok(())
}
