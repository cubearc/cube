//! Python boundary only: all algorithms and input validation live in cube-core.

use cube_core::{binary, ciphers, encoding, forensics, hashing, operations as catalog};
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod alphabet;
mod audio;
mod base58check;
mod binaryops;
mod blockchain;
mod candidates;
mod combinations;
mod constants;
mod digital;
mod image;
mod investigation;
mod modular;
mod pipeline;
mod puzzles;
mod recipe;
mod textops;
mod traversal;
mod unicode;
mod values;
mod video;
mod web;
mod word;

fn invalid_input(error: cube_core::Error) -> PyErr {
    Python::attach(|py| {
        let exception = PyValueError::new_err(error.to_string());
        let code = error.code().as_str();
        for (name, value) in [("code", code), ("operation", error.operation_id())] {
            if let Err(error) = exception.value(py).setattr(name, value) {
                return error;
            }
        }
        for name in ["parameter", "offset"] {
            if let Err(error) = exception.value(py).setattr(name, py.None()) {
                return error;
            }
        }
        exception
    })
}

fn file_error(error: cube_core::errors::OperationError) -> PyErr {
    Python::attach(|py| {
        let exception = if error.code == cube_core::errors::ErrorCode::Io {
            PyOSError::new_err(error.to_string())
        } else {
            PyValueError::new_err(error.to_string())
        };
        let value = exception.value(py);
        if let Err(error) = (|| -> PyResult<()> {
            value.setattr("code", error.code.as_str())?;
            value.setattr("operation", error.operation)?;
            value.setattr("parameter", error.parameter)?;
            value.setattr("offset", error.offset)?;
            Ok(())
        })() {
            return error;
        }
        exception
    })
}

/// Identify signature candidates from exact header bytes; no format validation.
#[pyfunction]
fn file_signatures(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    structured(py, &serde_json::json!(forensics::signatures::detect(data)))
}

/// Inspect a regular file without EXIF disclosure; symlinks/devices are rejected.
#[pyfunction]
fn inspect_file(py: Python<'_>, path: std::path::PathBuf) -> PyResult<Py<PyAny>> {
    let info = forensics::inspect::file_info(&path).map_err(file_error)?;
    structured(py, &serde_json::json!(info))
}

/// Scan bounded embedded signature candidates, retaining offsets and completeness.
#[pyfunction]
#[pyo3(signature = (path, *, offset=0, length=None, format=None, max_results=1000))]
fn scan_file(
    py: Python<'_>,
    path: std::path::PathBuf,
    offset: u64,
    length: Option<u64>,
    format: Option<&str>,
    max_results: usize,
) -> PyResult<Py<PyAny>> {
    let mut file = forensics::open_regular(&path).map_err(file_error)?;
    let result = forensics::signatures::scan(
        &mut file,
        forensics::ByteRange { offset, length },
        format,
        max_results,
    )
    .map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Return exactly one selected range as bytes, subject to the 16 MiB allocation cap.
#[pyfunction]
fn extract_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    offset: u64,
    length: u64,
) -> PyResult<Bound<'py, PyBytes>> {
    let result = forensics::read_range(
        &mut std::io::Cursor::new(data),
        forensics::ByteRange {
            offset,
            length: Some(length),
        },
    )
    .map_err(|e| file_error(e.operation("file.range.extract")))?;
    Ok(PyBytes::new(py, &result))
}

/// Stream selected bytes to a new file; existing files and all aliases are rejected.
#[pyfunction]
fn extract_file_range(
    source: std::path::PathBuf,
    destination: std::path::PathBuf,
    offset: u64,
    length: u64,
) -> PyResult<u64> {
    forensics::extract_range(&source, &destination, offset, length).map_err(file_error)
}

/// Return bounded hexadecimal rows with original bytes, offsets and ASCII previews.
#[pyfunction]
#[pyo3(signature = (path, *, offset=0, length=None, width=16))]
fn file_hex(
    py: Python<'_>,
    path: std::path::PathBuf,
    offset: u64,
    length: Option<u64>,
    width: usize,
) -> PyResult<Py<PyAny>> {
    let mut file = forensics::open_regular(&path).map_err(file_error)?;
    let length = Some(match length {
        Some(length) => length,
        None => file
            .metadata()
            .map_err(cube_core::errors::OperationError::io)
            .map_err(file_error)?
            .len()
            .saturating_sub(offset)
            .min(256),
    });
    let rows =
        forensics::ranges::hex_rows(&mut file, forensics::ByteRange { offset, length }, width)
            .map_err(file_error)?;
    let result = structured(py, &serde_json::json!(rows))?;
    for (item, row) in result.bind(py).try_iter()?.zip(&rows) {
        item?.set_item("bytes", PyBytes::new(py, &row.bytes))?;
    }
    Ok(result)
}

