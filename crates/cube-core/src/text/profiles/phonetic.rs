//! Pinned phonetic keys for explicit character alphabets and bounded batches.
use super::{check_report, invalid, parameter, resource};
use crate::errors::OperationError;
use rphonetic::{Caverphone1, Caverphone2, DoubleMetaphone, Encoder, Metaphone, Nysiis};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(super) const APACHE_PROFILE: &str = "apache-commons-codec-1.15";
const OP: &str = "text.phonetic";
const MAX_NAMES: usize = 256;
const MAX_NAME: usize = 256;
const MAX_BATCH_BYTES: usize = 64 << 10;
const MAX_BATCH_WORK: usize = 16 << 20;

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    SoundexNara,
    Nysiis,
    Metaphone,
    DoubleMetaphone,
    Caverphone1,
    Caverphone2,
    Cologne,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub method: Method,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[schemars(length(max = 256))]
    pub names: Option<Vec<String>>,
    #[serde(default)]
    pub strict: Option<bool>,
    #[serde(default)]
    #[schemars(range(min = 1, max = 32))]
    pub max_code_length: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Key {
    Single { key: String },
    Double { primary: String, alternate: String },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub method: Method,
    pub profile: &'static str,
    pub strict: Option<bool>,
    pub max_code_length: Option<usize>,
    pub keys: Vec<Key>,
}

pub(super) fn ascii_name(name: &str) -> Result<(), OperationError> {
    if name.len() > MAX_NAME {
        return Err(resource("phonetic name exceeds 256 ASCII bytes"));
    }
    if let Some(offset) = name.bytes().position(|byte| !byte.is_ascii_alphabetic()) {
        return Err(invalid("phonetic profile requires ASCII letters").offset(offset as u64));
    }
    Ok(())
}

fn cologne_name(name: &str) -> Result<(), OperationError> {
    if name.len() > 1024 || name.chars().take(MAX_NAME + 1).count() > MAX_NAME {
        return Err(resource(
            "Cologne name exceeds 256 scalars or 1024 UTF-8 bytes",
        ));
    }
    if let Some((offset, _)) = name.char_indices().find(|&(_, c)| {
        !c.is_ascii_alphabetic() && !matches!(c, 'Ä' | 'Ö' | 'Ü' | 'ä' | 'ö' | 'ü' | 'ß')
    }) {
        return Err(invalid("Cologne name is outside the declared alphabet").offset(offset as u64));
    }
    Ok(())
}

fn soundex(name: &str) -> String {
    let mut name = name.bytes().map(|byte| byte.to_ascii_uppercase());
    let Some(initial) = name.next() else {
        return String::new();
    };
    let digit = |upper| b"01230120022455012623010202"[usize::from(upper - b'A')];
    let mut previous = digit(initial);
    let mut output = String::with_capacity(4);
    output.push(char::from(initial));
    for byte in name {
        if matches!(byte, b'H' | b'W') {
            continue;
        }
        let code = digit(byte);
        if code != b'0' && previous != code {
            output.push(char::from(code));
            if output.len() == 4 {
                break;
            }
        }
        previous = code;
    }
    output.extend(std::iter::repeat_n('0', 4 - output.len()));
    output
}

// Codes suppressed from the output still update the previous code. This
// matters for equal consonant codes separated by a vowel or ignored H.
fn cologne(name: &str) -> String {
    let letters: Vec<u8> = name
        .to_uppercase()
        .chars()
        .map(|scalar| match scalar {
            'Ä' => b'A',
            'Ö' => b'O',
            'Ü' => b'U',
            letter => letter as u8,
        })
        .collect();
    let mut output = String::with_capacity(2 * letters.len());
    let mut prior_code = 0;
    let mut prior_letter = 0;
    for (index, &letter) in letters.iter().enumerate() {
        let next = letters.get(index + 1).copied().unwrap_or_default();
        let codes: &[u8] = match letter {
            b'A' | b'E' | b'I' | b'J' | b'O' | b'U' | b'Y' => b"0",
            b'B' => b"1",
            b'P' if next != b'H' => b"1",
            b'D' | b'T' if !b"CSZ".contains(&next) => b"2",
            b'F' | b'P' | b'V' | b'W' => b"3",
            b'G' | b'K' | b'Q' => b"4",
            b'X' if !b"CKQ".contains(&prior_letter) => b"48",
            b'C' if output.is_empty() && b"AHKLOQRUX".contains(&next) => b"4",
            b'C' if !output.is_empty()
                && !b"SZ".contains(&prior_letter)
                && b"AHKOQUX".contains(&next) =>
            {
                b"4"
            }
            b'D' | b'T' | b'X' | b'S' | b'Z' | b'C' => b"8",
            b'R' => b"7",
            b'L' => b"5",
            b'M' | b'N' => b"6",
            b'H' => b"\0",
            _ => unreachable!("checked Cologne alphabet"),
        };
        for &code in codes {
            if code != 0 && code != prior_code && (code != b'0' || output.is_empty()) {
                output.push(char::from(code));
            }
            prior_code = code;
        }
        prior_letter = letter;
    }
    output
}

