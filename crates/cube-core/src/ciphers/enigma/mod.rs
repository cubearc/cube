//! Enigma I and naval M3, with operator-view left-to-right settings.
//! Historical interoperability only; no confidentiality or authentication.
//! Wiring: <https://www.cryptomuseum.com/crypto/enigma/wiring.htm>.
//! Stepping: David Hamer, *Enigma: Actions Involved in the Double Stepping of
//! the Middle Rotor*, <https://www.cryptomuseum.com/people/hamer/files/double_stepping.pdf>.
mod catalog;
mod machine;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_INPUT: usize = 1024 * 1024;
pub const MAX_TRACE: usize = 65536;
pub const MAX_PARAMETERS: usize = 4096;
pub const ID: &str = "cipher.enigma";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    EnigmaI,
    M3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Rotor {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
    VIII,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Reflector {
    B,
    C,
}

/// Strict accepts only A-Z. Preserve is an explicit text wrapper: ASCII case and
/// all other scalars are retained, and only ASCII letters advance the machine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextMode {
    #[default]
    Strict,
    Preserve,
}

#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct Ring(#[schemars(range(min = 1, max = 26))] u8);

/// Required historical key settings; no guessed machine, rotor order or key.
/// Arrays and windows run left-to-right, with the rightmost rotor fastest.
/// Ring 1 corresponds to A. Notches are on the index ring: visible turnover
/// letters do not change with ring settings. M4/thin reflectors are unsupported.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    pub rotors: [Rotor; 3],
    pub reflector: Reflector,
    #[schemars(with = "[Ring; 3]")]
    pub rings: [u8; 3],
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub windows: String,
    #[schemars(length(max = 10), inner(regex(pattern = "^[A-Z]{2}$")))]
    pub plugboard: Vec<String>,
    #[serde(default)]
    pub text_mode: TextMode,
    #[serde(default)]
    pub trace: bool,
}

/// Post-step windows for one enciphered letter, identified by UTF-8 byte offset.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub windows: String,
}

/// Resume with final_windows, keeping all other settings. Decrypt by resetting
/// to the original windows; applying the operation at the final state is different.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max = 1048576), extend("x-cube-max-utf8-bytes" = 1048576))]
    pub text: String,
    #[schemars(regex(pattern = "^[A-Z]{3}$"))]
    pub final_windows: String,
    #[schemars(length(max = 65536))]
    pub trace: Vec<State>,
}

fn error(code: ErrorCode, parameter: &'static str, message: &'static str) -> OperationError {
    OperationError::new(code, message)
        .operation(ID)
        .parameter(parameter)
}

/// Parse a strict bounded JSON settings document. Reject duplicate keys, unknown
/// fields, unsupported names and excessive depth before typed deserialization.
pub fn parse_parameters(document: &str) -> Result<Parameters, OperationError> {
    let value = crate::pipeline::json::parse_with_limits(document, MAX_PARAMETERS, 4, 128)
        .map_err(|e| {
            error(
                e.error.code,
                "parameters",
                "invalid bounded settings document",
            )
        })?;
    serde_json::from_value(value).map_err(|_| {
        error(
            ErrorCode::InvalidParameter,
            "parameters",
            "use exact typed Enigma settings",
        )
    })
}

/// Transform at most 1 MiB, returning text, final windows and optional post-step
/// states. Enforce settings and trace limits before processing; never truncate.
/// Work is linear with six fixed rotor lookups per letter. No external state,
/// I/O, random choices, normalization or message-key procedures are involved.
///
/// ```
/// use cube_core::ciphers::enigma::{parse_parameters, transform};
/// let p = parse_parameters(r#"{"profile":"enigma_i","rotors":["I","II","III"],"reflector":"B","rings":[1,1,1],"windows":"AAA","plugboard":[]}"#)?;
/// let r = transform("AAAAA", &p)?;
/// assert_eq!(r.text, "BDZGO");
/// assert_eq!(r.final_windows, "AAF");
/// # Ok::<(), cube_core::errors::OperationError>(())
/// ```
pub fn transform(text: &str, parameters: &Parameters) -> Result<Report, OperationError> {
    if text.len() > MAX_INPUT {
        return Err(error(
            ErrorCode::ResourceLimit,
            "text",
            "text exceeds 1 MiB",
        ));
    }
    let mut machine = machine::Machine::new(parameters)?;
    let count = validate_text(text, parameters)?;
    let mut report = Report {
        text: String::with_capacity(text.len()),
        final_windows: String::new(),
        trace: Vec::with_capacity(if parameters.trace { count } else { 0 }),
    };
    for (offset, ch) in text.char_indices() {
        if ch.is_ascii_alphabetic() {
            let code = machine.key(ch.to_ascii_uppercase() as u8 - b'A');
            let output = char::from(b'A' + code);
            report.text.push(if ch.is_ascii_lowercase() {
                output.to_ascii_lowercase()
            } else {
                output
            });
            if parameters.trace {
                report.trace.push(State {
                    offset: offset as u32,
                    windows: machine.windows(),
                });
            }
        } else {
            report.text.push(ch);
        }
    }
    report.final_windows = machine.windows();
    Ok(report)
}

fn validate_text(text: &str, p: &Parameters) -> Result<usize, OperationError> {
    let mut letters = 0;
    for (offset, ch) in text.char_indices() {
        if p.text_mode == TextMode::Strict && !ch.is_ascii_uppercase() {
            return Err(error(
                ErrorCode::InvalidInput,
                "text",
                "strict mode requires ASCII A-Z",
            )
            .offset(offset as u64));
        }
        if ch.is_ascii_alphabetic() {
            letters += 1;
        }
    }
    if p.trace && letters > MAX_TRACE {
        return Err(error(
            ErrorCode::ResourceLimit,
            "trace",
            "trace exceeds 65536 letter states",
        ));
    }
    Ok(letters)
}
