//! M-125, 6K PROTON I, thirty-contact mode. Contact facts and motion adapted
//! from CrypTool 2, Copyright 2016 Eugen Antal, FEI STU Bratislava (Apache-2.0).
//! See THIRD_PARTY_NOTICES. Keyboard language/printing modes are not inferred.
pub use super::lorenz::parse_codes;
use super::{fialka_data as data, rotor_contacts::Wiring, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "fialka_m125_6k_proton_i_v1")]
    M125,
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
pub(super) struct Contact(#[schemars(range(max = 29))] u8);
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct RotorId(#[schemars(range(min = 1, max = 10))] u8);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Raw rotor offsets, physical left (reflector) to right (entry disk).
    #[schemars(with = "[Contact;10]")]
    pub rotor_positions: [u8; 10],
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    #[schemars(with = "[RotorId;10]")]
    pub rotor_order: [u8; 10],
    #[schemars(with = "[Contact;10]")]
    pub ring_offsets: [u8; 10],
    /// Complete permutation of thirty punched-card input contacts.
    #[schemars(with = "[Contact;30]")]
    pub punch_card: [u8; 30],
    pub initial_state: State,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub state: State,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(with = "Vec<Contact>", length(max = 1048576))]
    pub codes: Vec<u8>,
    pub final_state: State,
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
struct Machine {
    rotors: [Wiring<30>; 10],
    ids: [RotorId; 10],
    rings: [Contact; 10],
    state: State,
    card: Wiring<30>,
    keyboard: Wiring<30>,
    entry: Wiring<30>,
    reflector: Wiring<30>,
}
impl Machine {
    fn new(p: &Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        let mut seen = 0u16;
        for id in p.rotor_order {
            if !(1..=10).contains(&id) || seen & (1 << id) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "rotor_order",
                    "use each rotor ID 1..10 once",
                ));
            }
            seen |= 1 << id;
        }
        if p.ring_offsets
            .iter()
            .chain(&p.initial_state.rotor_positions)
            .any(|x| *x >= 30)
        {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "positions",
                "ring and rotor positions must be 0..29",
            ));
        }
        let mut contacts = 0u32;
        for x in p.punch_card {
            if x >= 30 || contacts & (1 << x) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "punch_card",
                    "use each contact 0..29 once",
                ));
            }
            contacts |= 1 << x;
        }
        let ids = p.rotor_order.map(RotorId);
        Ok(Self {
            rotors: ids.map(|i| Wiring::new(&data::ROTORS[usize::from(i.0 - 1)], false)),
            ids,
            rings: p.ring_offsets.map(Contact),
            state: p.initial_state.clone(),
            card: Wiring::new(&p.punch_card, false),
            keyboard: Wiring::new(&data::KEYBOARD, false),
            entry: Wiring::new(&data::ENTRY, false),
            reflector: Wiring::new(&data::REFLECTOR, false),
        })
    }
    fn letter(&self, input: u8, decode: bool) -> u8 {
        let mut x = (self
            .entry
            .forward(self.card.forward(self.keyboard.forward(input, 0), 0), 0)
            + 3)
            % 30;
        for i in (0..10).rev() {
            x = self.rotors[i].forward(x, (self.state.rotor_positions[i] + self.rings[i].0) % 30);
        }
        x = (x + 27) % 30;
        x = if decode {
            self.reflector.backward(x, 0)
        } else {
            self.reflector.forward(x, 0)
        };
        x = (x + 3) % 30;
        for i in 0..10 {
            x = self.rotors[i].backward(x, (self.state.rotor_positions[i] + self.rings[i].0) % 30);
        }
        self.keyboard.backward(
            self.card.backward(self.entry.backward((x + 27) % 30, 0), 0),
            0,
        )
    }
    fn advance(&mut self) {
        for (slots, delta, sense) in [([1, 3, 5, 7, 9], 29, 17), ([8, 6, 4, 2, 0], 1, 20)] {
            for i in slots {
                let at = (self.state.rotor_positions[i] + self.rings[i].0 + sense) % 30;
                let blocked = data::PINS[usize::from(self.ids[i].0 - 1)][usize::from(at)] != 0;
                self.state.rotor_positions[i] = (self.state.rotor_positions[i] + delta) % 30;
                if blocked {
                    break;
                }
            }
        }
    }
}
/// Encipher unpacked contacts 0..29, including all thirty electrical inputs.
pub fn encode(codes: &[u8], p: &Settings) -> Result<Report, OperationError> {
    transform(codes, p, false, "cipher.fialka.encode")
}
/// Invert the three-cycle reflector and the entire electrical path.
pub fn decode(codes: &[u8], p: &Settings) -> Result<Report, OperationError> {
    transform(codes, p, true, "cipher.fialka.decode")
}
fn transform(
    codes: &[u8],
    p: &Settings,
    decode: bool,
    op: &'static str,
) -> Result<Report, OperationError> {
    lorenz::input_budget(codes.len(), op)?;
    if p.trace && codes.len() > MAX_TRACE {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "trace",
            "trace exceeds 65536 input contacts",
        ));
    }
    let mut machine = Machine::new(p, op)?;
    let mut result = Vec::with_capacity(codes.len());
    let mut trace = Vec::new();
    for (offset, &x) in codes.iter().enumerate() {
        if x >= 30 {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "codes",
                "contacts must be 0..29",
            )
            .offset(offset as u64));
        }
        result.push(machine.letter(x, decode));
        machine.advance();
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: machine.state.clone(),
            });
        }
    }
    finish_report(
        Report {
            codes: result,
            final_state: machine.state,
            trace,
        },
        op,
    )
}