/// Extract ASCII runs; incomplete budget-limited results are flagged explicitly.
#[pyfunction]
#[pyo3(signature = (path, *, offset=0, length=None, min_length=4, max_results=1000))]
fn file_strings(
    py: Python<'_>,
    path: std::path::PathBuf,
    offset: u64,
    length: Option<u64>,
    min_length: usize,
    max_results: usize,
) -> PyResult<Py<PyAny>> {
    let mut file = forensics::open_regular(&path).map_err(file_error)?;
    let result = forensics::ranges::strings(
        &mut file,
        forensics::ByteRange { offset, length },
        min_length,
        max_results,
    )
    .map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Compare exact byte offsets; missing tail bytes are None rather than zero.
#[pyfunction]
#[pyo3(signature = (first, second, *, offset=0, length=None, max_results=1000))]
fn compare_files(
    py: Python<'_>,
    first: std::path::PathBuf,
    second: std::path::PathBuf,
    offset: u64,
    length: Option<u64>,
    max_results: usize,
) -> PyResult<Py<PyAny>> {
    let mut first = forensics::open_regular(&first).map_err(file_error)?;
    let mut second = forensics::open_regular(&second).map_err(file_error)?;
    let result = forensics::ranges::compare(
        &mut first,
        &mut second,
        forensics::ByteRange { offset, length },
        max_results,
    )
    .map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Explicit EXIF metadata from JPEG/TIFF bytes; no pixel decoding or network access.
#[pyfunction]
fn inspect_exif(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let result = forensics::exif::inspect(data).map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Explicit bounded EXIF inspection of a regular JPEG/TIFF file.
#[pyfunction]
fn file_exif(py: Python<'_>, path: std::path::PathBuf) -> PyResult<Py<PyAny>> {
    let result = forensics::exif::read(&mut forensics::open_regular(&path).map_err(file_error)?)
        .map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Detect trailing bytes after supported PNG/JPEG/GIF/ZIP framing; no pixel decoding.
#[pyfunction]
fn appended_data(py: Python<'_>, path: std::path::PathBuf) -> PyResult<Py<PyAny>> {
    let result =
        forensics::boundaries::inspect(&mut forensics::open_regular(&path).map_err(file_error)?)
            .map_err(file_error)?;
    structured(py, &serde_json::json!(result))
}

/// Validate one explicit ZIP origin; payloads are never decompressed or executed.
#[pyfunction]
#[pyo3(signature=(data, *, offset=0))]
fn validate_zip(py: Python<'_>, data: &[u8], offset: u64) -> PyResult<Py<PyAny>> {
    let report =
        forensics::zip::validate(&mut std::io::Cursor::new(data), offset).map_err(file_error)?;
    structured(py, &serde_json::json!(report))
}
#[pyfunction]
#[pyo3(signature=(path, *, offset=0))]
fn validate_zip_file(py: Python<'_>, path: std::path::PathBuf, offset: u64) -> PyResult<Py<PyAny>> {
    let report = forensics::zip::validate(
        &mut forensics::open_regular(&path).map_err(file_error)?,
        offset,
    )
    .map_err(file_error)?;
    structured(py, &serde_json::json!(report))
}

struct PythonWriter<'py>(Bound<'py, PyAny>);

/// Recover a selected range or revalidated candidate with explicit local output.
#[pyfunction]
#[pyo3(signature=(source, destination, offset, *, length=None, max_bytes=forensics::carving::DEFAULT_MAX_BYTES))]
fn carve_file(
    py: Python<'_>,
    source: std::path::PathBuf,
    destination: std::path::PathBuf,
    offset: u64,
    length: Option<u64>,
    max_bytes: u64,
) -> PyResult<Py<PyAny>> {
    let selection = length.map_or(
        forensics::carving::Selection::Candidate { offset },
        |length| forensics::carving::Selection::Range { offset, length },
    );
    let report = forensics::carving::carve(&source, &destination, selection, max_bytes)
        .map_err(file_error)?;
    structured(py, &serde_json::json!(report))
}
impl std::io::Write for PythonWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let result = self
            .0
            .call_method1("write", (PyBytes::new(self.0.py(), bytes),))
            .and_then(|r| r.extract::<usize>())
            .map_err(std::io::Error::other)?;
        if result > bytes.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "writer returned an invalid byte count",
            ));
        }
        Ok(result)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0
            .call_method0("flush")
            .map(|_| ())
            .map_err(std::io::Error::other)
    }
}

/// Stream up to 1 GiB to a caller-selected binary writer implementing write/flush.
#[pyfunction]
fn copy_file_range(
    source: std::path::PathBuf,
    writer: Bound<'_, PyAny>,
    offset: u64,
    length: u64,
) -> PyResult<u64> {
    let mut file = forensics::open_regular(&source).map_err(file_error)?;
    forensics::copy_range(&mut file, &mut PythonWriter(writer), offset, length).map_err(file_error)
}

