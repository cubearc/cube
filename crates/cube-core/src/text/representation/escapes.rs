//! JSON string content and HTML text-context character references.
use super::{
    HTML_DECODE, HTML_ENCODE, JSON_DECODE, JSON_ENCODE, MAX_ESCAPED, MAX_TEXT, check, invalid,
};
use crate::errors::OperationError;
use std::fmt::Write as _;
use web_atoms::{C1_REPLACEMENTS, NAMED_ENTITIES};

/// Encode RFC 8259 JSON string content without quotes. ASCII-only output
/// escapes non-ASCII characters and represents supplementary scalars as pairs.
pub fn json_escape_encode(text: &str, ascii_only: bool) -> Result<String, OperationError> {
    check(text.len(), MAX_TEXT, JSON_ENCODE)?;
    let mut output = String::with_capacity(text.len());
    for scalar in text.chars() {
        match scalar {
            '"' => output.push_str(r#"\""#),
            '\\' => output.push_str(r"\\"),
            '\u{08}' => output.push_str(r"\b"),
            '\u{0c}' => output.push_str(r"\f"),
            '\n' => output.push_str(r"\n"),
            '\r' => output.push_str(r"\r"),
            '\t' => output.push_str(r"\t"),
            scalar if scalar < ' ' || (ascii_only && !scalar.is_ascii()) => {
                for word in scalar.encode_utf16(&mut [0; 2]) {
                    write!(output, "\\u{word:04X}").expect("infallible string formatting");
                }
            }
            scalar => output.push(scalar),
        }
    }
    Ok(output)
}

/// Decode RFC 8259 string content, without surrounding quotes. Malformed
/// escapes, unescaped quotes and controls, and unpaired surrogates fail.
pub fn json_escape_decode(input: &str) -> Result<String, OperationError> {
    check(input.len(), MAX_ESCAPED, JSON_DECODE)?;
    let wrapped = format!("\"{input}\"");
    let decoded: String = serde_json::from_str(&wrapped).map_err(|error| {
        let earlier_lines = wrapped
            .split_inclusive('\n')
            .take(error.line().saturating_sub(1))
            .map(str::len)
            .sum::<usize>();
        let offset = (earlier_lines + error.column())
            .saturating_sub(2)
            .min(input.len());
        invalid(JSON_DECODE, offset, "invalid JSON string content")
    })?;
    check(decoded.len(), MAX_TEXT, JSON_DECODE)?;
    Ok(decoded)
}

/// Encode five markup-sensitive scalars as canonical hexadecimal references.
/// Other scalars pass through unchanged.
pub fn html_entity_encode(text: &str) -> Result<String, OperationError> {
    check(text.len(), MAX_TEXT, HTML_ENCODE)?;
    let mut output = String::with_capacity(text.len());
    for scalar in text.chars() {
        if matches!(scalar, '&' | '<' | '>' | '"' | '\'') {
            write!(output, "&#x{:X};", u32::from(scalar)).expect("infallible string formatting");
        } else {
            output.push(scalar);
        }
    }
    Ok(output)
}

fn named(input: &str) -> Option<(usize, u32, u32)> {
    // The upstream map includes incomplete prefixes as (0, 0).
    let mut found = None;
    for (index, byte) in input.bytes().take(32).enumerate() {
        if !byte.is_ascii_alphanumeric() && byte != b';' {
            break;
        }
        let n = index + 1;
        let Some(&(first, second)) = NAMED_ENTITIES.get(&input[..n]) else {
            break;
        };
        if first != 0 {
            found = Some((n, first, second));
        }
        if byte == b';' {
            break;
        }
    }
    found
}

fn numeric(text: &str) -> Option<(usize, char)> {
    let rest = text.strip_prefix('#')?;
    let (rest, radix, prefix) = match rest.strip_prefix(['x', 'X']) {
        Some(rest) => (rest, 16, 2),
        None => (rest, 10, 1),
    };
    let mut end = 0;
    let mut value = 0u32;
    for byte in rest.bytes() {
        let Some(digit) = char::from(byte).to_digit(radix) else {
            break;
        };
        value = value.saturating_mul(radix).saturating_add(digit);
        end += 1;
    }
    if end == 0 {
        return None;
    }
    let used = prefix + end + usize::from(rest.as_bytes().get(end) == Some(&b';'));
    let scalar = match value {
        0 => '\u{fffd}',
        0x80..=0x9f => C1_REPLACEMENTS[(value - 0x80) as usize]
            .unwrap_or_else(|| char::from_u32(value).expect("C1 scalar")),
        value => char::from_u32(value).unwrap_or('\u{fffd}'),
    };
    Some((used, scalar))
}

/// Decode HTML character references in the text context. Unrecognized names
/// pass through literally. Numeric references follow WHATWG replacement rules.
/// Markup and original line endings are passed through as text.
pub fn html_entity_decode(text: &str) -> Result<String, OperationError> {
    check(text.len(), MAX_ESCAPED, HTML_DECODE)?;
    let mut output = String::with_capacity(text.len().min(MAX_TEXT));
    let mut rest = text;
    while let Some(index) = rest.find('&') {
        check(output.len() + index, MAX_TEXT, HTML_DECODE)?;
        output.push_str(&rest[..index]);
        rest = &rest[index + 1..];
        if let Some((used, scalar)) = numeric(rest) {
            output.push(scalar);
            rest = &rest[used..];
        } else if let Some((used, first, second)) = named(rest) {
            output.push(char::from_u32(first).expect("HTML table contains scalars"));
            if second != 0 {
                output.push(char::from_u32(second).expect("HTML table contains scalars"));
            }
            rest = &rest[used..];
        } else {
            output.push('&');
        }
        check(output.len(), MAX_TEXT, HTML_DECODE)?;
    }
    check(output.len() + rest.len(), MAX_TEXT, HTML_DECODE)?;
    output.push_str(rest);
    Ok(output)
}
