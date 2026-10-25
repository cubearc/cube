//! Pipeline-only values; direct operation APIs keep their ordinary Rust types.
use super::{Error, invalid};
use crate::errors::ErrorCode;
use serde::{Deserialize, Serialize};
/// The initial structured kind is an inert byte range, with no associated path
/// or file permission. No initial transform adapter consumes it; identity works.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Bytes,
    Text,
    ByteRange,
    ForensicReport,
}
impl std::str::FromStr for Kind {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Error> {
        match value {
            "bytes" => Ok(Self::Bytes),
            "text" => Ok(Self::Text),
            "byte_range" => Ok(Self::ByteRange),
            "forensic_report" => Ok(Self::ForensicReport),
            _ => Err(
                invalid("type must be bytes, text, byte_range or forensic_report")
                    .parameter("input_type"),
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Bytes {
        #[serde(rename = "base64", with = "base64_wire")]
        data: Vec<u8>,
    },
    Text {
        data: String,
    },
    ByteRange {
        offset: u64,
        length: Option<u64>,
    },
    ForensicReport {
        data: super::ForensicReport,
    },
}
impl Value {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Bytes { .. } => Kind::Bytes,
            Self::Text { .. } => Kind::Text,
            Self::ByteRange { .. } => Kind::ByteRange,
            Self::ForensicReport { .. } => Kind::ForensicReport,
        }
    }
    /// Logical budget units: exact byte/UTF-8 length or 16 bytes for a byte range.
    pub fn size(&self) -> usize {
        match self {
            Self::Bytes { data } => data.len(),
            Self::Text { data } => data.len(),
            Self::ByteRange { .. } => 16,
            Self::ForensicReport { data } => data.size(),
        }
    }
    pub fn validate(&self, maximum: usize) -> Result<(), Error> {
        if self.size() > maximum {
            return Err(Error::new(
                ErrorCode::ResourceLimit,
                "value exceeds byte budget",
            ));
        }
        if let Self::ForensicReport { data } = self {
            data.validate()?;
        }
        if let Self::ByteRange {
            offset,
            length: Some(length),
        } = self
        {
            offset
                .checked_add(*length)
                .ok_or_else(|| invalid("byte range offset + length overflows"))?;
        }
        Ok(())
    }
}
/// JSON parameter scalars preserve their actual types; binary keys need an
/// explicit {"base64":"..."} object. Floats/null/lists are not parameters in v1.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Parameter {
    Integer(i64),
    Boolean(bool),
    Text(String),
    Bytes {
        #[serde(rename = "base64", with = "base64_wire")]
        data: Vec<u8>,
    },
}
impl Parameter {
    pub(super) fn borrowed(&self) -> crate::operations::ParameterValue<'_> {
        use crate::operations::ParameterValue as V;
        match self {
            Self::Integer(n) => V::Integer(*n),
            Self::Boolean(v) => V::Boolean(*v),
            Self::Text(v) => V::Text(v),
            Self::Bytes { data } => V::Bytes(data),
        }
    }
    pub(super) fn size(&self) -> usize {
        match self {
            Self::Integer(_) => 8,
            Self::Boolean(_) => 1,
            Self::Text(v) => v.len(),
            Self::Bytes { data } => data.len(),
        }
    }
}
pub(super) mod base64_wire {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&crate::encoding::base64_encode(data))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.len() > super::super::MAX_VALUE_BYTES.div_ceil(3) * 4 {
            return Err(serde::de::Error::custom("base64 exceeds value budget"));
        }
        crate::encoding::base64_decode(&text)
            .map_err(|_| serde::de::Error::custom("invalid canonical base64"))
    }
}
