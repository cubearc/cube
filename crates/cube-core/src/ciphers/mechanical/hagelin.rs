//! Explicit C-36 fixed25, C-38 standard27, M-209, C-52 C2 and later
//! CX-52M-27 profiles. Pins are sampled at logical contact indices, before
//! advancement; no window translation, message procedure or printer substitution.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Required, independently specified machine configuration; these are precise
/// profiles, not an assertion that all machines sharing a name interoperate.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "profile")]
#[schemars(extend("x-cube-max-serialized-bytes" = 524288))]
pub enum Settings {
    #[serde(rename = "c36_fixed25_bouch_v1")]
    C36(FixedSettings),
    #[serde(rename = "c38_standard27_v1")]
    C38(LugSettings),
    #[serde(rename = "m209_standard_v1")]
    M209(M209Settings),
    #[serde(rename = "c52_c2_27_v1")]
    C52(LugSettings),
    #[serde(rename = "cx52_m27_late_v1")]
    Cx52(LugSettings),
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct PinPattern<'a>(#[schemars(regex(pattern = "^[01]{17,47}$"))] &'a str);
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FixedSettings {
    /// Left-to-right lengths25,23,21,19,17. One means an effective pin.
    #[schemars(with = "[PinPattern<'static>; 5]")]
    pub pins: [String; 5],
    pub initial_state: [u8; 5],
    /// Numeric A=0..Z=25; reversed standard alphabets only.
    #[schemars(range(max = 25))]
    pub slide: u8,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LugSettings {
    #[schemars(with = "[PinPattern<'static>; 6]")]
    pub pins: [String; 6],
    /// Complete27 printing-bar matrix, columns left-to-right wheels1..6.
    /// A true cell denotes a lug; each displaced bar contributes exactly once.
    pub lugs: [[bool; 6]; 27],
    pub initial_state: [u8; 6],
    #[schemars(range(max = 25))]
    pub slide: u8,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct M209Settings {
    #[schemars(with = "[PinPattern<'static>; 6]")]
    pub pins: [String; 6],
    /// Each bar may have at most two effective lugs. Inactive positions are
    /// represented by false cells; physical lug ordering has no cipher effect.
    pub lugs: [[bool; 6]; 27],
    pub initial_state: [u8; 6],
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    /// Post-cycle logical contact indices; the next symbol starts here.
    #[schemars(length(min = 5, max = 6))]
    pub state: Vec<u8>,
    /// Guide-arm selection latched for this complete cycle.
    #[schemars(range(max = 63))]
    pub active_mask: u8,
    #[schemars(range(max = 27))]
    pub displacement: u8,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max = 1048576), extend("x-cube-max-utf8-bytes" = 1048576))]
    pub text: String,
    #[schemars(length(min = 5, max = 6))]
    pub final_state: Vec<u8>,
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
#[derive(Clone, Copy)]
enum Advancement {
    Regular,
    DoubleLastFour,
    PrefixOr,
}
struct Machine<'a> {
    pins: Vec<PinPattern<'a>>,
    state: Vec<u8>,
    bars: Vec<u8>,
    slide: u8,
    trace: bool,
    advancement: Advancement,
}
impl<'a> Machine<'a> {
    fn configure(p: &'a Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        let machine = match p {
            Settings::C36(p) => {
                let bars = [1, 2, 3, 7, 12]
                    .into_iter()
                    .enumerate()
                    .flat_map(|(wheel, count)| std::iter::repeat_n(1 << wheel, count))
                    .collect();
                Self::new(
                    &p.pins,
                    &p.initial_state,
                    bars,
                    p.slide,
                    p.trace,
                    Advancement::Regular,
                )
                .checked(&[25, 23, 21, 19, 17], op)?
            }
            Settings::M209(p) => {
                check_two_lugs(&p.lugs, op)?;
                Self::new(
                    &p.pins,
                    &p.initial_state,
                    masks(&p.lugs),
                    25,
                    p.trace,
                    Advancement::Regular,
                )
                .checked(&[26, 25, 23, 21, 19, 17], op)?
            }
            Settings::C38(p) => {
                check_two_lugs(&p.lugs, op)?;
                Self::new(
                    &p.pins,
                    &p.initial_state,
                    masks(&p.lugs),
                    p.slide,
                    p.trace,
                    Advancement::Regular,
                )
                .checked(&[26, 25, 23, 21, 19, 17], op)?
            }
            Settings::C52(p) => Self::new(
                &p.pins,
                &p.initial_state,
                masks(&p.lugs),
                p.slide,
                p.trace,
                Advancement::DoubleLastFour,
            )
            .checked(&[26, 25, 46, 42, 38, 34], op)?,
            Settings::Cx52(p) => Self::new(
                &p.pins,
                &p.initial_state,
                masks(&p.lugs),
                p.slide,
                p.trace,
                Advancement::PrefixOr,
            )
            .checked(&[47; 6], op)?,
        };
        Ok(machine)
    }
    fn new<const N: usize>(
        pins: &'a [String; N],
        state: &[u8; N],
        bars: Vec<u8>,
        slide: u8,
        trace: bool,
        advancement: Advancement,
    ) -> Self {
        Self {
            pins: pins.iter().map(|p| PinPattern(p)).collect(),
            state: state.to_vec(),
            bars,
            slide,
            trace,
            advancement,
        }
    }
    fn checked(self, lengths: &[usize], op: &'static str) -> Result<Self, OperationError> {
        if self.slide > 25 {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "slide",
                "slide must be in0..25",
            ));
        }
        for ((pins, position), length) in self.pins.iter().zip(&self.state).zip(lengths) {
            if pins.0.len() != *length
                || !pins.0.as_bytes().iter().all(|b| matches!(b, b'0' | b'1'))
            {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "pins",
                    "use exact binary pin lengths for the selected machine profile",
                ));
            }
            if usize::from(*position) >= *length {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "initial_state",
                    "contact index exceeds its wheel length",
                ));
            }
        }
        Ok(self)
    }
    fn active_mask(&self) -> u8 {
        self.pins
            .iter()
            .zip(&self.state)
            .enumerate()
            .fold(0, |mask, (wheel, (pins, position))| {
                mask | (u8::from(pins.0.as_bytes()[usize::from(*position)] == b'1') << wheel)
            })
    }
    fn advance(&mut self, active: u8) {
        for (wheel, (position, pins)) in self.state.iter_mut().zip(&self.pins).enumerate() {
            let steps = match self.advancement {
                Advancement::Regular => 1,
                Advancement::DoubleLastFour => {
                    if wheel < 2 {
                        1
                    } else {
                        2
                    }
                }
                Advancement::PrefixOr => {
                    usize::from(wheel == 0 || active & ((1 << wheel) - 1) != 0)
                }
            };
            *position = ((usize::from(*position) + steps) % pins.0.len()) as u8;
        }
    }
}
fn masks(lugs: &[[bool; 6]; 27]) -> Vec<u8> {
    lugs.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .fold(0, |m, (i, yes)| m | (u8::from(*yes) << i))
        })
        .collect()
}
fn check_two_lugs(lugs: &[[bool; 6]; 27], op: &'static str) -> Result<(), OperationError> {
    if lugs
        .iter()
        .any(|row| row.iter().filter(|yes| **yes).count() > 2)
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "lugs",
            "this profile has at most two effective lugs on each bar",
        ));
    }
    Ok(())
}
/// Encipher prepared uppercase A-Z using a complete required configuration.
/// The returned logical contact indices support exact chunk continuation.
pub fn encode(text: &str, settings: &Settings) -> Result<Report, OperationError> {
    transform(text, settings, "cipher.hagelin.encode")
}
/// Decipher with the same initial state. All supported profiles use reciprocal
/// standard reversed alphabets; printer spaces and grouping are not introduced.
pub fn decode(text: &str, settings: &Settings) -> Result<Report, OperationError> {
    transform(text, settings, "cipher.hagelin.decode")
}
fn transform(text: &str, settings: &Settings, op: &'static str) -> Result<Report, OperationError> {
    let mut machine = Machine::configure(settings, op)?;
    bounded_text(text, machine.trace, op)?;
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
        let active = machine.active_mask();
        let displacement = machine
            .bars
            .iter()
            .filter(|bar| **bar & active != 0)
            .count() as u8;
        result.push((b'A' + (machine.slide + displacement + 26 - (input - b'A')) % 26) as char);
        machine.advance(active);
        if machine.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: machine.state.clone(),
                active_mask: active,
                displacement,
            });
        }
    }
    finish_report(
        Report {
            text: result,
            final_state: machine.state,
            trace,
        },
        op,
    )
}
