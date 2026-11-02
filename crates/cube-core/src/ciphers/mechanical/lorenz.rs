//! Lorenz SZ40/SZ42a/SZ42b on unpacked five-bit codes, with explicit logical
//! contact indices and history. No physical-window offset or ITA2 conversion is guessed.
//! Sources: TICOM T663, printed pp. 8–11 and 17–19; Sawyers, restoration report
//! sections 4.3–4.4; General Report on Tunny, 11B and 53C(d).
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CHI_LENGTHS: [usize; 5] = [41, 31, 29, 26, 23];
pub const PSI_LENGTHS: [usize; 5] = [43, 47, 51, 53, 59];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "lorenz_sz40_v1")]
    Sz40,
    #[serde(rename = "lorenz_sz42a_v1")]
    Sz42a,
    #[serde(rename = "lorenz_sz42b_v1")]
    Sz42b,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    /// Logical contact indices in each supplied sequence, not window numbers.
    pub chi: [u8; 5],
    pub psi: [u8; 5],
    #[schemars(range(max = 60))]
    pub mu61: u8,
    #[schemars(range(max = 36))]
    pub mu37: u8,
    /// Signal from the preceding processed character, even if Psi did not move.
    pub chi2_previous: bool,
    pub psi1_previous: bool,
    /// Fifth plaintext impulses two characters back, then one character back.
    pub plain5_previous: [bool; 2],
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct PinPattern<'a>(#[schemars(regex(pattern = "^[01]{23,61}$"))] &'a str);
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct Code(#[schemars(range(max = 31))] u8);
impl Code {
    fn parse(value: u8, op: &'static str, offset: usize) -> Result<Self, OperationError> {
        if value > 31 {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "codes",
                "each code must be in 0..31",
            )
            .offset(offset as u64));
        }
        Ok(Self(value))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 524288))]
