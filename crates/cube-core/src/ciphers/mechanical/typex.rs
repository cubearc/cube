//! Configured Typex Mark22 raw contact path with three moving drums and two stators.
use super::{alphabet::Alphabet, *};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "typex_mk22_configured_v1")]
    Mk22Configured,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rotor {
    /// Distinct caller-supplied core identity, separate from its orientation.
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = "^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
    )]
    pub id: String,
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub wiring: String,
    pub reversed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Physical left-to-right slow, middle, fast, stator2, stator1 windows.
    #[schemars(regex(pattern = "^[A-Z]{5}$"))]
    pub windows: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = "^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
    )]
    pub wheel_set_id: String,
    /// Physical left-to-right cores: three moving followed by two stationary.
    pub rotors: [Rotor; 5],
    #[schemars(regex(pattern = "^[A-Z]{5}$"))]
    pub ring_settings: String,
    /// Old middle and fast window letters that permit turnover. Reversing a
    /// wiring core does not reverse its casing or these physical notch windows.
    pub turnover_windows: [String; 2],
    /// Keyboard contacts A-Z to the right-hand entry plate, in the chosen frame.
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub entry_wiring: String,
    /// Complete arbitrary input plug permutation; the return path is its inverse.
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub plugboard: String,
    /// Complete involution of thirteen pairs, without self contacts.
    #[schemars(regex(pattern = "^[A-Z]{26}$"))]
    pub reflector: String,
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
#[derive(Clone, Copy, Debug)]
struct Contacts {
    forward: [u8; 26],
    backward: [u8; 26],
}
impl Contacts {
    fn new(
        text: &str,
        reversed: bool,
        op: &'static str,
        field: &'static str,
    ) -> Result<Self, OperationError> {
        let a = Alphabet::new(text, op, field)?;
        // Turning over a core exchanges the two faces and reverses contact
        // numbering: R(x)=-W^-1(-x), not merely W^-1(x).
        let forward = std::array::from_fn(|i| {
            if reversed {
                (26 - a
                    .position(b'A' + ((26 - i) % 26) as u8)
                    .expect("validated A-Z"))
                    % 26
            } else {
                a.letter(i as u8) - b'A'
            }
        });
        let mut backward = [0; 26];
        for (x, y) in forward.iter().enumerate() {
            backward[usize::from(*y)] = x as u8;
        }
        Ok(Self { forward, backward })
    }
}
fn identity(text: &str) -> bool {
    (1..=128).contains(&text.len())
        && text.as_bytes()[0].is_ascii_alphanumeric()
        && text
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
fn letters(text: &str, op: &'static str, field: &'static str) -> Result<[u8; 5], OperationError> {
    if text.len() != 5 || !text.bytes().all(|c| c.is_ascii_uppercase()) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            field,
            "use five exact uppercase window or ring letters",
        ));
    }
    Ok(std::array::from_fn(|i| text.as_bytes()[i] - b'A'))
}
fn notches(text: &str, op: &'static str) -> Result<u32, OperationError> {
    let mut bits = 0;
    if text.is_empty() || text.len() > 26 {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "turnover_windows",
            "use distinct uppercase notch window letters",
        ));
    }
    for c in text.bytes() {
        if !c.is_ascii_uppercase() || bits & (1 << (c - b'A')) != 0 {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "turnover_windows",
                "use distinct uppercase notch window letters",
            ));
        }
        bits |= 1 << (c - b'A');
    }
    Ok(bits)
}
struct Machine {
    windows: [u8; 5],
    rings: [u8; 5],
    cores: [Contacts; 5],
    turnover: [u32; 2],
    entry: Contacts,
    plugs: Contacts,
    reflector: Contacts,
}
impl Machine {
    fn configure(p: &Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        if !identity(&p.wheel_set_id) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "wheel_set_id",
                "use an explicit ASCII wheel-set identity",
            ));
        }
        for (i, r) in p.rotors.iter().enumerate() {
            if !identity(&r.id) || p.rotors[..i].iter().any(|other| other.id == r.id) {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "rotors",
                    "use five distinct explicit core identities",
                ));
            }
        }
        let mut cores = Vec::with_capacity(5);
        for r in &p.rotors {
            cores.push(Contacts::new(&r.wiring, r.reversed, op, "rotors")?);
        }
        let reflector = Contacts::new(&p.reflector, false, op, "reflector")?;
        if (0..26).any(|x| {
            reflector.forward[x] == x as u8
                || reflector.forward[usize::from(reflector.forward[x])] != x as u8
        }) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "reflector",
                "use thirteen complete reciprocal pairs without self contacts",
            ));
        }
        Ok(Self {
            windows: letters(&p.initial_state.windows, op, "initial_state")?,
            rings: letters(&p.ring_settings, op, "ring_settings")?,
            cores: cores.try_into().expect("five checked cores"),
            turnover: [
                notches(&p.turnover_windows[0], op)?,
                notches(&p.turnover_windows[1], op)?,
            ],
            entry: Contacts::new(&p.entry_wiring, false, op, "entry_wiring")?,
            plugs: Contacts::new(&p.plugboard, false, op, "plugboard")?,
            reflector,
        })
    }
    fn state(&self) -> State {
        State {
            windows: self.windows.iter().map(|x| char::from(b'A' + *x)).collect(),
        }
    }
    fn advance(&mut self) {
        let middle_notch = self.turnover[0] & (1 << self.windows[1]) != 0;
        let fast_notch = self.turnover[1] & (1 << self.windows[2]) != 0;
        self.windows[2] = (self.windows[2] + 1) % 26;
        if middle_notch || fast_notch {
            self.windows[1] = (self.windows[1] + 1) % 26;
        }
        if middle_notch {
            self.windows[0] = (self.windows[0] + 1) % 26;
        }
    }
    fn letter(&self, input: u8) -> u8 {
        let mut x = self.entry.forward[usize::from(self.plugs.forward[usize::from(input)])];
        for i in (0..5).rev() {
            x = contact(
                x,
                &self.cores[i].forward,
                (self.windows[i] + 26 - self.rings[i]) % 26,
            );
        }
        x = self.reflector.forward[usize::from(x)];
        for i in 0..5 {
            x = contact(
                x,
                &self.cores[i].backward,
                (self.windows[i] + 26 - self.rings[i]) % 26,
            );
        }
        self.plugs.backward[usize::from(self.entry.backward[usize::from(x)])]
    }
}
fn contact(x: u8, w: &[u8; 26], shift: u8) -> u8 {
    (w[usize::from((x + shift) % 26)] + 26 - shift) % 26
}
/// Encipher prepared contact symbols A-Z with complete caller wiring and Mk22
/// three-pawl pre-step mechanics. No keyboard shifts or message procedure.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, "cipher.typex.encode")
}
/// Reciprocal deciphering from the sender's original settings; final_state is
/// the complete resumable state, including both stationary window letters.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, "cipher.typex.decode")
}
fn transform(text: &str, p: &Settings, op: &'static str) -> Result<Report, OperationError> {
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
                "prepared text must contain only ASCII A-Z",
            )
            .offset(offset as u64));
        }
        m.advance();
        result.push(char::from(b'A' + m.letter(c - b'A')));
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: m.state(),
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
