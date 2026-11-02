//! RED Type A direct-wired sixes/twenties half-rotors and 47-pin breakwheel.
//! Independently implemented from the Rowlett description and J.F. Bouchaudy's
//! published 1937 reconstruction checks; no external simulator code is included.
use super::{alphabet::Alphabet, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "red_type_a_vigenere_v1")]
    TypeA,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Direct,
    Reverse,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[schemars(range(max = 46))]
    pub breakwheel: u8,
    #[schemars(range(max = 5))]
    pub sixes: u8,
    #[schemars(range(max = 19))]
    pub twenties: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub alphabet: String,
    /// Forty-seven physical pins in zero-based order; 1 present, 0 absent.
    #[schemars(regex(pattern = "^[01]{47}$"))]
    pub pins: String,
    pub direction: Direction,
    pub initial_state: State,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub state: State,
    #[schemars(range(min = 1, max = 47))]
    pub steps: u8,
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
fn advance(state: &mut State, pins: &[u8], direction: Direction) -> u8 {
    let delta = match direction {
        Direction::Direct => 1,
        Direction::Reverse => 46,
    };
    // A missing current pin is skipped; at least one present pin is validated.
    let mut steps = 1;
    while pins[usize::from(state.breakwheel)] == b'0' {
        state.breakwheel = (state.breakwheel + delta) % 47;
        steps += 1;
    }
    state.breakwheel = (state.breakwheel + delta) % 47;
    match direction {
        Direction::Direct => {
            state.sixes = (state.sixes + steps) % 6;
            state.twenties = (state.twenties + steps) % 20;
        }
        Direction::Reverse => {
            state.sixes = (state.sixes + 48 - steps) % 6;
            state.twenties = (state.twenties + 60 - steps) % 20;
        }
    }
    steps
}
/// Apply the qualified direct-wired RED circuit before breakwheel movement.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false, "cipher.red.encode")
}
/// Reverse the electrical circuit, retaining the selected direction of movement.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true, "cipher.red.decode")
}
fn transform(
    text: &str,
    p: &Settings,
    decode: bool,
    op: &'static str,
) -> Result<Report, OperationError> {
    bounded_text(text, p.trace, op)?;
    parameter_size(p, op)?;
    let alphabet = Alphabet::new(&p.alphabet, op, "alphabet")?;
    if p.pins.len() != 47
        || !p.pins.bytes().all(|x| x == b'0' || x == b'1')
        || !p.pins.contains('1')
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "pins",
            "use 47 binary pins with at least one present pin",
        ));
    }
    let mut state = p.initial_state.clone();
    if state.breakwheel >= 47 || state.sixes >= 6 || state.twenties >= 20 {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "initial_state",
            "wheel positions exceed their circumference",
        ));
    }
    let mut result = String::with_capacity(text.len());
    let mut trace = Vec::new();
    for (offset, c) in text.bytes().enumerate() {
        let x = alphabet.position(c).ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "prepared contacts require ASCII A-Z",
            )
            .offset(offset as u64)
        })?;
        let (base, width, position) = if x < 6 {
            (0, 6, state.sixes)
        } else {
            (6, 20, state.twenties)
        };
        let y = if decode {
            (x - base + position) % width
        } else {
            (x - base + width - position) % width
        };
        result.push(char::from(alphabet.letter(base + y)));
        let steps = advance(&mut state, p.pins.as_bytes(), p.direction);
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: state.clone(),
                steps,
            });
        }
    }
    finish_report(
        Report {
            text: result,
            final_state: state,
            trace,
        },
        op,
    )
}