// Only structured metadata crosses this JSON boundary. Algorithm byte results
// continue to use PyBytes directly; no Python algorithm implementation exists.
fn structured(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    Ok(py
        .import("json")?
        .call_method1("loads", (value.to_string(),))?
        .unbind())
}

/// List descriptors with pipeline availability, sorted by ID; optionally filter a category.
#[pyfunction]
#[pyo3(signature = (category=None))]
fn operations(py: Python<'_>, category: Option<&str>) -> PyResult<Py<PyAny>> {
    let mut descriptors = catalog::filtered(category).map_err(file_error)?;
    descriptors.sort_by_key(|d| d.id);
    structured(
        py,
        &serde_json::json!(
            descriptors
                .iter()
                .map(|d| d.discovery())
                .collect::<Vec<_>>()
        ),
    )
}

/// Describe an exact stable ID with pipeline availability. Unknown IDs raise ValueError.
#[pyfunction]
fn describe_operation(py: Python<'_>, id: &str) -> PyResult<Py<PyAny>> {
    let descriptor = catalog::require(id).map_err(file_error)?;
    structured(py, &serde_json::json!(descriptor.discovery()))
}

/// Encode bytes as padded standard Base32 using uppercase letters.
#[pyfunction]
fn base32_encode(data: &[u8]) -> String {
    encoding::base32_encode(data)
}

/// Decode strict uppercase Base32 with canonical padding. Invalid input raises ValueError.
#[pyfunction]
fn base32_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base32_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode bytes as Bitcoin Base58, preserving leading zero bytes as 1.
#[pyfunction]
fn base58_encode(data: &[u8]) -> String {
    encoding::base58_encode(data)
}

/// Decode strict Bitcoin Base58 to bytes. Invalid input raises ValueError.
#[pyfunction]
fn base58_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base58_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode bytes as padded standard Base64.
#[pyfunction]
fn base64_encode(data: &[u8]) -> String {
    encoding::base64_encode(data)
}

/// Decode strict, padded standard Base64. Invalid input raises ValueError.
#[pyfunction]
fn base64_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::base64_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode bytes as lowercase hexadecimal digits.
#[pyfunction]
fn hex_encode(data: &[u8]) -> String {
    encoding::hex_encode(data)
}

/// Decode hexadecimal digits in either case. Invalid input raises ValueError.
#[pyfunction]
fn hex_decode<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = encoding::hex_decode(text).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Encode text by an ASCII Caesar shift from -2147483648 to 2147483647.
///
/// Shifts are normalized modulo 26; case and non-ASCII characters are preserved
/// without normalization. This is an educational cipher, not secure encryption.
/// Out-of-range integers raise OverflowError; incorrect types raise TypeError.
#[pyfunction]
fn caesar_encode(text: &str, shift: i32) -> String {
    ciphers::caesar_encode(text, shift)
}

/// Decode text with the same signed 32-bit shift used for Caesar encoding.
///
/// Only ASCII letters rotate. All other characters are preserved unchanged.
/// The inverse is safe for every shift, including -2147483648.
/// Out-of-range integers raise OverflowError; incorrect types raise TypeError.
#[pyfunction]
fn caesar_decode(text: &str, shift: i32) -> String {
    ciphers::caesar_decode(text, shift)
}

/// Rotate ASCII letters by 13, preserving all other characters.
#[pyfunction]
fn rot13(text: &str) -> String {
    ciphers::rot13(text)
}

/// Extract scalars using caller-supplied source and comma-separated book coordinates.
#[pyfunction]
#[pyo3(signature=(source,coordinates,*,scheme="word_character",index_base=1))]
fn book_decode(source: &str, coordinates: &str, scheme: &str, index_base: u8) -> PyResult<String> {
    ciphers::book::decode(
        source,
        coordinates,
        ciphers::book::Scheme::parse(scheme).map_err(file_error)?,
        index_base,
    )
    .map_err(file_error)
}

/// Compute CRC-32/ISO-HDLC (IEEE) as eight lowercase hexadecimal digits.
///
/// Accepts arbitrary bytes, preserves leading zeroes, and returns 00000000 for
/// empty input. This is a checksum, not a cryptographic hash.
#[pyfunction]
fn crc32(data: &[u8]) -> String {
    hashing::crc32(data)
}

/// Compute RIPEMD-160 as 40 lowercase hexadecimal digits; input <=16 MiB.
#[pyfunction]
fn keccak256(data: &[u8]) -> PyResult<String> {
    hashing::keccak256(data).map_err(file_error)
}
#[pyfunction]
fn ripemd160(data: &[u8]) -> PyResult<String> {
    hashing::ripemd160(data).map_err(file_error)
}

