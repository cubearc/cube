//! Explicit character sets and string escape profiles without normalization.
mod catalog;
mod escapes;

pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
pub use escapes::{html_entity_decode, html_entity_encode, json_escape_decode, json_escape_encode};

use crate::errors::{ErrorCode, OperationError};

/// Maximum UTF-8 input byte count for text encoders.
pub const MAX_TEXT: usize = super::MAX_INPUT;
/// Maximum byte count of UTF-16 input and output, including a possible BOM.
pub const MAX_UTF16: usize = 4 * MAX_TEXT;
/// Maximum byte count of escaped/encoded text.
pub const MAX_ESCAPED: usize = 12 * MAX_TEXT;

const ASCII_ENCODE: &str = "text.ascii.encode";
const ASCII_DECODE: &str = "text.ascii.decode";
const UTF16_ENCODE: &str = "text.utf16.encode";
const UTF16_DECODE: &str = "text.utf16.decode";
const JSON_ENCODE: &str = "text.json_escape.encode";
const JSON_DECODE: &str = "text.json_escape.decode";
const HTML_ENCODE: &str = "text.html_entity.encode";
const HTML_DECODE: &str = "text.html_entity.decode";

fn check(length: usize, max: usize, id: &'static str) -> Result<(), OperationError> {
    if length > max {
        Err(OperationError::new(
            ErrorCode::ResourceLimit,
            format!("value exceeds {max} bytes"),
        )
        .operation(id))
    } else {
        Ok(())
    }
}
fn invalid(id: &'static str, offset: usize, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
        .operation(id)
        .offset(offset as u64)
}

/// Convert exact ASCII scalars to the corresponding byte values.
pub fn ascii_encode(text: &str) -> Result<Vec<u8>, OperationError> {
    ascii(text.as_bytes(), ASCII_ENCODE)?;
    Ok(text.as_bytes().to_vec())
}
/// Convert ASCII byte values to the corresponding Unicode scalars.
pub fn ascii_decode(data: &[u8]) -> Result<String, OperationError> {
    ascii(data, ASCII_DECODE)?;
    Ok(String::from_utf8(data.to_vec()).expect("ASCII is UTF-8"))
}
fn ascii(data: &[u8], id: &'static str) -> Result<(), OperationError> {
    check(data.len(), MAX_TEXT, id)?;
    if let Some(index) = data.iter().position(|byte| !byte.is_ascii()) {
        return Err(invalid(
            id,
            index,
            "only ASCII code points U+0000 through U+007F are allowed",
        ));
    }
    Ok(())
}

/// Explicit byte ordering for UTF-16 code units.
#[derive(Clone, Copy, Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ByteOrder {
    /// Little-endian.
    Le,
    /// Big-endian.
    Be,
}
impl ByteOrder {
    fn pack(self, word: u16) -> [u8; 2] {
        match self {
            Self::Le => word.to_le_bytes(),
            Self::Be => word.to_be_bytes(),
        }
    }
    fn unpack(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Le => u16::from_le_bytes(bytes),
            Self::Be => u16::from_be_bytes(bytes),
        }
    }
}

/// Handling of an initial UTF-16 byte-order mark, with the byte order fixed separately.
#[derive(Clone, Copy, Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Bom {
    /// Emit a mark and require and remove a matching initial mark when decoding.
    Require,
    /// Refuse either marker at the start.
    Forbid,
    /// Encode the given scalars and remove one matching initial mark when decoding.
    Strip,
    /// Treat every code unit as data, including a leading U+FEFF or U+FFFE.
    Preserve,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Utf16Settings {
    /// Required code-unit byte order.
    pub byte_order: ByteOrder,
    /// Required initial marker policy.
    pub bom: Bom,
}

/// Encode Unicode scalars as UTF-16 in the explicit byte order. Only the
/// Require policy adds a byte-order mark.
pub fn utf16_encode(text: &str, settings: Utf16Settings) -> Result<Vec<u8>, OperationError> {
    check(text.len(), MAX_TEXT, UTF16_ENCODE)?;
    if matches!(settings.bom, Bom::Forbid) && text.starts_with(['\u{feff}', '\u{fffe}']) {
        return Err(invalid(
            UTF16_ENCODE,
            0,
            "initial byte-order mark forbidden",
        ));
    }
    let mut output = Vec::with_capacity(text.len() * 2 + 2);
    if matches!(settings.bom, Bom::Require) {
        output.extend(settings.byte_order.pack(0xfeff));
    }
    for word in text.encode_utf16() {
        output.extend(settings.byte_order.pack(word));
    }
    Ok(output)
}

/// Decode complete UTF-16 code units with strict surrogate and initial BOM handling.
/// Error offsets refer to bytes of the original input.
pub fn utf16_decode(data: &[u8], settings: Utf16Settings) -> Result<String, OperationError> {
    check(data.len(), MAX_UTF16, UTF16_DECODE)?;
    if !data.len().is_multiple_of(2) {
        return Err(invalid(
            UTF16_DECODE,
            data.len() - 1,
            "incomplete UTF-16 code unit",
        ));
    }
    let first = data
        .first_chunk::<2>()
        .map(|bytes| settings.byte_order.unpack(*bytes));
    let start = match (settings.bom, first) {
        (Bom::Require | Bom::Strip, Some(0xfeff)) => 2,
        (Bom::Require, _) | (Bom::Strip, Some(0xfffe)) => {
            return Err(invalid(
                UTF16_DECODE,
                0,
                "matching byte-order mark required by policy",
            ));
        }
        (Bom::Forbid, Some(0xfeff | 0xfffe)) => {
            return Err(invalid(
                UTF16_DECODE,
                0,
                "initial byte-order mark forbidden",
            ));
        }
        _ => 0,
    };
    let mut output = String::with_capacity(data.len().min(MAX_UTF16));
    let words = data[start..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| settings.byte_order.unpack(*word));
    let mut byte_offset = start;
    for scalar in char::decode_utf16(words) {
        let scalar =
            scalar.map_err(|_| invalid(UTF16_DECODE, byte_offset, "unpaired UTF-16 surrogate"))?;
        check(output.len() + scalar.len_utf8(), MAX_UTF16, UTF16_DECODE)?;
        byte_offset += scalar.len_utf16() * 2;
        output.push(scalar);
    }
    Ok(output)
}

impl Utf16Settings {
    /// Parse the exact public byte order and BOM spellings.
    pub fn parse(byte_order: &str, bom: &str) -> Result<Self, OperationError> {
        let invalid = |parameter, detail| {
            OperationError::new(ErrorCode::InvalidParameter, detail).parameter(parameter)
        };
        let byte_order = match byte_order {
            "le" => ByteOrder::Le,
            "be" => ByteOrder::Be,
            _ => return Err(invalid("byte_order", "use le or be")),
        };
        let bom = match bom {
            "require" => Bom::Require,
            "forbid" => Bom::Forbid,
            "strip" => Bom::Strip,
            "preserve" => Bom::Preserve,
            _ => return Err(invalid("bom", "use require, forbid, strip or preserve")),
        };
        Ok(Self { byte_order, bom })
    }
}
