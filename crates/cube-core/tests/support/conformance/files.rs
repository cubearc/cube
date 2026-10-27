//! Fixed-fixture driver for direct public Core APIs, compiled only by integration tests.
use super::field;
use cube_core::{
    errors::{ErrorCode, OperationError},
    forensics::{self, ByteRange, boundaries, exif, ranges, signatures, zip},
    web,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::io::Cursor;

enum Adapter {
    Appended,
    Compare,
    Exif,
    Hex,
    Extract,
    Signature,
    Scan,
    Strings,
    Zip,
    Carve,
    Info,
    Html,
    Har,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "file.appended.detect" => Adapter::Appended,
        "file.binary.compare" => Adapter::Compare,
        "file.exif.inspect" => Adapter::Exif,
        "file.hex.inspect" => Adapter::Hex,
        "file.range.extract" => Adapter::Extract,
        "file.signature.detect" => Adapter::Signature,
        "file.signature.scan" => Adapter::Scan,
        "file.strings.extract" => Adapter::Strings,
        "file.zip.validate" => Adapter::Zip,
        "file.carve" => Adapter::Carve,
        "file.info.inspect" => Adapter::Info,
        "web.html.inspect" => Adapter::Html,
        "web.har.inspect" => Adapter::Har,
        _ => return None,
    })
}
pub(super) fn supported(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact bytes and bounded file inspection parameters",
    )
}
fn number(p: &Value, key: &str) -> Result<u64, OperationError> {
    let value = &p[key];
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .ok_or_else(invalid)
}
fn selection(p: &Value) -> Result<ByteRange, OperationError> {
    Ok(ByteRange {
        offset: number(p, "offset")?,
        length: p.get("length").map(|_| number(p, "length")).transpose()?,
    })
}
fn output(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|v| json!({"output":v}))
        .map_err(|_| invalid())
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        if matches!(selected, Adapter::Info) {
            return Err(OperationError::new(
                ErrorCode::InvalidInput,
                "File inspection requires an authorized workspace file binding",
            ));
        }
        if matches!(selected, Adapter::Carve) {
            let selection = if p.get("length").is_some() {
                forensics::carving::Selection::Range {
                    offset: number(&p, "offset")?,
                    length: number(&p, "length")?,
                }
            } else {
                forensics::carving::Selection::Candidate {
                    offset: number(&p, "offset")?,
                }
            };
            let (report, data) = forensics::carving::carve_reader(
                &mut Cursor::new(super::bytes(&inputs["source"])?),
                selection,
                number(&p, "max_bytes")?,
            )?;
            return Ok(
                json!({"output":report,"data":{"base64":cube_core::encoding::base64_encode(&data)}}),
            );
        }
        if matches!(selected, Adapter::Html | Adapter::Har) {
            let text: String = field(&inputs, "input")?;
            return match selected {
                Adapter::Html => output(web::html::inspect(&text)?),
                _ => output(web::har::inspect(&text)?),
            };
        }
        if matches!(selected, Adapter::Compare) {
            return output(ranges::compare(
                &mut Cursor::new(super::bytes(&inputs["left"])?),
                &mut Cursor::new(super::bytes(&inputs["right"])?),
                selection(&p)?,
                field(&p, "max_results")?,
            )?);
        }
        let bytes = super::bytes(&inputs["input"])?;
        let mut reader = Cursor::new(&bytes);
        match selected {
            Adapter::Appended => output(boundaries::inspect(&mut reader)?),
            Adapter::Exif => output(exif::inspect(&bytes)?),
            Adapter::Signature => output(signatures::detect(&bytes)),
            Adapter::Hex => {
                let mut range = selection(&p)?;
                // Match Core's documented CLI default; an explicit length is exact.
                if range.length.is_none() {
                    range.length = Some((bytes.len() as u64).saturating_sub(range.offset).min(256));
                }
                output(ranges::hex_rows(&mut reader, range, field(&p, "width")?)?)
            }
            Adapter::Extract => {
                let data = forensics::read_range(&mut reader, selection(&p)?)?;
                output(json!({"base64":cube_core::encoding::base64_encode(&data)}))
            }
            Adapter::Scan => {
                let format = p
                    .get("format")
                    .map(|_| field::<String>(&p, "format"))
                    .transpose()?;
                output(signatures::scan(
                    &mut reader,
                    selection(&p)?,
                    format.as_deref(),
                    field(&p, "max_results")?,
                )?)
            }
            Adapter::Strings => output(ranges::strings(
                &mut reader,
                selection(&p)?,
                field(&p, "min_length")?,
                field(&p, "max_results")?,
            )?),
            Adapter::Zip => output(zip::validate(&mut reader, number(&p, "offset")?)?),
            _ => Err(invalid()),
        }
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