pub struct Settings {
    pub profile: Profile,
    /// Impulse order 1..5. Index zero is the first supplied contact.
    #[schemars(with = "[PinPattern<'static>; 5]")]
    pub chi: [String; 5],
    #[schemars(with = "[PinPattern<'static>; 5]")]
    pub psi: [String; 5],
    #[schemars(regex(pattern = "^[01]{61}$"))]
    pub mu61: String,
    #[schemars(regex(pattern = "^[01]{37}$"))]
    pub mu37: String,
    /// Required KT switch; SZ40 refuses true. Initial history is always explicit.
    pub plaintext_limitation: bool,
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
    #[schemars(range(max = 31))]
    pub key_code: u8,
    pub basic_motor: bool,
    pub psi_step: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(with = "Vec<Code>", length(max = 1048576))]
    pub codes: Vec<u8>,
    pub final_state: State,
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
struct Pins<'a> {
    chi: [&'a [u8]; 5],
    psi: [&'a [u8]; 5],
    mu61: &'a [u8],
    mu37: &'a [u8],
}
impl<'a> Pins<'a> {
    fn new(p: &'a Settings, op: &'static str) -> Result<Self, OperationError> {
        let chi = p.chi.each_ref().map(|s| PinPattern(s));
        let psi = p.psi.each_ref().map(|s| PinPattern(s));
        for (patterns, lengths, field) in [(&chi, CHI_LENGTHS, "chi"), (&psi, PSI_LENGTHS, "psi")] {
            for (pattern, length) in patterns.iter().zip(lengths) {
                validate_pattern(*pattern, length, op, field)?;
            }
        }
        validate_pattern(PinPattern(&p.mu61), 61, op, "mu61")?;
        validate_pattern(PinPattern(&p.mu37), 37, op, "mu37")?;
        validate_state(&p.initial_state, op)?;
        if p.profile == Profile::Sz40 && p.plaintext_limitation {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "plaintext_limitation",
                "SZ40 has no KT limitation",
            ));
        }
        Ok(Self {
            chi: chi.map(|p| p.0.as_bytes()),
            psi: psi.map(|p| p.0.as_bytes()),
            mu61: p.mu61.as_bytes(),
            mu37: p.mu37.as_bytes(),
        })
    }
}
fn validate_pattern(
    pattern: PinPattern<'_>,
    length: usize,
    op: &'static str,
    field: &'static str,
) -> Result<(), OperationError> {
    let pins = pattern.0.as_bytes();
    if pins.len() != length || pins.iter().any(|b| !matches!(b, b'0' | b'1')) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            field,
            "use the exact wheel length and binary pins",
        ));
    }
    Ok(())
}
fn validate_state(s: &State, op: &'static str) -> Result<(), OperationError> {
    let invalid = s
        .chi
        .iter()
        .zip(CHI_LENGTHS)
        .chain(s.psi.iter().zip(PSI_LENGTHS))
        .any(|(&position, length)| usize::from(position) >= length);
    if invalid || s.mu61 >= 61 || s.mu37 >= 37 {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "initial_state",
            "contact index outside its wheel",
        ));
    }
    Ok(())
}
pub(super) fn input_budget(count: usize, op: &'static str) -> Result<(), OperationError> {
    if count > MAX_INPUT {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "codes",
            "more than 1048576 five-bit codes",
        ));
    }
    Ok(())
}
/// Parse a bounded JSON array for CLI/native transports. Reject malformed values
/// without echoing private data; code range validation remains in the transform.
pub fn parse_codes(document: &str, op: &'static str) -> Result<Vec<u8>, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_OUTPUT, 2, MAX_INPUT + 1)
        .map_err(|e| {
            error(
                op,
                e.error.code,
                "codes",
                "invalid bounded five-bit code array",
            )
        })?;
    serde_json::from_value(value).map_err(|_| {
        error(
            op,
            ErrorCode::InvalidInput,
            "codes",
            "use an array of five-bit integer codes",
        )
    })
}
/// Encipher unpacked numeric codes 0..31; impulse 1 is the least significant bit.
/// Sample the current contacts, then advance indices by one modulo wheel length.
/// Supply complete history and use final_state as the next initial_state.
pub fn encode(codes: &[u8], settings: &Settings) -> Result<Report, OperationError> {
    transform(codes, settings, false)
}
/// Decipher using the same original state. KT feedback uses recovered plaintext,
/// so history is identical at both ends, including across continued blocks.
pub fn decode(codes: &[u8], settings: &Settings) -> Result<Report, OperationError> {
    transform(codes, settings, true)
}
fn transform(codes: &[u8], p: &Settings, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.lorenz.decode"
    } else {
        "cipher.lorenz.encode"
    };
    input_budget(codes.len(), op)?;
    parameter_size(p, op)?;
    if p.trace && codes.len() > MAX_TRACE {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "trace",
            "trace exceeds 65536 symbols",
        ));
    }
    let pins = Pins::new(p, op)?;
    let mut s = p.initial_state;
    let mut r = Report {
        codes: Vec::with_capacity(codes.len()),
        final_state: s,
        trace: Vec::with_capacity(if p.trace { codes.len() } else { 0 }),
    };
    for (offset, &input) in codes.iter().enumerate() {
        let input = Code::parse(input, op, offset)?.0;
        let chi = signal(&pins.chi, &s.chi);
        let psi = signal(&pins.psi, &s.psi);
        let key = chi ^ psi;
        let output = input ^ key;
        let basic_motor = pins.mu37[usize::from(s.mu37)] == b'1';
        let psi_step = basic_motor || limitation_releases(p, &s);
        advance(&mut s, &pins, psi_step);
        s.chi2_previous = chi & 2 != 0;
        s.psi1_previous = psi & 1 != 0;
        s.plain5_previous = [
            s.plain5_previous[1],
            (if decode { output } else { input }) & 16 != 0,
        ];
        r.codes.push(output);
        if p.trace {
            r.trace.push(Trace {
                offset: offset as u32,
                state: s,
                key_code: key,
                basic_motor,
                psi_step,
            });
        }
    }
    r.final_state = s;
    finish_report(r, op)
}
fn signal(pins: &[&[u8]; 5], positions: &[u8; 5]) -> u8 {
    pins.iter()
        .zip(positions)
        .enumerate()
        .fold(0, |code, (bit, (pins, &position))| {
            code | (u8::from(pins[usize::from(position)] == b'1') << bit)
        })
}
fn limitation_releases(p: &Settings, s: &State) -> bool {
    if p.profile == Profile::Sz40 {
        return false;
    }
    let mut limitation = s.chi2_previous;
    if p.profile == Profile::Sz42b {
        limitation ^= s.psi1_previous;
    }
    if p.plaintext_limitation {
        limitation ^= s.plain5_previous[0];
    }
    !limitation
}
fn advance(s: &mut State, pins: &Pins<'_>, psi_step: bool) {
    for (position, length) in s.chi.iter_mut().zip(CHI_LENGTHS) {
        *position = (*position + 1) % length as u8;
    }
    if psi_step {
        for (position, length) in s.psi.iter_mut().zip(PSI_LENGTHS) {
            *position = (*position + 1) % length as u8;
        }
    }
    if pins.mu61[usize::from(s.mu61)] == b'1' {
        s.mu37 = (s.mu37 + 1) % 37;
    }
    s.mu61 = (s.mu61 + 1) % 61;
}
