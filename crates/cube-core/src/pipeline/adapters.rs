//! Explicit allowlist referencing existing descriptors/functions, never a plugin registry.
use super::{Error, Kind, Parameter, Step, invalid};
use crate::{errors::ErrorCode, operations};
#[derive(Clone, Copy, Debug)]
pub(super) enum Action {
    Base58CheckEncode,
    SemaphoreEncode,
    SemaphoreDecode,
    HtmlAsset,
    HarAsset,
    And,
    Or,
    Not,
    ReverseBytes,
    ShiftLeft,
    ShiftRight,
    SwapEndian,
    TextSelect,
    A1z26Encode,
    A1z26Decode,
    Base32Encode,
    Base32Decode,
    Base58Encode,
    Base58Decode,
    Base64Encode,
    Base64Decode,
    HexEncode,
    HexDecode,
    CaesarEncode,
    CaesarDecode,
    Rot13,
    Atbash,
    VigenereEncode,
    VigenereDecode,
    Beaufort,
    AffineEncode,
    AffineDecode,
    Crc32,
    Sha256,
    Sha512,
    Sha1,
    Md5,
    Crc32c,
    Crc16Ibm3740,
    Xor,
    Utf8Encode,
    Utf8Decode,
    Signatures,
    HexInspect,
    Strings,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Adapter {
    pub id: &'static str,
    pub version: u32,
    pub action: Action,
}
pub(super) const ADAPTERS: &[Adapter] = &[
    Adapter {
        id: "encoding.base58check.encode",
        version: 1,
        action: Action::Base58CheckEncode,
    },
    Adapter {
        id: "symbol.semaphore.encode",
        version: 1,
        action: Action::SemaphoreEncode,
    },
    Adapter {
        id: "symbol.semaphore.decode",
        version: 1,
        action: Action::SemaphoreDecode,
    },
    Adapter {
        id: "web.html.asset",
        version: 1,
        action: Action::HtmlAsset,
    },
    Adapter {
        id: "web.har.asset",
        version: 1,
        action: Action::HarAsset,
    },
    Adapter {
        id: "hash.sha512",
        version: 1,
        action: Action::Sha512,
    },
    Adapter {
        id: "hash.sha1",
        version: 1,
        action: Action::Sha1,
    },
    Adapter {
        id: "hash.md5",
        version: 1,
        action: Action::Md5,
    },
    Adapter {
        id: "hash.crc32c",
        version: 1,
        action: Action::Crc32c,
    },
    Adapter {
        id: "hash.crc16_ibm3740",
        version: 1,
        action: Action::Crc16Ibm3740,
    },
    Adapter {
        id: "binary.and",
        version: 1,
        action: Action::And,
    },
    Adapter {
        id: "binary.or",
        version: 1,
        action: Action::Or,
    },
    Adapter {
        id: "binary.not",
        version: 1,
        action: Action::Not,
    },
    Adapter {
        id: "binary.reverse",
        version: 1,
        action: Action::ReverseBytes,
    },
    Adapter {
        id: "binary.shift_left",
        version: 1,
        action: Action::ShiftLeft,
    },
    Adapter {
        id: "binary.shift_right",
        version: 1,
        action: Action::ShiftRight,
    },
    Adapter {
        id: "binary.swap_endian",
        version: 1,
        action: Action::SwapEndian,
    },
    Adapter {
        id: "text.select",
        version: 1,
        action: Action::TextSelect,
    },
    Adapter {
        id: "symbol.a1z26.encode",
        version: 1,
        action: Action::A1z26Encode,
    },
    Adapter {
        id: "symbol.a1z26.decode",
        version: 1,
        action: Action::A1z26Decode,
    },
    Adapter {
        id: "file.signature.detect",
        version: 1,
        action: Action::Signatures,
    },
    Adapter {
        id: "file.hex.inspect",
        version: 1,
        action: Action::HexInspect,
    },
    Adapter {
        id: "file.strings.extract",
        version: 1,
        action: Action::Strings,
    },
    Adapter {
        id: "encoding.base32.encode",
        version: 1,
        action: Action::Base32Encode,
    },
    Adapter {
        id: "encoding.base32.decode",
        version: 1,
        action: Action::Base32Decode,
    },
    Adapter {
        id: "encoding.base58.encode",
        version: 1,
        action: Action::Base58Encode,
    },
    Adapter {
        id: "encoding.base58.decode",
        version: 1,
        action: Action::Base58Decode,
    },
    Adapter {
        id: "encoding.base64.encode",
        version: 1,
        action: Action::Base64Encode,
    },
    Adapter {
        id: "encoding.base64.decode",
        version: 1,
        action: Action::Base64Decode,
    },
    Adapter {
        id: "encoding.hex.encode",
        version: 1,
        action: Action::HexEncode,
    },
    Adapter {
        id: "encoding.hex.decode",
        version: 1,
        action: Action::HexDecode,
    },
    Adapter {
        id: "cipher.caesar.encode",
        version: 1,
        action: Action::CaesarEncode,
    },
    Adapter {
        id: "cipher.caesar.decode",
        version: 1,
        action: Action::CaesarDecode,
    },
    Adapter {
        id: "cipher.rot13",
        version: 1,
        action: Action::Rot13,
    },
    Adapter {
        id: "cipher.atbash",
        version: 1,
        action: Action::Atbash,
    },
    Adapter {
        id: "cipher.vigenere.encode",
        version: 1,
        action: Action::VigenereEncode,
    },
    Adapter {
        id: "cipher.vigenere.decode",
        version: 1,
        action: Action::VigenereDecode,
    },
    Adapter {
        id: "cipher.beaufort",
        version: 1,
        action: Action::Beaufort,
    },
    Adapter {
        id: "cipher.affine.encode",
        version: 1,
        action: Action::AffineEncode,
    },
    Adapter {
        id: "cipher.affine.decode",
        version: 1,
        action: Action::AffineDecode,
    },
    Adapter {
        id: "hash.crc32",
        version: 1,
        action: Action::Crc32,
    },
    Adapter {
        id: "hash.sha256",
        version: 1,
        action: Action::Sha256,
    },
    Adapter {
        id: "binary.xor",
        version: 1,
        action: Action::Xor,
    },
    Adapter {
        id: "text.utf8.encode",
        version: 1,
        action: Action::Utf8Encode,
    },
    Adapter {
        id: "text.utf8.decode",
        version: 1,
        action: Action::Utf8Decode,
    },
];
#[derive(Clone, Debug)]
pub(super) enum Arguments {
    Version(u8),
    WebAsset {
        index: usize,
        selector: crate::web::assets::HtmlSelector,
    },
    Units {
        bits: u8,
        unit_bytes: usize,
    },
    Selector(crate::text::selection::Selector),
    None,
    Shift(i32),
    Key(String),
    Bytes(Vec<u8>),
    Affine(i32, i32),
    Forensic {
        range: crate::forensics::ByteRange,
        width: usize,
        min_length: usize,
        max_results: usize,
    },
}
#[derive(Clone, Debug)]
pub(super) struct Prepared {
    pub adapter: Adapter,
    pub arguments: Arguments,
}
impl Prepared {
    pub fn kinds(&self) -> Result<(Kind, Kind), Error> {
        self.adapter.kinds()
    }
}
impl Adapter {
    pub fn kinds(&self) -> Result<(Kind, Kind), Error> {
        let d = operations::describe(self.id)
            .ok_or_else(|| invalid("adapter descriptor is unavailable"))?;
        if matches!(
            self.action,
            Action::Signatures | Action::HexInspect | Action::Strings
        ) {
            return Ok((Kind::Bytes, Kind::ForensicReport));
        }
        Ok((d.input_type.parse()?, d.output_type.parse()?))
    }
}
pub(super) fn prepare(step: &Step) -> Result<Prepared, Error> {
    let adapter = *ADAPTERS
        .iter()
        .find(|a| a.id == step.operation)
        .ok_or_else(|| invalid("operation has no permitted bounded pipeline adapter"))?;
    if step.version != adapter.version {
        return Err(invalid("unsupported operation-contract version").parameter("version"));
    }
    if step
        .parameters
        .keys()
        .any(|k| k.len() > 64 || !k.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
    {
        return Err(invalid(
            "parameter names must be lowercase ASCII and at most 64 bytes",
        ));
    }
    let descriptor =
        operations::describe(adapter.id).ok_or_else(|| invalid("missing adapter descriptor"))?;
    let values: Vec<_> = step
        .parameters
        .iter()
        .map(|(k, v)| (k.as_str(), v.borrowed()))
        .collect();
    descriptor
        .validate(&values)
        .map_err(|e| invalid(e.reason).parameter(&e.parameter))?;
    let integer = |name: &str| match step.parameters.get(name) {
        Some(Parameter::Integer(n)) => Ok(*n as i32),
        _ => Err(invalid("required integer parameter is missing or invalid").parameter(name)),
    };
    let arguments = match adapter.action {
        Action::Base58CheckEncode => Arguments::Version(integer("version")? as u8),
        Action::HtmlAsset | Action::HarAsset => {
            let index = usize::try_from(integer("index")?)
                .map_err(|_| invalid("invalid asset index").parameter("index"))?;
            let selector = match step.parameters.get("selector") {
                None => crate::web::assets::HtmlSelector::Inline,
                Some(Parameter::Text(value)) => {
                    crate::web::assets::HtmlSelector::parse(value).map_err(Error::from_operation)?
                }
                _ => return Err(invalid("selector must be text").parameter("selector")),
            };
            Arguments::WebAsset { index, selector }
        }

        Action::ShiftLeft | Action::ShiftRight | Action::SwapEndian => {
            let unit_bytes = match step.parameters.get("unit_bytes") {
                Some(Parameter::Integer(n)) => usize::try_from(*n)
                    .map_err(|_| invalid("invalid unit width").parameter("unit_bytes"))?,
                None => 1,
                _ => return Err(invalid("expected integer unit width").parameter("unit_bytes")),
            };
            let bits = if matches!(adapter.action, Action::SwapEndian) {
                0
            } else {
                integer("bits")? as u8
            };
            crate::binary::transform::validate_shift(bits, unit_bytes, adapter.id)
                .map_err(Error::from_operation)?;
            Arguments::Units { bits, unit_bytes }
        }
        Action::TextSelect => {
            let Some(Parameter::Text(selector)) = step.parameters.get("selector") else {
                return Err(invalid("selector is required").parameter("selector"));
            };
            Arguments::Selector(
                crate::text::selection::Selector::parse(selector).map_err(Error::from_operation)?,
            )
        }
        Action::HexInspect | Action::Strings => {
            let number = |name: &str, default: u64| -> Result<u64, Error> {
                match step.parameters.get(name) {
                    Some(Parameter::Integer(n)) => u64::try_from(*n)
                        .map_err(|_| invalid("expected nonnegative integer").parameter(name)),
                    None => Ok(default),
                    _ => Err(invalid("expected integer").parameter(name)),
                }
            };
            let range = crate::forensics::ByteRange {
                offset: number("offset", 0)?,
                length: step
                    .parameters
                    .contains_key("length")
                    .then(|| number("length", 0))
                    .transpose()?,
            };
            if let Some(length) = range.length {
                range
                    .offset
                    .checked_add(length)
                    .ok_or_else(|| invalid("offset + length overflows"))?;
            }
            let narrow = |name, default| {
                usize::try_from(number(name, default)?)
                    .map_err(|_| invalid("parameter exceeds platform bounds").parameter(name))
            };
            Arguments::Forensic {
                range,
                width: narrow("width", 16)?,
                min_length: narrow("min_length", 4)?,
                max_results: narrow("max_results", 1000)?,
            }
        }
        Action::CaesarEncode | Action::CaesarDecode => Arguments::Shift(integer("shift")?),
        Action::VigenereEncode | Action::VigenereDecode | Action::Beaufort => {
            let Some(Parameter::Text(key)) = step.parameters.get("key") else {
                return Err(invalid("ASCII-letter key required").parameter("key"));
            };
            crate::ciphers::classical::validate_key(key, adapter.id)
                .map_err(Error::from_operation)?;
            Arguments::Key(key.clone())
        }
        Action::AffineEncode | Action::AffineDecode => {
            let (a, b) = (integer("a")?, integer("b")?);
            crate::ciphers::classical::validate_affine_keys(a, b, adapter.id)
                .map_err(Error::from_operation)?;
            Arguments::Affine(a, b)
        }
        Action::Xor | Action::And | Action::Or => {
            let Some(Parameter::Bytes { data }) = step.parameters.get("key") else {
                return Err(invalid("explicit binary key required").parameter("key"));
            };
            if data.len() > 65536 {
                return Err(Error::new(
                    ErrorCode::ResourceLimit,
                    "binary operand exceeds pipeline's 64 KiB parameter limit",
                )
                .parameter("key"));
            }
            Arguments::Bytes(data.clone())
        }
        _ => Arguments::None,
    };
    Ok(Prepared { adapter, arguments })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn adapter_references_are_unique_known_and_typed() {
        let mut ids = BTreeSet::new();
        for adapter in ADAPTERS {
            assert!(ids.insert(adapter.id), "duplicate adapter: {}", adapter.id);
            let descriptor = operations::require(adapter.id).expect("known adapter operation");
            assert!(adapter.version > 0);
            let (input, output) = adapter.kinds().expect("typed adapter");
            assert!(descriptor.deterministic);
            if output == Kind::ForensicReport {
                assert_eq!(input, Kind::Bytes);
            } else {
                assert_eq!(input, descriptor.input_type.parse().unwrap());
                assert_eq!(output, descriptor.output_type.parse().unwrap());
            }
        }
    }
}
