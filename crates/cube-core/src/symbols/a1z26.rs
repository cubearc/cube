//! Strict ASCII alphabet/number notation. Whitespace is data, not a separator guess.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::Descriptor,
    text,
};
pub const MAX_ENCODED: usize = 3 * text::MAX_INPUT;
fn whitespace(byte: u8) -> bool {
    byte == b' ' || (b'\t'..=b'\r').contains(&byte)
}
fn invalid(message: &str, op: &'static str, offset: usize) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
        .operation(op)
        .offset(offset as u64)
}
/// Encode ASCII letters as canonical 1..26 decimal tokens separated by hyphens.
/// ASCII whitespace is preserved byte-for-byte and resets the word. Lowercase
/// canonicalizes to uppercase on decoding. Punctuation/non-ASCII fail. <=1 MiB
/// input, <=3 MiB output (and the shared 8 MiB ceiling); empty input is valid.
pub fn encode(input: &str) -> Result<String, OperationError> {
    const OP: &str = "symbol.a1z26.encode";
    text::check_input(input, OP)?;
    let mut output = String::new();
    let mut previous = false;
    for (offset, byte) in input.bytes().enumerate() {
        if byte.is_ascii_alphabetic() {
            if previous {
                output.push('-');
            }
            let n = byte.to_ascii_uppercase() - b'A' + 1;
            if n >= 10 {
                output.push(char::from(b'0' + n / 10));
            }
            output.push(char::from(b'0' + n % 10));
            previous = true;
        } else if whitespace(byte) {
            output.push(char::from(byte));
            previous = false;
        } else {
            return Err(invalid(
                "A1Z26 accepts only ASCII letters and ASCII whitespace",
                OP,
                offset,
            ));
        }
    }
    Ok(output)
}
/// Decode canonical hyphen-separated decimal tokens into uppercase ASCII.
/// Tokens must be exactly 1..26 with no sign or leading zero. ASCII whitespace
/// is preserved exactly; repeated/leading/trailing hyphens and other bytes fail.
pub fn decode(input: &str) -> Result<String, OperationError> {
    const OP: &str = "symbol.a1z26.decode";
    if input.len() > MAX_ENCODED {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "A1Z26 encoded input exceeds 3 MiB",
        )
        .operation(OP));
    }
    let mut output = String::new();
    let mut offset = 0;
    let bytes = input.as_bytes();
    while offset < bytes.len() {
        if whitespace(bytes[offset]) {
            output.push(char::from(bytes[offset]));
            offset += 1;
            continue;
        }
        let start = offset;
        while offset < bytes.len() && !whitespace(bytes[offset]) {
            offset += 1;
        }
        let mut token_offset = start;
        for token in input[start..offset].split('-') {
            if token.is_empty()
                || token.len() > 2
                || token.starts_with('0')
                || !token.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(invalid(
                    "expected canonical A1Z26 numbers 1..26 separated by hyphens",
                    OP,
                    token_offset,
                ));
            }
            let n = token.bytes().fold(0u8, |n, b| n * 10 + b - b'0');
            if !(1..=26).contains(&n) {
                return Err(invalid("A1Z26 number must be in 1..=26", OP, token_offset));
            }
            output.push(char::from(b'A' + n - 1));
            token_offset += token.len() + 1;
        }
    }
    Ok(output)
}
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "symbol.a1z26.encode",
        category: "symbols",
        description: "ASCII A-Z to canonical 1..26 hyphen tokens; preserve ASCII whitespace; lowercase loses case; <=1 MiB input/3 MiB output.",
        input_type: "text",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "encode a1z26",
        example: "cube encode a1z26 --text 'Hi Cube'",
    },
    Descriptor {
        id: "symbol.a1z26.decode",
        category: "symbols",
        description: "Strict canonical 1..26 hyphen tokens to uppercase ASCII; preserve ASCII whitespace, reject malformed tokens; <=3 MiB.",
        input_type: "text",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "decode a1z26",
        example: "cube decode a1z26 --text '8-9 3-21-2-5'",
    },
];