// Input is already limited to the declared alphabet and scalar/byte bounds.
// Metaphone uses index-based char scans in the pinned implementation. Other
// encoders are fixed passes or bounded rule lookahead over a short ASCII name.
fn bound(method: Method, size: usize) -> usize {
    match method {
        Method::SoundexNara => 4 * size,
        Method::Nysiis => 64 * size + 128,
        Method::Metaphone => 12 * size * (size + 1) / 2 + 64 * size,
        Method::DoubleMetaphone => 384 * size + 64,
        Method::Caverphone1 | Method::Caverphone2 => 256 * (size + 16),
        Method::Cologne => 64 * (size + 1),
    }
}

fn encode_one(
    method: Method,
    strict: Option<bool>,
    max_code_length: Option<usize>,
    name: &str,
) -> Key {
    let key = match method {
        Method::SoundexNara => soundex(name),
        Method::Nysiis => Nysiis::new(strict.expect("resolved NYSIIS profile")).encode(name),
        Method::Metaphone => Metaphone::new(max_code_length).encode(name),
        Method::DoubleMetaphone => {
            let pair = DoubleMetaphone::new(max_code_length).double_metaphone(name);
            return Key::Double {
                primary: pair.primary(),
                alternate: pair.alternate(),
            };
        }
        Method::Caverphone1 => Caverphone1.encode(name),
        Method::Caverphone2 => Caverphone2.encode(name),
        Method::Cologne => cologne(name),
    };
    Key::Single { key }
}

fn encode_batch(request: &Request) -> Result<Report, OperationError> {
    let names = match (&request.name, &request.names) {
        (Some(name), None) => std::slice::from_ref(name),
        (None, Some(names)) => names.as_slice(),
        _ => return Err(parameter("provide exactly one of name or names")),
    };
    if names.len() > MAX_NAMES {
        return Err(resource("phonetic batch exceeds 256 names"));
    }
    let strict = if matches!(request.method, Method::Nysiis) {
        Some(request.strict.unwrap_or(true))
    } else if request.strict.is_some() {
        return Err(parameter("strict applies only to nysiis"));
    } else {
        None
    };
    let max_code_length = if matches!(request.method, Method::Metaphone | Method::DoubleMetaphone) {
        let selected = request.max_code_length.unwrap_or(4);
        if !(1..=32).contains(&selected) {
            return Err(parameter("max_code_length must be in 1..=32"));
        }
        Some(selected)
    } else if request.max_code_length.is_some() {
        return Err(parameter(
            "max_code_length applies only to Metaphone profiles",
        ));
    } else {
        None
    };
    let mut size = 0;
    let mut work = 0;
    let short = matches!(request.method, Method::SoundexNara | Method::Nysiis);
    for name in names {
        if matches!(request.method, Method::Cologne) {
            cologne_name(name)?
        } else {
            ascii_name(name)?
        }
        let one = bound(request.method, name.len());
        let ceiling = if short { 65_536 } else { 1 << 20 };
        if one > ceiling {
            return Err(resource("phonetic name exceeds its rule-visit limit"));
        }
        size += name.len();
        work += one;
        if size > MAX_BATCH_BYTES || work > MAX_BATCH_WORK {
            return Err(resource("phonetic batch exceeds its byte or work limit"));
        }
    }
    let keys = names
        .iter()
        .map(|name| encode_one(request.method, strict, max_code_length, name))
        .collect();
    let profile = if matches!(request.method, Method::SoundexNara) {
        "nara-census-four-character-v1"
    } else {
        APACHE_PROFILE
    };
    let report = Report {
        method: request.method,
        profile,
        strict,
        max_code_length,
        keys,
    };
    check_report(&report, 1 << 20)?;
    Ok(report)
}

/// Encode one name or a bounded ordered batch under one explicit rule profile.
pub fn encode(request: &Request) -> Result<Report, OperationError> {
    encode_batch(request).map_err(|error| error.operation(OP))
}
