//! Siemens T52a/b: XOR followed by the pluggable double-changeover network.
//! Sources: OKM M.Dv.Nr.35 (1937); Weierud, Sturgeon, pp3–5 and21–23.
//! Complete caller-supplied cam facts use logical contacts, not window numbers.
pub use super::lorenz::parse_codes;
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const WHEEL_LENGTHS: [usize; 10] = [47, 53, 59, 61, 64, 65, 67, 69, 71, 73];
const FIRST: [u8; 5] = [1, 8, 6, 4, 2];
const SECOND: [u8; 5] = [10, 9, 7, 5, 3];
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "siemens_t52ab_configured_v1")]
    Configured,
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct CamPattern<'a>(#[schemars(regex(pattern = "^[01]{47,73}$"))] &'a str);
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct Settings {
    pub profile: Profile,
    /// Required identity of the caller's measured/provenanced fixed cam set.
    #[schemars(regex(pattern = "^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$"))]
    pub wheel_set_id: String,
    /// Ascending wheel-length order47,53,59,61,64,65,67,69,71,73.
    #[schemars(with = "[CamPattern<'static>;10]")]
    pub cam_patterns: [String; 10],
    /// Complete permutation of logical wheel IDs0..9: first five select XOR
    /// impulses1..5; last five control the correspondingly ordered socket pairs.
    pub wheel_order: [u8; 10],
    /// Every socket1..10 occurs exactly once, split into five unordered pairs.
    /// Active cam1 passes straight; inactive cam0 crosses between paired plugs.
    pub socket_pairs: [[u8; 2]; 5],
    pub initial_state: [u8; 10],
    #[serde(default)]
    pub trace: bool,
}
#[derive(Clone, Copy, JsonSchema)]
#[serde(transparent)]
struct Impulse(#[schemars(range(max = 4))] u8);
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    #[schemars(range(max = 1048575))]
    pub offset: u32,
    pub state: [u8; 10],
    #[schemars(range(max = 31))]
    pub xor_code: u8,
    /// Input impulse index0..4 -> output impulse index0..4 before decoding.
    #[schemars(with = "[Impulse; 5]")]
    pub permutation: [u8; 5],
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(with = "Vec<super::lorenz::Code>", length(max = 1048576))]
    pub codes: Vec<u8>,
    pub final_state: [u8; 10],
    #[schemars(length(max = 65536))]
    pub trace: Vec<Trace>,
}
struct Machine<'a> {
    patterns: [CamPattern<'a>; 10],
    state: [u8; 10],
    order: [u8; 10],
    permutations: [[Impulse; 5]; 32],
}
impl<'a> Machine<'a> {
    fn configure(p: &'a Settings, op: &'static str) -> Result<Self, OperationError> {
        parameter_size(p, op)?;
        validate_id(&p.wheel_set_id, op)?;
        let patterns = p.cam_patterns.each_ref().map(|s| CamPattern(s));
        for ((pattern, position), length) in patterns.iter().zip(p.initial_state).zip(WHEEL_LENGTHS)
        {
            if pattern.0.len() != length || !pattern.0.bytes().all(|b| matches!(b, b'0' | b'1')) {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "cam_patterns",
                    "use exact binary patterns in ascending wheel-length order",
                ));
            }
            if usize::from(position) >= length {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "initial_state",
                    "logical contact index exceeds wheel length",
                ));
            }
        }
        let mut seen = 0u16;
        for wheel in p.wheel_order {
            if wheel > 9 || seen & (1 << wheel) != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "wheel_order",
                    "use each logical wheel ID0..9 exactly once",
                ));
            }
            seen |= 1 << wheel;
        }
        let permutations = switch_network(&p.socket_pairs, op)?;
        Ok(Self {
            patterns,
            state: p.initial_state,
            order: p.wheel_order,
            permutations,
        })
    }
    fn signals(&self) -> (u8, u8) {
        let mut xor = 0;
        let mut control = 0;
        for (channel, wheel) in self.order.iter().enumerate() {
            let index = usize::from(*wheel);
            let bit =
                u8::from(self.patterns[index].0.as_bytes()[usize::from(self.state[index])] == b'1');
            if channel < 5 {
                xor |= bit << channel;
            } else {
                control |= bit << (channel - 5);
            }
        }
        (xor, control)
    }
    fn advance(&mut self) {
        for (index, length) in self.state.iter_mut().zip(WHEEL_LENGTHS) {
            *index = ((*index as usize + 1) % length) as u8;
        }
    }
}
fn validate_id(id: &str, op: &'static str) -> Result<(), OperationError> {
    if id.is_empty()
        || id.len() > 128
        || !id.as_bytes()[0].is_ascii_alphanumeric()
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "wheel_set_id",
            "required bounded cam-set identity uses ASCII letters, digits, dot, underscore or hyphen",
        ));
    }
    Ok(())
}
fn switch_network(
    pairs: &[[u8; 2]; 5],
    op: &'static str,
) -> Result<[[Impulse; 5]; 32], OperationError> {
    let mut partner = [0u8; 11];
    let mut unit = [0usize; 11];
    for (index, pair) in pairs.iter().enumerate() {
        for socket in pair {
            if !(1..=10).contains(socket) || partner[usize::from(*socket)] != 0 {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "socket_pairs",
                    "use every socket1..10 exactly once",
                ));
            }
            // Mark both membership and partner; a self-pair is rejected on its
            // second appearance without permitting an unconnected socket.
            partner[usize::from(*socket)] = if *socket == pair[0] { pair[1] } else { pair[0] };
            unit[usize::from(*socket)] = index;
        }
    }
    let mut result = [[Impulse(0); 5]; 32];
    for (control, map) in result.iter_mut().enumerate() {
        for (input, socket) in FIRST.iter().enumerate() {
            map[input] = Impulse(follow(*socket, control, &partner, &unit, op)?);
        }
        let mask = map.iter().fold(0u8, |mask, output| mask | (1 << output.0));
        if mask != 31 {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "socket_pairs",
                "switch circuit must connect each impulse to one distinct output",
            ));
        }
    }
    Ok(result)
}
fn follow(
    mut socket: u8,
    control: usize,
    partner: &[u8; 11],
    unit: &[usize; 11],
    op: &'static str,
) -> Result<u8, OperationError> {
    // Five input paths cannot enter a closed internal circuit: all left and
    // right contacts have one connection. At most ten sockets are traversed.
    for _ in 0..10 {
        let current = usize::from(socket);
        let next = if control & (1 << unit[current]) != 0 {
            socket
        } else {
            partner[current]
        };
        if let Some(output) = SECOND.iter().position(|s| *s == next) {
            return Ok(output as u8);
        }
        let row = FIRST.iter().position(|s| *s == next).ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidParameter,
                "socket_pairs",
                "invalid switch contact",
            )
        })?;
        socket = SECOND[row];
    }
    Err(error(
        op,
        ErrorCode::InvalidParameter,
        "socket_pairs",
        "switch path did not reach an output",
    ))
}
fn permute(code: u8, map: [Impulse; 5], inverse: bool) -> u8 {
    map.iter().enumerate().fold(0, |result, (input, output)| {
        let (source, target) = if inverse {
            (usize::from(output.0), input)
        } else {
            (input, usize::from(output.0))
        };
        result | (((code >> source) & 1) << target)
    })
}
/// Encipher unpacked0..31 codes with explicit T52a/b fixed cam facts and plugs.
/// Current contacts feed XOR then permutation; all ten wheels advance afterward.
pub fn encode(codes: &[u8], settings: &Settings) -> Result<Report, OperationError> {
    transform(codes, settings, false)
}
/// Undo the actual switch permutation first, then XOR with the same stream.
/// Use the sender's original state; final_state supports independent chunks.
pub fn decode(codes: &[u8], settings: &Settings) -> Result<Report, OperationError> {
    transform(codes, settings, true)
}
fn transform(codes: &[u8], p: &Settings, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.t52ab.decode"
    } else {
        "cipher.t52ab.encode"
    };
    super::lorenz::input_budget(codes.len(), op)?;
    if p.trace && codes.len() > MAX_TRACE {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "trace",
            "trace exceeds65536 input symbols",
        ));
    }
    let mut machine = Machine::configure(p, op)?;
    let mut output = Vec::with_capacity(codes.len());
    let mut trace = Vec::new();
    for (offset, code) in codes.iter().enumerate() {
        super::lorenz::Code::parse(*code, op, offset)?;
        let (xor, control) = machine.signals();
        let permutation = machine.permutations[usize::from(control)];
        output.push(if decode {
            permute(*code, permutation, true) ^ xor
        } else {
            permute(*code ^ xor, permutation, false)
        });
        machine.advance();
        if p.trace {
            trace.push(Trace {
                offset: offset as u32,
                state: machine.state,
                xor_code: xor,
                permutation: permutation.map(|i| i.0),
            });
        }
    }
    finish_report(
        Report {
            codes: output,
            final_state: machine.state,
            trace,
        },
        op,
    )
}
