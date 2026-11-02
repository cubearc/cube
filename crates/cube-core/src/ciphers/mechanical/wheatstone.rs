//! Wheatstone 1879's original 27/26-cell clock cryptograph, reproduced in
//! Friedman, Several Machine Ciphers (1918), printed pages 7–10.
use super::{alphabet::Alphabet, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "wheatstone_1879_v1")]
    Wheatstone1879V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    ///0 is the word separator +; 1..26 are A..Z on the fixed outer ring.
    #[schemars(range(max = 26))]
    pub plain_position: u8,
    #[schemars(range(max = 25))]
    pub cipher_position: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub cipher_alphabet: String,
    pub initial_state: State,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub state: State,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))]
    pub text: String,
    pub final_state: State,
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
/// Encipher prepared A-Z/+ with strictly positive clockwise motion. Consecutive
/// equal plain symbols (including the initial pointer) fail rather than silently
/// inserting or dropping a letter. No preparation or sentence resets are guessed.
pub fn encode(text: &str, settings: &Settings) -> Result<Report, OperationError> {
    transform(text, settings, false)
}
/// Invert using the same initial pointers/alphabet; a repeated cipher symbol
/// represents 26 advances. The result is prepared text, with + word separators.
pub fn decode(text: &str, settings: &Settings) -> Result<Report, OperationError> {
    transform(text, settings, true)
}
fn transform(text: &str, p: &Settings, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.wheatstone.decode"
    } else {
        "cipher.wheatstone.encode"
    };
    parameter_size(p, op)?;
    bounded_text(text, p.trace, op)?;
    let alphabet = Alphabet::new(&p.cipher_alphabet, op, "cipher_alphabet")?;
    let mut state = p.initial_state;
    if state.plain_position > 26 || state.cipher_position > 25 {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "initial_state",
            "pointer positions outside 27/26-cell rings",
        ));
    }
    let mut r = Report {
        text: String::with_capacity(text.len()),
        final_state: state,
        trace: Vec::with_capacity(if p.trace { text.len() } else { 0 }),
    };
    for (offset, byte) in text.bytes().enumerate() {
        let symbol = if decode {
            decipher(byte, &mut state, alphabet)
        } else {
            encipher(byte, &mut state, alphabet)
        }
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "use prepared ring symbols; no repeated plain symbol",
            )
            .offset(offset as u64)
        })?;
        r.text.push(char::from(symbol));
        if p.trace {
            r.trace.push(Trace {
                offset: offset as u32,
                state,
            });
        }
    }
    r.final_state = state;
    finish_report(r, op)
}
fn encipher(byte: u8, state: &mut State, alphabet: Alphabet) -> Option<u8> {
    let target = if byte == b'+' {
        0
    } else if byte.is_ascii_uppercase() {
        byte - b'A' + 1
    } else {
        return None;
    };
    let step = (target + 27 - state.plain_position) % 27;
    if step == 0 {
        return None;
    }
    state.plain_position = target;
    state.cipher_position = (state.cipher_position + step) % 26;
    Some(alphabet.letter(state.cipher_position))
}
fn decipher(byte: u8, state: &mut State, alphabet: Alphabet) -> Option<u8> {
    let target = alphabet.position(byte)?;
    let step = (target + 26 - state.cipher_position) % 26;
    state.cipher_position = target;
    state.plain_position = (state.plain_position + if step == 0 { 26 } else { step }) % 27;
    Some(if state.plain_position == 0 {
        b'+'
    } else {
        b'A' + state.plain_position - 1
    })
}
