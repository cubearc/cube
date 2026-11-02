//! CSP889 cipher/control/index banks with complete physical windows.
//! Wiring and control equations qualified against CSG and rmsk2 (Apache-2.0),
//! Copyright 2015-2018 Martin Grap; see THIRD_PARTY_NOTICES.
use super::{rotor_contacts::Wiring, sigaba_data as data, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "sigaba_csp889_v1")]
    Csp889,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rotor {
    /// Reference-set identity0..9; all ten cipher/control rotors must be distinct.
    #[schemars(range(max = 9))]
    pub id: u8,
    pub reversed: bool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexRotor {
    #[schemars(range(max = 4))]
    pub id: u8,
    pub reversed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[schemars(regex(pattern = "^[A-Z]{5}$"))]
    pub cipher_windows: String,
    /// Physical order: left stator, slow, fast, middle, right stator.
    #[schemars(regex(pattern = "^[A-Z]{5}$"))]
    pub control_windows: String,
    #[schemars(regex(pattern = "^[0-9]{5}$"))]
    pub index_windows: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    pub cipher_rotors: [Rotor; 5],
    pub control_rotors: [Rotor; 5],
    pub index_rotors: [IndexRotor; 5],
    pub initial_state: State,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    /// State after the current symbol and all simultaneous post-step motion.
    pub state: State,
    #[schemars(range(max = 31))]
    pub stepped_mask: u8,
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
#[derive(Clone, Copy)]
struct Wheel<const N: usize> {
    wiring: Wiring<N>,
    position: u8,
    reversed: bool,
}
impl<const N: usize> Wheel<N> {
    fn new(source: &[u8; N], reversed: bool, window: u8) -> Self {
        Self {
            wiring: Wiring::new(source, reversed),
            position: if reversed {
                ((N - usize::from(window)) % N) as u8
            } else {
                window
            },
            reversed,
        }
    }
    fn window(&self) -> u8 {
        if self.reversed {
            ((N - usize::from(self.position)) % N) as u8
        } else {
            self.position
        }
    }
    fn step_back(&mut self) {
        self.position = ((usize::from(self.position) + N - 1) % N) as u8;
    }
    fn forward(&self, x: u8) -> u8 {
        self.wiring.forward(x, self.position)
    }
    fn backward(&self, x: u8) -> u8 {
        self.wiring.backward(x, self.position)
    }
}
fn windows(
    s: &str,
    base: u8,
    size: u8,
    op: &'static str,
    field: &'static str,
) -> Result<[u8; 5], OperationError> {
    if s.len() != 5 || !s.bytes().all(|x| x >= base && x < base + size) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            field,
            "supply all five exact ASCII bank window marks",
        ));
    }
    Ok(std::array::from_fn(|i| s.as_bytes()[i] - base))
}
struct Machine {
    cipher: [Wheel<26>; 5],
    control: [Wheel<26>; 5],
    index: [Wheel<10>; 5],
    index_outputs: [u8; 10],
}
impl Machine {
    fn configure(p: &Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        let mut seen = 0u16;
        for r in p.cipher_rotors.iter().chain(&p.control_rotors) {
            if r.id > 9 || seen & (1 << r.id) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "rotors",
                    "use each cipher/control reference rotor0..9 exactly once",
                ));
            }
            seen |= 1 << r.id;
        }
        let mut seen = 0u8;
        for r in p.index_rotors {
            if r.id > 4 || seen & (1 << r.id) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "index_rotors",
                    "use each index reference rotor0..4 exactly once",
                ));
            }
            seen |= 1 << r.id;
        }
        let c = windows(
            &p.initial_state.cipher_windows,
            b'A',
            26,
            op,
            "cipher_windows",
        )?;
        let s = windows(
            &p.initial_state.control_windows,
            b'A',
            26,
            op,
            "control_windows",
        )?;
        let i = windows(
            &p.initial_state.index_windows,
            b'0',
            10,
            op,
            "index_windows",
        )?;
        let cipher = std::array::from_fn(|i| {
            Wheel::new(
                &data::ROTORS[usize::from(p.cipher_rotors[i].id)],
                p.cipher_rotors[i].reversed,
                c[i],
            )
        });
        let control = std::array::from_fn(|i| {
            Wheel::new(
                &data::ROTORS[usize::from(p.control_rotors[i].id)],
                p.control_rotors[i].reversed,
                s[i],
            )
        });
        let index: [Wheel<10>; 5] = std::array::from_fn(|j| {
            Wheel::new(
                &data::INDEX[usize::from(p.index_rotors[j].id)],
                p.index_rotors[j].reversed,
                i[j],
            )
        });
        let index_outputs =
            std::array::from_fn(|i| index.iter().fold(i as u8, |x, w| w.forward(x)));
        Ok(Self {
            cipher,
            control,
            index,
            index_outputs,
        })
    }
    fn state(&self) -> State {
        State {
            cipher_windows: self
                .cipher
                .iter()
                .map(|w| char::from(b'A' + w.window()))
                .collect(),
            control_windows: self
                .control
                .iter()
                .map(|w| char::from(b'A' + w.window()))
                .collect(),
            index_windows: self
                .index
                .iter()
                .map(|w| char::from(b'0' + w.window()))
                .collect(),
        }
    }
    fn letter(&self, input: u8, decode: bool) -> u8 {
        if decode {
            self.cipher.iter().rev().fold(input, |x, w| w.backward(x))
        } else {
            self.cipher.iter().fold(input, |x, w| w.forward(x))
        }
    }
    fn advance(&mut self) -> u8 {
        const GROUP: [usize; 26] = [
            9, 1, 2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8,
        ];
        const MOTOR: [u8; 10] = [0, 4, 4, 3, 3, 2, 2, 1, 1, 0];
        let mut selected = 0u8;
        // All four energized currents use the old control/index positions.
        for input in 5..9 {
            let x = self.control.iter().rev().fold(input, |x, w| w.backward(x));
            let output = self.index_outputs[GROUP[usize::from(x)]];
            selected |= 1 << MOTOR[usize::from(output)];
        }
        let middle = self.control[2].window() == 14;
        let slow = middle && self.control[3].window() == 14;
        self.control[2].step_back();
        if middle {
            self.control[3].step_back();
        }
        if slow {
            self.control[1].step_back();
        }
        for (i, w) in self.cipher.iter_mut().enumerate() {
            if selected & (1 << i) != 0 {
                w.step_back();
            }
        }
        selected
    }
}
/// Encipher prepared A-Z contact symbols with complete CSP889 banks/windows.
/// Physical plaintext space is contact Z; preparation is deliberately explicit.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false, "cipher.sigaba.encode")
}
/// Decipher with the inverse cipher bank and identical post-step control path.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true, "cipher.sigaba.decode")
}
fn transform(
    text: &str,
    p: &Settings,
    decode: bool,
    op: &'static str,
) -> Result<Report, OperationError> {
    bounded_text(text, p.trace, op)?;
    let mut m = Machine::configure(p, op)?;
    let mut result = String::with_capacity(text.len());
    let mut trace = Vec::new();
    for (offset, c) in text.bytes().enumerate() {
        if !c.is_ascii_uppercase() {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "prepared contacts must contain only ASCII A-Z",
            )
            .offset(offset as u64));
        }
        result.push(char::from(b'A' + m.letter(c - b'A', decode)));
        let stepped_mask = m.advance();
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: m.state(),
                stepped_mask,
            });
        }
    }
    finish_report(
        Report {
            text: result,
            final_state: m.state(),
            trace,
        },
        op,
    )
}
