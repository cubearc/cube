//! KL7 fixed reference rotor/notch set, raw36-contact positions and bounded re-entry.
//! Motion/data adapted from rmsk2, Copyright 2015-2018 Martin Grap (Apache-2.0).
//! See THIRD_PARTY_NOTICES; independent reference: Rijmenants KL7 simulator.
use super::{kl7_data as data, rotor_contacts::Wiring, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
const MOVING: [usize; 7] = [0, 1, 2, 4, 5, 6, 7];
const FEEDBACK: [u8; 10] = [2, 6, 10, 13, 16, 19, 23, 26, 30, 35];
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "kl7_rijmenants_v1")]
    Rijmenants,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Rotor {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
}
impl Rotor {
    fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
            Self::G => 6,
            Self::H => 7,
            Self::I => 8,
            Self::J => 9,
            Self::K => 10,
            Self::L => 11,
            Self::M => 12,
        }
    }
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct Position(#[schemars(range(max = 35))] u8);
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct RingId(#[schemars(range(min = 1, max = 11))] u8);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Raw core displacements0..35 in physical rotor order1..8; rotor4 is fixed.
    #[schemars(with = "[Position;8]")]
    pub core_positions: [u8; 8],
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    pub rotors: [Rotor; 8],
    /// Distinct ring identities1..11 on moving slots1,2,3,5,6,7,8.
    #[schemars(with = "[RingId;7]")]
    pub notch_rings: [u8; 7],
    /// Raw notch offsets relative to each core, before the fixed +10 sensing arm.
    #[schemars(with = "[Position;7]")]
    pub notch_offsets: [u8; 7],
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
    #[schemars(range(min = 1, max = 11))]
    pub feedback_passes: u8,
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
struct Notch {
    id: RingId,
    offset: Position,
}
struct Machine {
    rotors: [Wiring<36>; 8],
    positions: [Position; 8],
    rings: [Notch; 7],
    base: Wiring<36>,
}
impl Machine {
    fn configure(p: &Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        let mut rotors = 0u16;
        for r in p.rotors {
            let bit = 1 << r.index();
            if rotors & bit != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "rotors",
                    "select eight distinct reference rotors A-M",
                ));
            }
            rotors |= bit;
        }
        let mut rings = 0u16;
        for id in p.notch_rings {
            if !(1..=11).contains(&id) || rings & (1 << id) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "notch_rings",
                    "select seven distinct reference rings1..11",
                ));
            }
            rings |= 1 << id;
        }
        if p.initial_state
            .core_positions
            .iter()
            .chain(&p.notch_offsets)
            .any(|x| *x >= 36)
        {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "positions",
                "raw core positions and notch offsets must be0..35",
            ));
        }
        Ok(Self {
            rotors: std::array::from_fn(|i| Wiring::new(&data::ROTORS[p.rotors[i].index()], false)),
            positions: p.initial_state.core_positions.map(Position),
            rings: std::array::from_fn(|i| Notch {
                id: RingId(p.notch_rings[i]),
                offset: Position(p.notch_offsets[i]),
            }),
            base: Wiring::new(&data::BASE, false),
        })
    }
    fn state(&self) -> State {
        State {
            core_positions: self.positions.map(|p| p.0),
        }
    }
    fn advance(&mut self) {
        let mut active = [false; 8];
        for (slot, ring) in MOVING.into_iter().zip(self.rings) {
            let at = (self.positions[slot].0 + ring.offset.0 + 10) % 36;
            active[slot] = data::NOTCHES[usize::from(ring.id.0 - 1)][usize::from(at)] != 0;
        }
        let a = active;
        let common = !a[1] || !a[2];
        let selected = [
            !a[7] && common,
            !a[5] || !a[6],
            a[1] || a[6],
            false,
            common,
            !a[0] || a[2],
            a[0] || a[5],
            !a[4] && common,
        ];
        for (p, step) in self.positions.iter_mut().zip(selected) {
            if step {
                p.0 = (p.0 + 1) % 36;
            }
        }
    }
    fn letter(
        &self,
        input: u8,
        decode: bool,
        op: &'static str,
    ) -> Result<(u8, u8), OperationError> {
        let mut value = self.base.backward(input, 0);
        // Ten internal contacts loop back to themselves. Since every complete
        // bank is a permutation, an external input exits in at most11 passes.
        for passes in 1..=11 {
            value = if decode {
                self.rotors
                    .iter()
                    .zip(self.positions)
                    .rev()
                    .fold(value, |x, (w, p)| w.backward(x, p.0))
            } else {
                self.rotors
                    .iter()
                    .zip(self.positions)
                    .fold(value, |x, (w, p)| w.forward(x, p.0))
            };
            if !FEEDBACK.contains(&value) {
                return Ok((self.base.forward(value, 0), passes));
            }
        }
        Err(error(
            op,
            ErrorCode::ResourceLimit,
            "feedback",
            "re-entry exceeds the eleven-pass contact bound",
        ))
    }
}
/// Encipher prepared A-Z keyboard contact symbols from explicit raw core state.
/// No automatic letter/figure shifts or preparatory stepping are performed.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false, "cipher.kl7.encode")
}
/// Reverse the complete re-entry path, using identical old-notch post-stepping.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true, "cipher.kl7.decode")
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
        let (x, feedback_passes) = m.letter(c - b'A', decode, op)?;
        result.push(char::from(b'A' + x));
        m.advance();
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: m.state(),
                feedback_passes,
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