/// Compute SHA-256 as 64 lowercase hexadecimal digits.
#[pyfunction]
fn sha256(data: &[u8]) -> String {
    hashing::sha256(data)
}

/// Exact SHA-512 digest of bytes, 128 lowercase hex digits.
#[pyfunction]
fn sha512(data: &[u8]) -> String {
    hashing::sha512(data)
}
/// Legacy SHA-1; unsuitable for collision-resistant security.
#[pyfunction]
fn sha1(data: &[u8]) -> String {
    hashing::sha1(data)
}
/// Legacy MD5; unsuitable for collision-resistant security.
#[pyfunction]
fn md5(data: &[u8]) -> String {
    hashing::md5(data)
}
/// CRC-32/ISCSI (CRC32C), eight hex digits; not cryptographic.
#[pyfunction]
fn crc32c(data: &[u8]) -> String {
    hashing::crc32c(data)
}
/// CRC-16/IBM-3740, four hex digits; not cryptographic.
#[pyfunction]
fn crc16_ibm3740(data: &[u8]) -> String {
    hashing::crc16_ibm3740(data)
}

/// XOR bytes with a repeating key. An empty key raises ValueError.
#[pyfunction]
fn xor<'py>(py: Python<'py>, data: &[u8], key: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = binary::xor(data, key).map_err(invalid_input)?;
    Ok(PyBytes::new(py, &bytes))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    pipeline::register(module)?;
    investigation::register(module)?;
    recipe::register(module)?;
    audio::register(module)?;
    video::register(module)?;
    web::register(module)?;
    binaryops::register(module)?;
    textops::register(module)?;
    unicode::register(module)?;
    values::register(module)?;
    word::register(module)?;
    traversal::register(module)?;
    puzzles::register(module)?;
    combinations::register(module)?;
    constants::register(module)?;
    alphabet::register(module)?;
    modular::register(module)?;
    digital::register(module)?;
    base58check::register(module)?;
    blockchain::register(module)?;
    candidates::register(module)?;
    module.add_class::<image::PyImage>()?;
    module.add_function(wrap_pyfunction!(image::image_pairs, module)?)?;
    module.add_function(wrap_pyfunction!(image::png_chunks, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_exif, module)?)?;
    module.add_function(wrap_pyfunction!(file_exif, module)?)?;
    module.add_function(wrap_pyfunction!(appended_data, module)?)?;
    module.add_function(wrap_pyfunction!(file_hex, module)?)?;
    module.add_function(wrap_pyfunction!(file_strings, module)?)?;
    module.add_function(wrap_pyfunction!(compare_files, module)?)?;
    module.add_function(wrap_pyfunction!(file_signatures, module)?)?;
    module.add_function(wrap_pyfunction!(inspect_file, module)?)?;
    module.add_function(wrap_pyfunction!(scan_file, module)?)?;
    module.add_function(wrap_pyfunction!(extract_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(extract_file_range, module)?)?;
    module.add_function(wrap_pyfunction!(copy_file_range, module)?)?;
    module.add_function(wrap_pyfunction!(operations, module)?)?;
    module.add_function(wrap_pyfunction!(describe_operation, module)?)?;
    module.add_function(wrap_pyfunction!(validate_zip, module)?)?;
    module.add_function(wrap_pyfunction!(validate_zip_file, module)?)?;
    module.add_function(wrap_pyfunction!(carve_file, module)?)?;
    module.add_function(wrap_pyfunction!(base32_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base32_decode, module)?)?;
    module.add_function(wrap_pyfunction!(base58_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base58_decode, module)?)?;
    module.add_function(wrap_pyfunction!(base64_encode, module)?)?;
    module.add_function(wrap_pyfunction!(base64_decode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_encode, module)?)?;
    module.add_function(wrap_pyfunction!(hex_decode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_encode, module)?)?;
    module.add_function(wrap_pyfunction!(caesar_decode, module)?)?;
    module.add_function(wrap_pyfunction!(rot13, module)?)?;
    module.add_function(wrap_pyfunction!(book_decode, module)?)?;
    module.add_function(wrap_pyfunction!(crc32, module)?)?;
    module.add_function(wrap_pyfunction!(sha256, module)?)?;
    module.add_function(wrap_pyfunction!(ripemd160, module)?)?;
    module.add_function(wrap_pyfunction!(keccak256, module)?)?;
    module.add_function(wrap_pyfunction!(sha512, module)?)?;
    module.add_function(wrap_pyfunction!(sha1, module)?)?;
    module.add_function(wrap_pyfunction!(md5, module)?)?;
    module.add_function(wrap_pyfunction!(crc32c, module)?)?;
    module.add_function(wrap_pyfunction!(crc16_ibm3740, module)?)?;
    module.add_function(wrap_pyfunction!(xor, module)?)?;
    Ok(())
}
