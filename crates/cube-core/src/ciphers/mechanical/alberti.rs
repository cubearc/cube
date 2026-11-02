//! Di Palma 2009's precisely configured museum reconstruction, pp11–13:
//! https://www2.mathematik.uni-halle.de/reports/sources/2009/09-04report.pdf.
//! Fixed outer A alignment; digit indicators feed back the emitted inner symbol.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const OUTER: &[u8; 24] = b"ABCDEFGILMNOPQRSTVXZ1234";
pub const INNER: &[u8; 24] = b"c&bmdgpfznxyutoskerlhaiq";
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "alberti_di_palma_2009_v1")]
    DiPalma2009V1,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    ///Inner letter placed under fixed A; the publication's example uses c.
    #[schemars(regex(pattern = "^[c&bmdgpfznxyutoskerlhaiq]$"))]
    pub initial_index: String,
    #[serde(default)]
    pub trace: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub index: char,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))]
    pub text: String,
    pub final_index: char,
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
/// Encipher exact prepared outer-ring symbols, including explicitly supplied
/// control digits 1..4. Use canonical uppercase V and lowercase u for the U/V cell;
/// no case folding, random digit insertion or plaintext preparation is performed.
pub fn encode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, false)
}
/// Recover exact prepared symbols, retaining digits so the chosen alignment
/// history remains reversible. Phrase-book expansion and word spacing are separate.
pub fn decode(text: &str, p: &Settings) -> Result<Report, OperationError> {
    transform(text, p, true)
}
fn transform(text: &str, p: &Settings, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.alberti.decode"
    } else {
        "cipher.alberti.encode"
    };
    parameter_size(p, op)?;
    bounded_text(text, p.trace, op)?;
    let mut index = if p.initial_index.len() == 1 {
        INNER
            .iter()
            .position(|&b| b == p.initial_index.as_bytes()[0])
    } else {
        None
    }
    .ok_or_else(|| {
        error(
            op,
            ErrorCode::InvalidParameter,
            "initial_index",
            "select exactly one inner-ring symbol",
        )
    })?;
    let mut r = Report {
        text: String::with_capacity(text.len()),
        final_index: char::from(INNER[index]),
        trace: Vec::with_capacity(if p.trace { text.len() } else { 0 }),
    };
    for (offset, byte) in text.bytes().enumerate() {
        let position = if decode {
            INNER.iter().position(|&b| b == byte)
        } else {
            OUTER.iter().position(|&b| b == byte)
        }
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "text",
                "use exact prepared ring symbols",
            )
            .offset(offset as u64)
        })?;
        let (plain, cipher) = if decode {
            (OUTER[(position + 24 - index) % 24], byte)
        } else {
            (byte, INNER[(position + index) % 24])
        };
        r.text.push(char::from(if decode { plain } else { cipher }));
        if plain.is_ascii_digit() {
            index = INNER
                .iter()
                .position(|&b| b == cipher)
                .expect("selected inner symbol");
        }
        if p.trace {
            r.trace.push(Trace {
                offset: offset as u32,
                index: char::from(INNER[index]),
            });
        }
    }
    r.final_index = char::from(INNER[index]);
    finish_report(r, op)
}
