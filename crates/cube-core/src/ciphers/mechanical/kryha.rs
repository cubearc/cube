//! Precisely sourced 17-sector profile: Callimahos, Q.E.D.–2 Hours,41 Minutes,
//! NSA Technical Journal XVIII/4 (1973), pp. 20–21 and effective step table.
//! https://www.nsa.gov/portals/75/documents/news-features/declassified-documents/tech-journals/qed.pdf
//! The 52-stop successor and other reconstructions have different geometry.
use super::{alphabet::Alphabet, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const DISPLACEMENTS: [u8; 17] = [7, 6, 7, 5, 6, 7, 6, 8, 6, 10, 5, 6, 5, 7, 6, 5, 9];
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "kryha_callimahos_1973_v1")]
    Callimahos1973V1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[schemars(range(max = 25))]
    pub rotation: u8,
    ///Numbered sector 1..17 used for the next post-letter advance.
    #[schemars(range(min = 1, max = 17))]
    pub selector: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub plain_alphabet: String,
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
/// Encipher exact A-Z with the declared mixed alphabets and post-letter gear
/// schedule. The first letter uses initial_state before any advance.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false)
}
/// Decipher from the same initial state; return the state needed for continuation.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true)
}
fn transform(text: &str, p: &Settings, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.kryha.decode"
    } else {
        "cipher.kryha.encode"
    };
    parameter_size(p, op)?;
    bounded_text(text, p.trace, op)?;
    let plain = Alphabet::new(&p.plain_alphabet, op, "plain_alphabet")?;
    let cipher = Alphabet::new(&p.cipher_alphabet, op, "cipher_alphabet")?;
    let mut state = p.initial_state;
    if state.rotation > 25 || !(1..=17).contains(&state.selector) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "initial_state",
            "use rotation 0..25 and selector 1..17",
        ));
    }
    let mut r = Report {
        text: String::with_capacity(text.len()),
        final_state: state,
        trace: Vec::with_capacity(if p.trace { text.len() } else { 0 }),
    };
    for (offset, byte) in text.bytes().enumerate() {
        let position = if decode {
            cipher.position(byte)
        } else {
            plain.position(byte)
        }
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "use prepared ASCII A-Z only",
            )
            .offset(offset as u64)
        })?;
        let letter = if decode {
            plain.letter((position + 26 - state.rotation) % 26)
        } else {
            cipher.letter((position + state.rotation) % 26)
        };
        r.text.push(char::from(letter));
        state.rotation = (state.rotation + DISPLACEMENTS[usize::from(state.selector - 1)]) % 26;
        state.selector = state.selector % 17 + 1;
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
