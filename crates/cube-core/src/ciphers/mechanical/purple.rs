//! Type B/PURPLE stepping switches. Tables adapted from Brian Neal's MIT-licensed
//! reconstruction of Freeman, Sullivan and Weierud (2003); see THIRD_PARTY_NOTICES.
use super::{alphabet::Alphabet, purple_data as data, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "purple_type_b_v1")]
    TypeB,
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct Position(#[schemars(range(max = 24))] u8);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Zero-based sixes, twenties 1, twenties 2, twenties 3 positions.
    #[schemars(with = "[Position;4]")]
    pub switch_positions: [u8; 4],
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    /// Six input/output letters followed by the twenty remaining letters.
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub alphabet: String,
    #[schemars(range(min = 1, max = 3))]
    pub fast_switch: u8,
    #[schemars(range(min = 1, max = 3))]
    pub middle_switch: u8,
    pub initial_state: State,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub state: State,
    /// Physical twenties switch 1, 2 or 3 which advanced after this symbol.
    #[schemars(range(min = 1, max = 3))]
    pub advanced_switch: u8,
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
fn contact(row: &[u8], value: u8, decode: bool) -> u8 {
    if decode {
        row[usize::from(value)]
    } else {
        row.iter()
            .position(|x| *x == value)
            .expect("qualified permutation") as u8
    }
}
fn advance(p: &mut [Position; 4], fast: u8, middle: u8) -> u8 {
    let motor = if p[0].0 == 23 && p[usize::from(middle)].0 == 24 {
        6 - fast - middle
    } else if p[0].0 == 24 {
        middle
    } else {
        fast
    };
    p[0].0 = (p[0].0 + 1) % 25;
    p[usize::from(motor)].0 = (p[usize::from(motor)].0 + 1) % 25;
    motor
}
/// Encipher prepared uppercase contacts with all four switch positions explicit.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false, "cipher.purple.encode")
}
/// Reverse the switch path while preserving the same post-symbol advancement.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true, "cipher.purple.decode")
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
    if !(1..=3).contains(&p.fast_switch)
        || !(1..=3).contains(&p.middle_switch)
        || p.fast_switch == p.middle_switch
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "switches",
            "select distinct fast and middle switches 1..3",
        ));
    }
    let mut positions = p.initial_state.switch_positions.map(Position);
    if positions.iter().any(|x| x.0 >= 25) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "initial_state",
            "switch positions must be 0..24",
        ));
    }
    let mut output = String::with_capacity(text.len());
    let mut trace = Vec::new();
    for (offset, c) in text.bytes().enumerate() {
        let mut x = alphabet.position(c).ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "prepared contacts require ASCII A-Z",
            )
            .offset(offset as u64)
        })?;
        if x < 6 {
            x = contact(&data::SIXES[usize::from(positions[0].0)], x, decode);
        } else {
            x -= 6;
            for k in 0..3 {
                let slot = if decode { 2 - k } else { k };
                x = contact(
                    &data::TWENTIES[slot][usize::from(positions[slot + 1].0)],
                    x,
                    decode,
                );
            }
            x += 6;
        }
        output.push(char::from(alphabet.letter(x)));
        let advanced_switch = advance(&mut positions, p.fast_switch, p.middle_switch);
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: State {
                    switch_positions: positions.map(|x| x.0),
                },
                advanced_switch,
            });
        }
    }
    finish_report(
        Report {
            text: output,
            final_state: State {
                switch_positions: positions.map(|x| x.0),
            },
            trace,
        },
        op,
    )
}
