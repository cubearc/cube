//! Letter-only flag Semaphore. Positions are viewed facing the sender;
//! LH/RH identify the sender's anatomical left/right arms, not screen order.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::Descriptor,
};
pub const DIRECTIONS: &[&str] = &["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
pub const MAX_SYMBOLS: usize = 16384;
// Factual positions checked against the Marine Corps Museum and Scout Association
// charts. ANBG's textual I entry says RH up; both charts show RH high (NW).
// No artwork copied. D's documented alternate assignment is accepted separately.
// https://www.cpbr.gov.au/flags/semaphore.html
// https://www.usmcmuseum.com/uploads/6/0/3/6/60364049/nmmc_semaphore_flag_booklet_final_1.pdf
// https://prod-cms.scouts.org.uk/media/5508/generic-resources-semaphore-signals.pdf
const ALPHABET: [(&str, &str); 26] = [
    ("S", "SW"),
    ("S", "W"),
    ("S", "NW"),
    ("S", "N"),
    ("NE", "S"),
    ("E", "S"),
    ("SE", "S"),
    ("SW", "W"),
    ("SW", "NW"),
    ("E", "N"),
    ("N", "SW"),
    ("NE", "SW"),
    ("E", "SW"),
    ("SE", "SW"),
    ("NW", "W"),
    ("N", "W"),
    ("NE", "W"),
    ("E", "W"),
    ("SE", "W"),
    ("N", "NW"),
    ("NE", "NW"),
    ("SE", "N"),
    ("E", "NE"),
    ("SE", "NE"),
    ("E", "NW"),
    ("E", "SE"),
];
fn error(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}
/// Decode one anatomical LH/RH pair. D permits the documented alternate arm
/// assignment N,S; other reversed assignments are not silently accepted.
/// Numeral/annul/control signals are excluded; J is always the literal letter J.
pub fn decode_pair(left: &str, right: &str) -> Result<char, OperationError> {
    let result = (|| {
        if !DIRECTIONS.contains(&left) || !DIRECTIONS.contains(&right) || left == right {
            return Err(error(
                "pair needs two distinct exact directions: N,NE,E,SE,S,SW,W,NW",
            ));
        }
        if (left, right) == ("N", "S") {
            return Ok('D');
        }
        ALPHABET
            .iter()
            .position(|pair| *pair == (left, right))
            .map(|i| char::from(b'A' + i as u8))
            .ok_or_else(|| {
                error("unsupported arm pair; controls and numeric mode are not supported")
            })
    })();
    result.map_err(|e: OperationError| e.operation("symbol.semaphore.decode"))
}
/// Encode ASCII letters and literal ASCII spaces. Letters canonicalize to upper
/// case; each space becomes `/`. Tokens are `LH,RH`, separated by exactly one
/// ASCII space. Empty text is empty output. No other whitespace or controls.
/// Input <=1 MiB and <=16384 symbols; encoded output <96 KiB.
pub fn encode(input: &str) -> Result<String, OperationError> {
    const ID: &str = "symbol.semaphore.encode";
    crate::text::check_input(input, ID)?;
    if input.len() > MAX_SYMBOLS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "Semaphore input exceeds 16384 ASCII symbols",
        )
        .operation(ID));
    }
    let mut output = Vec::with_capacity(input.len());
    for c in input.bytes() {
        if c == b' ' {
            output.push("/".to_owned());
        } else if c.is_ascii_alphabetic() {
            let (left, right) = ALPHABET[(c.to_ascii_uppercase() - b'A') as usize];
            output.push(format!("{left},{right}"));
        } else {
            return Err(
                error("Semaphore accepts only ASCII letters and literal spaces").operation(ID),
            );
        }
    }
    Ok(output.join(" "))
}
/// Decode exact space-separated `LH,RH` tokens or `/` spaces, preserving repeated
/// spaces as repeated `/` tokens. Reject empty tokens, extra delimiters, unknown
/// directions, swapped arms and unsupported control state. <=16384 tokens.
pub fn decode(input: &str) -> Result<String, OperationError> {
    const ID: &str = "symbol.semaphore.decode";
    crate::text::check_input(input, ID)?;
    if input.is_empty() {
        return Ok(String::new());
    }
    let mut output = String::new();
    for (i, token) in input.split(' ').enumerate() {
        if i == MAX_SYMBOLS {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "Semaphore token count exceeds 16384",
            )
            .operation(ID));
        }
        if token == "/" {
            output.push(' ');
            continue;
        }
        let (left, right) = token
            .split_once(',')
            .ok_or_else(|| error("expected LH,RH pair or / token").operation(ID))?;
        output.push(decode_pair(left, right)?);
    }
    Ok(output)
}
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "symbol.semaphore.encode",
        category: "symbols",
        description: "Letter-only flag Semaphore: sender anatomical LH,RH, receiver-facing N/NE/E/SE/S/SW/W/NW; ASCII letters uppercase, each space=/, <=16384 symbols; no numeric/control state.",
        input_type: "text",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "encode semaphore",
        example: "cube encode semaphore --text 'A B'",
    },
    Descriptor {
        id: "symbol.semaphore.decode",
        category: "symbols",
        description: "Decode exact LH,RH tokens or /, separated by one ASCII space; <=16384 tokens, literal J, alternate D accepted; no guessing or visual recognition.",
        input_type: "text",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(false),
        command: "decode semaphore",
        example: "cube decode semaphore --text 'S,SW / S,W'",
    },
];
