//! Byte-oriented radix encodings and the RFC 3986 component profile.
mod base85;
mod base92;
mod catalog;

use crate::errors::{ErrorCode, OperationError};
use num_bigint::BigUint;
use std::fmt::Write as _;

pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;

/// Maximum byte count for the linear encodings.
pub const MAX_BYTES: usize = 1 << 20;
/// Maximum byte count for the magnitude-conversion profiles.
pub const MAX_MAGNITUDE_BYTES: usize = 4096;

const GMP36: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const GMP62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Exact byte-to-text encoding, including alphabet and framing.
#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Profile {
    /// Binary with one space between byte tokens.
    Binary,
    /// Octal with one space between byte tokens.
    Octal,
    /// Lowercase Base36 with explicit leading zero bytes.
    Base36,
    /// GMP alphabet Base62 with explicit leading zero bytes.
    Base62,
    /// Adobe Ascii85 with literal opening and closing markers.
    Ascii85,
    /// Adobe Ascii85 without the outer markers.
    Ascii85Bare,
    /// Joachim Henke's original basE91 alphabet and bit packing.
    Base91,
    /// Original Base92 with the tilde empty marker.
    Base92,
    /// RFC 3986 component percent encoding.
    UrlComponent,
}

impl Profile {
    pub(crate) const ALL: &[Self] = &[
        Self::Binary,
        Self::Octal,
        Self::Base36,
        Self::Base62,
        Self::Ascii85,
        Self::Ascii85Bare,
        Self::Base91,
        Self::Base92,
        Self::UrlComponent,
    ];

    pub(crate) fn find(operation: &str) -> Option<(Self, bool)> {
        Self::ALL.iter().copied().find_map(|profile| {
            if operation == profile.id(true) {
                Some((profile, true))
            } else if operation == profile.id(false) {
                Some((profile, false))
            } else {
                None
            }
        })
    }

    /// Stable ID for the named direction.
    pub const fn id(self, encode: bool) -> &'static str {
        catalog::id(self, encode)
    }

    /// Limit on the exact raw byte value.
    pub const fn byte_limit(self) -> usize {
        match self {
            Self::Base36 | Self::Base62 => MAX_MAGNITUDE_BYTES,
            _ => MAX_BYTES,
        }
    }

    /// Limit on the encoded byte value.
    pub const fn text_limit(self) -> usize {
        self.max_encoded(self.byte_limit())
    }

    /// Conservative output size before doing any interpretation.
    pub const fn max_encoded(self, length: usize) -> usize {
        match self {
            Self::Binary => (length * 9).saturating_sub(1),
            Self::Octal => (length * 4).saturating_sub(1),
            Self::Ascii85 => length.div_ceil(4) * 5 + 4,
            Self::Ascii85Bare => length.div_ceil(4) * 5,
            Self::UrlComponent => length * 3,
            Self::Base92 if length == 0 => 1,
            Self::Base92 => length * 2,
            Self::Base36 | Self::Base62 | Self::Base91 => length * 2,
        }
    }

    /// Encode exact bytes using the selected alphabet and canonical form.
    pub fn encode(self, data: &[u8]) -> Result<String, OperationError> {
        let id = self.id(true);
        check(data.len(), self.byte_limit(), id)?;
        Ok(match self {
            Self::Binary | Self::Octal => tokens(data, self),
            Self::Base36 => magnitude_encode(data, GMP36),
            Self::Base62 => magnitude_encode(data, GMP62),
            Self::Ascii85 | Self::Ascii85Bare => {
                base85::encode(data, matches!(self, Self::Ascii85))
            }
            Self::Base91 => base91::EncodeIterator::new(data.iter().copied())
                .as_char_iter()
                .collect(),
            Self::Base92 => base92::encode(data),
            Self::UrlComponent => percent_encode(data),
        })
    }

    /// Decode the exact named representation into arbitrary bytes.
    pub fn decode(self, text: &str) -> Result<Vec<u8>, OperationError> {
        let id = self.id(false);
        check(text.len(), self.text_limit(), id)?;
        let value = match self {
            Self::Binary => parse_tokens(text, 8, 2, id)?,
            Self::Octal => parse_tokens(text, 3, 8, id)?,
            Self::Base36 => magnitude_decode(text, GMP36, id)?,
            Self::Base62 => magnitude_decode(text, GMP62, id)?,
            Self::Ascii85 | Self::Ascii85Bare => {
                base85::decode(text, matches!(self, Self::Ascii85), id)?
            }
            Self::Base91 => {
                let bytes: Vec<u8> = base91::DecodeIterator::new(text.bytes())
                    .take(MAX_BYTES + 1)
                    .collect();
                check(bytes.len(), MAX_BYTES, id)?;
                if self.encode(&bytes)? != text {
                    return Err(invalid(id, 0, "use the canonical basE91 alphabet and tail"));
                }
                bytes
            }
            Self::Base92 => base92::decode(text, id)?,
            Self::UrlComponent => percent_decode(text, id)?,
        };
        check(value.len(), self.byte_limit(), id)?;
        Ok(value)
    }
}

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

fn invalid(id: &'static str, offset: usize, detail: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, detail)
        .operation(id)
        .offset(offset as u64)
}

