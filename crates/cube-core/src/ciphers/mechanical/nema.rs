//! Swiss Model45 operational/training machines, with actual lower-row windows.
//! Step from the old cam selections, then apply the post-step electrical path.
use super::{nema_data as data, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "nema_model45_operational_v1")]
    Operational,
    #[serde(rename = "nema_model45_training_v1")]
    Training,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Rotor {
    A,
    B,
    C,
    D,
    E,
    F,
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
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Ten lower-row window letters, physically left to right: UKW through red.
    #[schemars(regex(pattern = "^[A-Z]{10}$"))]
    pub windows: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    /// Complete distinct contact-wheel identities, left to right.
    pub rotors: [Rotor; 4],
    /// Complete distinct paired drive-wheel identities, left to right.
    pub drive_wheels: [u8; 4],
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
struct Machine {
    positions: [u8; 10],
    forward: [[u8; 26]; 4],
    inverse: [[u8; 26]; 4],
    reflector: [u8; 26],
    entry: [u8; 26],
    exit: [u8; 26],
    drive: [&'static [u8; 26]; 5],
    red_right: &'static [u8; 26],
}
fn wiring(source: &[u8; 26]) -> [u8; 26] {
    source.map(|b| b - b'A')
}
fn inverse(source: [u8; 26]) -> [u8; 26] {
    let mut result = [0; 26];
    for (input, output) in source.iter().enumerate() {
        result[usize::from(*output)] = input as u8;
    }
    result
}
fn notch(id: u8, op: &'static str) -> Result<&'static [u8; 26], OperationError> {
    data::NOTCHES
        .iter()
        .find(|(number, _)| *number == id)
        .map(|(_, value)| *value)
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidParameter,
                "drive_wheels",
                "unknown Model45 drive wheel",
            )
        })
}
impl Machine {
    fn configure(p: &Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        let allowed: &[u8] = match p.profile {
            Profile::Operational => &[12, 13, 14, 15, 17, 18],
            Profile::Training => &[16, 19, 20, 21],
        };
        let mut rotors = 0u8;
        let mut drives = 0u32;
        for (rotor, drive) in p.rotors.iter().zip(p.drive_wheels) {
            let index = rotor.index();
            if rotors & (1 << index) != 0 || matches!(p.profile, Profile::Training) && index > 3 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "rotors",
                    "select four distinct contact wheels available in the chosen profile",
                ));
            }
            rotors |= 1 << index;
            if !allowed.contains(&drive) || drives & (1 << drive) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "drive_wheels",
                    "select four distinct drive wheels available in the chosen profile",
                ));
            }
            drives |= 1 << drive;
        }
        let windows = p.initial_state.windows.as_bytes();
        if windows.len() != 10 || !windows.iter().all(u8::is_ascii_uppercase) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "initial_state",
                "use ten exact uppercase lower-row window letters",
            ));
        }
        let positions = std::array::from_fn(|i| windows[9 - i] - b'A');
        let forward = std::array::from_fn(|i| wiring(data::ROTORS[p.rotors[3 - i].index()]));
        let reverse = forward.map(inverse);
        let entry = wiring(data::ENTRY);
        let (left, right) = match p.profile {
            Profile::Operational => (22, 1),
            Profile::Training => (23, 2),
        };
        let drive = [
            notch(left, op)?,
            notch(p.drive_wheels[3], op)?,
            notch(p.drive_wheels[2], op)?,
            notch(p.drive_wheels[1], op)?,
            notch(p.drive_wheels[0], op)?,
        ];
        Ok(Self {
            positions,
            forward,
            inverse: reverse,
            reflector: wiring(data::REFLECTOR),
            entry,
            exit: inverse(entry),
            drive,
            red_right: notch(right, op)?,
        })
    }
    fn state(&self) -> State {
        State {
            windows: self
                .positions
                .iter()
                .rev()
                .map(|v| char::from(b'A' + *v))
                .collect(),
        }
    }
    fn advance(&mut self) {
        // Ordinary fingers read at window-10; the red right probe reads at-9.
        // All selections use old positions, including when an adjacent wheel moves.
        let active = |pattern: &[u8; 26], slot: usize, offset: u8| {
            pattern[usize::from((self.positions[slot] + 26 - offset) % 26)] == b'1'
        };
        let slow = active(self.red_right, 0, 9);
        let selected = [
            true,
            active(self.drive[0], 0, 10),
            slow,
            slow && active(self.drive[1], 2, 10),
            true,
            active(self.drive[2], 4, 10),
            slow,
            slow && active(self.drive[3], 6, 10),
            true,
            active(self.drive[4], 8, 10),
        ];
        for (position, step) in self.positions.iter_mut().zip(selected) {
            if step {
                *position = (*position + 25) % 26;
            }
        }
    }
    fn letter(&self, input: u8) -> u8 {
        let mut value = self.entry[usize::from(input)];
        for (index, source) in self.forward.iter().enumerate() {
            value = contact(value, source, self.positions[index * 2 + 1]);
        }
        value = contact(value, &self.reflector, self.positions[9]);
        for index in (0..4).rev() {
            value = contact(value, &self.inverse[index], self.positions[index * 2 + 1]);
        }
        self.exit[usize::from(value)]
    }
}
fn contact(value: u8, source: &[u8; 26], window: u8) -> u8 {
    // Contact1 is I (8); the lower-row benchmark is two letters below top.
    // In an A-numbered array the electrical displacement is window-10.
    let shift = (window + 16) % 26;
    (source[usize::from((value + shift) % 26)] + 26 - shift) % 26
}
/// Encipher prepared A-Z with required Model45 wheel selection and lower-row
/// windows. The stepping phase and fixed contact facts belong to this module.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, "cipher.nema.encode")
}
/// Reciprocal deciphering with the sender's original windows and wheel choices.
/// final_state can be supplied as the next chunk's initial_state.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, "cipher.nema.decode")
}
fn transform(text: &str, p: &Settings, op: &'static str) -> Result<Report, OperationError> {
    let mut machine = Machine::configure(p, op)?;
    bounded_text(text, p.trace, op)?;
    let mut result = String::with_capacity(text.len());
    let mut trace = Vec::new();
    for (offset, input) in text.bytes().enumerate() {
        if !input.is_ascii_uppercase() {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "prepared text must contain only ASCII A-Z",
            )
            .offset(offset as u64));
        }
        machine.advance();
        result.push(char::from(b'A' + machine.letter(input - b'A')));
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: machine.state(),
            });
        }
    }
    finish_report(
        Report {
            text: result,
            final_state: machine.state(),
            trace,
        },
        op,
    )
}
