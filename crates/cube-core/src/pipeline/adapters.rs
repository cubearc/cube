//! Explicit allowlist referencing existing descriptors/functions, never a plugin registry.
use super::{Error, Kind, Parameter, Step, invalid};
use crate::{errors::ErrorCode, operations};
#[derive(Clone, Copy, Debug)]
pub(super) enum Action {
    MaskedText,
    Digest(crate::hashing::extended::Profile),
    ByteText(crate::encoding::byte_text::Profile, bool),
    Keccak256,
    Ripemd160,
    Base58CheckEncode,
    SemaphoreEncode,
    SemaphoreDecode,
    HtmlAsset,
    HarAsset,
    RotateLeft,
    RotateRight,
    XorExact,
    AddWrapping,
    SubWrapping,
    And,
    Or,
    Not,
    ReverseBytes,
    ReverseBits,
    Compress,
    Decompress,
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
    AsciiEncode,
    AsciiDecode,
    Utf16Encode,
    Utf16Decode,
    JsonEscapeEncode,
    JsonEscapeDecode,
    HtmlEntityEncode,
    HtmlEntityDecode,
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
        id: crate::binary::compression::ENCODE,
        version: 1,
        action: Action::Compress,
    },
    Adapter {
        id: crate::binary::compression::DECODE,
        version: 1,
        action: Action::Decompress,
    },
    Adapter {
        id: "text.ascii.encode",
        version: 1,
        action: Action::AsciiEncode,
    },
    Adapter {
        id: "text.ascii.decode",
        version: 1,
        action: Action::AsciiDecode,
    },
    Adapter {
        id: "text.utf16.encode",
        version: 1,
        action: Action::Utf16Encode,
    },
    Adapter {
        id: "text.utf16.decode",
        version: 1,
        action: Action::Utf16Decode,
    },
    Adapter {
        id: "text.json_escape.encode",
        version: 1,
        action: Action::JsonEscapeEncode,
    },
    Adapter {
        id: "text.json_escape.decode",
        version: 1,
        action: Action::JsonEscapeDecode,
    },
    Adapter {
        id: "text.html_entity.encode",
        version: 1,
        action: Action::HtmlEntityEncode,
    },
    Adapter {
        id: "text.html_entity.decode",
        version: 1,
        action: Action::HtmlEntityDecode,
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Binary.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Binary, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Binary.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Binary, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Octal.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Octal, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Octal.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Octal, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base36.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base36, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base36.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base36, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base62.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base62, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base62.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base62, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Ascii85.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Ascii85, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Ascii85.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Ascii85, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Ascii85Bare.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Ascii85Bare, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Ascii85Bare.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Ascii85Bare, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base91.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base91, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base91.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base91, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base92.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base92, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::Base92.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::Base92, false),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::UrlComponent.id(true),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::UrlComponent, true),
    },
    Adapter {
        id: crate::encoding::byte_text::Profile::UrlComponent.id(false),
        version: 1,
        action: Action::ByteText(crate::encoding::byte_text::Profile::UrlComponent, false),
    },
    Adapter {
        id: "binary.reverse_bits",
        version: 1,
        action: Action::ReverseBits,
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha224.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha224),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha384.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha384),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha512224.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha512224),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha512256.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha512256),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha3224.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha3224),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha3256.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha3256),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha3384.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha3384),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Sha3512.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Sha3512),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Keccak224.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Keccak224),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Keccak384.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Keccak384),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Keccak512.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Keccak512),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Ripemd128.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Ripemd128),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Ripemd256.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Ripemd256),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Ripemd320.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Ripemd320),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Blake2B512.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Blake2B512),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Blake2S256.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Blake2S256),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Blake3.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Blake3),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Md4.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Md4),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Whirlpool.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Whirlpool),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Crc8Smbus.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Crc8Smbus),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Crc16Arc.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Crc16Arc),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Crc64Ecma182.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Crc64Ecma182),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Adler32.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Adler32),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Fletcher16.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Fletcher16),
    },
    Adapter {
        id: crate::hashing::extended::Profile::Fletcher32.id(),
        version: 1,
        action: Action::Digest(crate::hashing::extended::Profile::Fletcher32),
    },
    Adapter {
        id: "binary.sub_wrapping",
        version: 1,
        action: Action::SubWrapping,
    },
    Adapter {
        id: "binary.add_wrapping",
        version: 1,
        action: Action::AddWrapping,
    },
    Adapter {
        id: "binary.xor_exact",
        version: 1,
        action: Action::XorExact,
    },
    Adapter {
        id: "binary.rotate_right",
        version: 1,
        action: Action::RotateRight,
    },
    Adapter {
        id: "binary.rotate_left",
        version: 1,
        action: Action::RotateLeft,
    },
    Adapter {
        id: "text.transform.masked",
        version: 1,
        action: Action::MaskedText,
    },
    Adapter {
        id: "hash.keccak256",
        version: 1,
        action: Action::Keccak256,
    },
    Adapter {
        id: "hash.ripemd160",
        version: 1,
        action: Action::Ripemd160,
    },
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
    Masked(crate::text::transform::Config),
    Utf16(crate::text::representation::Utf16Settings),
    AsciiOnly(bool),
    Compression(crate::binary::compression::Profile, Option<u8>),
    Version(u8),
    WebAsset {
        index: usize,
        selector: crate::web::assets::HtmlSelector,
    },
    Rotation {
        count: u64,
        unit_bytes: usize,
    },
    Paired {
        key: Vec<u8>,
        unit_bytes: usize,
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
    let text_lists: Vec<Vec<&str>> = step
        .parameters
        .values()
        .map(|p| match p {
            Parameter::TextList { values } => values.iter().map(String::as_str).collect(),
            _ => Vec::new(),
        })
        .collect();
    let values: Vec<_> = step
        .parameters
        .iter()
        .zip(&text_lists)
        .map(|((k, v), text)| (k.as_str(), v.borrowed(text)))
        .collect();
    descriptor
        .validate(&values)
        .map_err(|e| invalid(e.reason).parameter(&e.parameter))?;
    let integer = |name: &str| match step.parameters.get(name) {
        Some(Parameter::Integer(n)) => Ok(*n as i32),
        _ => Err(invalid("required integer parameter is missing or invalid").parameter(name)),
    };
    let arguments = match adapter.action {
        Action::Compress | Action::Decompress => {
            use crate::binary::compression as c;
            let Some(Parameter::Text(profile)) = step.parameters.get("profile") else {
                return Err(invalid("select an exact compression profile").parameter("profile"));
            };
            let profile = c::Profile::parse(profile, adapter.id).map_err(Error::from_operation)?;
            let level = step
                .parameters
                .get("level")
                .map(|_| integer("level"))
                .transpose()?
                .map(|value| value as u8);
            Arguments::Compression(profile, level)
        }
        Action::JsonEscapeEncode => {
            let ascii_only = match step.parameters.get("ascii_only") {
                Some(Parameter::Boolean(value)) => *value,
                None => true,
                _ => return Err(invalid("expected a boolean").parameter("ascii_only")),
            };
            Arguments::AsciiOnly(ascii_only)
        }
        Action::Utf16Encode | Action::Utf16Decode => {
            let required_text = |name| match step.parameters.get(name) {
                Some(Parameter::Text(value)) => Ok(value.as_str()),
                _ => Err(invalid("required text parameter").parameter(name)),
            };
            Arguments::Utf16(
                crate::text::representation::Utf16Settings::parse(
                    required_text("byte_order")?,
                    required_text("bom")?,
                )
                .map_err(Error::from_operation)?,
            )
        }
        Action::MaskedText => Arguments::Masked(super::masked::prepare(step)?),
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

        Action::RotateLeft | Action::RotateRight | Action::AddWrapping | Action::SubWrapping => {
            let unit_bytes = match step.parameters.get("unit_bytes") {
                Some(Parameter::Integer(n)) => {
                    usize::try_from(*n).map_err(|_| invalid("invalid unit width"))?
                }
                None => 1,
                _ => return Err(invalid("expected integer unit width").parameter("unit_bytes")),
            };
            crate::binary::transform::validate_unit(unit_bytes, adapter.id)
                .map_err(Error::from_operation)?;
            if matches!(adapter.action, Action::RotateLeft | Action::RotateRight) {
                let Some(Parameter::Integer(count)) = step.parameters.get("count") else {
                    return Err(invalid("required nonnegative integer count").parameter("count"));
                };
                Arguments::Rotation {
                    count: *count as u64,
                    unit_bytes,
                }
            } else {
                let Some(Parameter::Bytes { data }) = step.parameters.get("key") else {
                    return Err(invalid("explicit byte operand required").parameter("key"));
                };
                if data.len() > 65536 {
                    return Err(Error::new(
                        ErrorCode::ResourceLimit,
                        "operand exceeds pipeline's 64 KiB parameter limit",
                    )
                    .parameter("key"));
                }
                if !data.len().is_multiple_of(unit_bytes) {
                    return Err(invalid("operand must contain complete units").parameter("key"));
                }
                Arguments::Paired {
                    key: data.clone(),
                    unit_bytes,
                }
            }
        }
        Action::ShiftLeft | Action::ShiftRight | Action::SwapEndian | Action::ReverseBits => {
            let unit_bytes = match step.parameters.get("unit_bytes") {
                Some(Parameter::Integer(n)) => usize::try_from(*n)
                    .map_err(|_| invalid("invalid unit width").parameter("unit_bytes"))?,
                None => 1,
                _ => return Err(invalid("expected integer unit width").parameter("unit_bytes")),
            };
            let bits = if matches!(adapter.action, Action::SwapEndian | Action::ReverseBits) {
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
        Action::Xor | Action::XorExact | Action::And | Action::Or => {
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