fn tokens(data: &[u8], profile: Profile) -> String {
    let mut text = String::with_capacity(profile.max_encoded(data.len()));
    for (i, byte) in data.iter().enumerate() {
        if i != 0 {
            text.push(' ');
        }
        if matches!(profile, Profile::Binary) {
            write!(text, "{byte:08b}").expect("infallible string formatting");
        } else {
            write!(text, "{byte:03o}").expect("infallible string formatting");
        }
    }
    text
}

fn parse_tokens(
    text: &str,
    width: usize,
    radix: u32,
    id: &'static str,
) -> Result<Vec<u8>, OperationError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let mut output = Vec::with_capacity((text.len() / (width + 1) + 1).min(MAX_BYTES));
    let mut offset = 0;
    for token in text.split(' ') {
        let value = if token.len() == width && token.bytes().all(|byte| byte.is_ascii_digit()) {
            u8::from_str_radix(token, radix).ok()
        } else {
            None
        };
        output
            .push(value.ok_or_else(|| invalid(id, offset, "expected one fixed-width byte token"))?);
        offset += width + 1;
    }
    Ok(output)
}

fn magnitude_encode(data: &[u8], alphabet: &[u8]) -> String {
    let zeroes = data.iter().take_while(|&&byte| byte == 0).count();
    let mut output = "0".repeat(zeroes);
    if zeroes < data.len() {
        let digits = BigUint::from_bytes_be(&data[zeroes..]).to_radix_be(alphabet.len() as u32);
        output.extend(
            digits
                .into_iter()
                .map(|digit| char::from(alphabet[digit as usize])),
        );
    }
    output
}

fn magnitude_decode(
    text: &str,
    alphabet: &[u8],
    id: &'static str,
) -> Result<Vec<u8>, OperationError> {
    let zeroes = text.bytes().take_while(|&byte| byte == b'0').count();
    check(zeroes, MAX_MAGNITUDE_BYTES, id)?;
    let digits = text.as_bytes()[zeroes..]
        .iter()
        .enumerate()
        .map(|(i, byte)| {
            alphabet
                .iter()
                .position(|entry| byte == entry)
                .map(|digit| digit as u8)
                .ok_or_else(|| invalid(id, zeroes + i, "digit outside the selected alphabet"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut output = vec![0; zeroes];
    if !digits.is_empty() {
        let magnitude = BigUint::from_radix_be(&digits, alphabet.len() as u32)
            .expect("digits checked against alphabet");
        let bytes = magnitude.to_bytes_be();
        check(zeroes + bytes.len(), MAX_MAGNITUDE_BYTES, id)?;
        output.extend_from_slice(&bytes);
    }
    Ok(output)
}

fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn percent_encode(data: &[u8]) -> String {
    let mut output = String::with_capacity(data.len());
    for &byte in data {
        if unreserved(byte) {
            output.push(char::from(byte));
        } else {
            write!(output, "%{byte:02X}").expect("infallible string formatting");
        }
    }
    output
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn percent_decode(text: &str, id: &'static str) -> Result<Vec<u8>, OperationError> {
    let mut rest = text.as_bytes();
    let mut output = Vec::with_capacity(rest.len().min(MAX_BYTES));
    while let Some((&byte, tail)) = rest.split_first() {
        if output.len() == MAX_BYTES {
            check(MAX_BYTES + 1, MAX_BYTES, id)?;
        }
        if unreserved(byte) {
            output.push(byte);
            rest = tail;
        } else {
            let offset = text.len() - rest.len();
            let [b'%', high, low, tail @ ..] = rest else {
                return Err(invalid(
                    id,
                    offset,
                    "expected an unreserved byte or a percent triplet",
                ));
            };
            let (Some(high), Some(low)) = (hex(*high), hex(*low)) else {
                return Err(invalid(
                    id,
                    offset,
                    "expected two hexadecimal digits after percent",
                ));
            };
            output.push(high << 4 | low);
            rest = tail;
        }
    }
    Ok(output)
}

macro_rules! byte_text_api {
    ($encode:ident, $decode:ident, $profile:ident, $name:literal) => {
        #[doc = concat!("Encode exact bytes as ", $name, ".")]
        pub fn $encode(data: &[u8]) -> Result<String, OperationError> {
            Profile::$profile.encode(data)
        }
        #[doc = concat!("Decode ", $name, " into exact bytes.")]
        pub fn $decode(text: &str) -> Result<Vec<u8>, OperationError> {
            Profile::$profile.decode(text)
        }
    };
}
byte_text_api!(binary_encode, binary_decode, Binary, "Binary byte tokens");
byte_text_api!(octal_encode, octal_decode, Octal, "Octal byte tokens");
byte_text_api!(base36_encode, base36_decode, Base36, "Base36 bytes");
byte_text_api!(base62_encode, base62_decode, Base62, "Base62 bytes");
byte_text_api!(
    ascii85_encode,
    ascii85_decode,
    Ascii85,
    "Framed Adobe Ascii85"
);
byte_text_api!(
    ascii85_bare_encode,
    ascii85_bare_decode,
    Ascii85Bare,
    "Bare Adobe Ascii85"
);
byte_text_api!(base91_encode, base91_decode, Base91, "Original basE91");
byte_text_api!(base92_encode, base92_decode, Base92, "Original Base92");
byte_text_api!(
    url_component_encode,
    url_component_decode,
    UrlComponent,
    "RFC 3986 component"
);
