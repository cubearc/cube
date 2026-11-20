//! Explicit reproducible integer and bit-stream profiles.
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MT_WORDS: usize = 624;
const PRNG_WORK: usize = 4 << 20;
const LFSR_WORK: usize = 16 << 20;

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Packing {
    MsbFirst,
    LsbFirst,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tail {
    Reject,
    PadZero,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LfsrRequest {
    #[schemars(range(min = 2, max = 64))]
    pub width: u8,
    pub state: u64,
    pub tap_mask: u64,
    #[schemars(range(max = 1048576))]
    pub count: usize,
    pub packing: Packing,
    pub tail: Tail,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct LfsrReport {
    pub hex: String,
    pub valid_bits: usize,
    pub final_state: u64,
    pub packing: Packing,
    pub work: usize,
}

/// Emit the old LSB, then update with the parity of the masked old state.
pub fn lfsr(request: &LfsrRequest) -> Result<LfsrReport, OperationError> {
    const OP: &str = "number.lfsr";
    lfsr_request(request).map_err(|e| e.operation(OP))
}

fn lfsr_request(request: &LfsrRequest) -> Result<LfsrReport, OperationError> {
    let LfsrRequest {
        width,
        mut state,
        tap_mask,
        count,
        packing,
        tail,
        max_work,
    } = *request;
    if !(2..=64).contains(&width) {
        return Err(parameter("LFSR width must be 2 through 64"));
    }
    let fits = |value: u64| value != 0 && (width == 64 || value < 1_u64 << width);
    if !fits(state) || !fits(tap_mask) {
        return Err(invalid(
            "LFSR requires nonzero state and mask within the selected width",
        ));
    }
    if count > 1 << 20 {
        return Err(resource("LFSR count exceeds 1048576 bits"));
    }
    if matches!(tail, Tail::Reject) && !count.is_multiple_of(8) {
        return Err(parameter(
            "reject tail policy requires a whole number of output bytes",
        ));
    }
    let mut cancelled = || false;
    let mut work = Work::new(max_work, LFSR_WORK, &mut cancelled)?;
    work.charge(count).map_err(Stop::error)?;
    let mut bytes = vec![0_u8; count.div_ceil(8)];
    for i in 0..count {
        let offset = match packing {
            Packing::MsbFirst => 7 - i % 8,
            Packing::LsbFirst => i % 8,
        };
        bytes[i / 8] |= ((state & 1) as u8) << offset;
        let feedback = u64::from((state & tap_mask).count_ones() & 1);
        state = (state >> 1) | (feedback << (width - 1));
    }
    Ok(LfsrReport {
        hex: hex::encode(bytes),
        valid_bits: count,
        final_state: state,
        packing,
        work: work.used(),
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "profile", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    #[serde(rename = "mt19937ar_2002")]
    Mt19937 {
        seed: Option<u32>,
        state: Option<MtState>,
    },
    Xorshift32 {
        seed: Option<u32>,
        state: Option<u32>,
    },
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MtState {
    #[schemars(length(equal = 624))]
    pub words: Vec<u32>,
    #[schemars(range(max = 624))]
    pub cursor: usize,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrngRequest {
    pub source: Source,
    #[schemars(range(max = 65536))]
    pub count: usize,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "profile", rename_all = "snake_case")]
pub enum PrngState {
    #[serde(rename = "mt19937ar_2002")]
    Mt19937 {
        state: MtState,
    },
    Xorshift32 {
        state: u32,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct PrngReport {
    pub values: Vec<u32>,
    pub final_state: PrngState,
    pub work: usize,
}

struct Mt19937 {
    words: [u32; MT_WORDS],
    cursor: usize,
}

impl Mt19937 {
    fn seed(seed: u32) -> Self {
        let mut words = [seed; MT_WORDS];
        for i in 1..MT_WORDS {
            let old = words[i - 1];
            words[i] = 1812433253_u32
                .wrapping_mul(old ^ (old >> 30))
                .wrapping_add(i as u32);
        }
        Self {
            words,
            cursor: MT_WORDS,
        }
    }

    fn restore(state: &MtState) -> Result<Self, OperationError> {
        let words: [u32; MT_WORDS] = state
            .words
            .as_slice()
            .try_into()
            .map_err(|_| invalid("MT19937 state requires exactly 624 words"))?;
        if state.cursor > MT_WORDS || words.iter().all(|&w| w == 0) {
            return Err(invalid(
                "MT19937 state requires a nonzero state and cursor 0 through 624",
            ));
        }
        Ok(Self {
            words,
            cursor: state.cursor,
        })
    }

    fn next(&mut self, work: &mut Work<'_>) -> Result<u32, OperationError> {
        if self.cursor == MT_WORDS {
            work.charge(MT_WORDS).map_err(Stop::error)?;
            for i in 0..MT_WORDS {
                let joined =
                    (self.words[i] & 0x80000000) | (self.words[(i + 1) % MT_WORDS] & 0x7fffffff);
                let odd = if joined & 1 == 0 { 0 } else { 0x9908b0df };
                self.words[i] = self.words[(i + 397) % MT_WORDS] ^ (joined >> 1) ^ odd;
            }
            self.cursor = 0;
        }
        let mut word = self.words[self.cursor];
        self.cursor += 1;
        word ^= word >> 11;
        word ^= (word << 7) & 0x9d2c5680;
        word ^= (word << 15) & 0xefc60000;
        Ok(word ^ (word >> 18))
    }
}

/// Generate reproducible u32 words and the explicit state for the next call.
pub fn prng(request: &PrngRequest) -> Result<PrngReport, OperationError> {
    prng_request(request).map_err(|e| e.operation("number.prng"))
}

fn prng_request(request: &PrngRequest) -> Result<PrngReport, OperationError> {
    if request.count > 65536 {
        return Err(resource("PRNG count exceeds 65536 words"));
    }
    let mut cancelled = || false;
    let mut work = Work::new(request.max_work, PRNG_WORK, &mut cancelled)?;
    work.charge(request.count).map_err(Stop::error)?;
    let mut values = Vec::with_capacity(request.count);
    let final_state = match &request.source {
        Source::Mt19937 { seed, state } => {
            work.charge(MT_WORDS).map_err(Stop::error)?;
            let mut rng = match (seed, state) {
                (Some(seed), None) => Mt19937::seed(*seed),
                (None, Some(state)) => Mt19937::restore(state)?,
                _ => return Err(parameter("provide exactly one of seed or state")),
            };
            for _ in 0..request.count {
                values.push(rng.next(&mut work)?);
            }
            PrngState::Mt19937 {
                state: MtState {
                    words: rng.words.to_vec(),
                    cursor: rng.cursor,
                },
            }
        }
        Source::Xorshift32 { seed, state } => {
            let mut state = match (seed, state) {
                (Some(value), None) | (None, Some(value)) if *value != 0 => *value,
                (Some(0), None) | (None, Some(0)) => {
                    return Err(invalid("xorshift32 requires a nonzero seed or state"));
                }
                _ => return Err(parameter("provide exactly one of seed or state")),
            };
            work.charge(1).map_err(Stop::error)?;
            for _ in 0..request.count {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                values.push(state);
            }
            PrngState::Xorshift32 { state }
        }
    };
    Ok(PrngReport {
        values,
        final_state,
        work: work.used(),
    })
}
