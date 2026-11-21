//! Fixed-model Witten–Neal–Cleary arithmetic and carry-buffered range coding.
use super::{
    bitstream::{Reader, Writer},
    check_limit, malformed, parameter,
};
use crate::{binary::packing::BitOrder, errors::OperationError};
use schemars::JsonSchema;
use serde::Deserialize;

pub const MAX_SYMBOLS: usize = 65_536;
pub const MAX_WIRE_BYTES: usize = 4 * MAX_SYMBOLS + 5;
const MAX_TOTAL: u64 = 16_383;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// 16-bit inclusive intervals and E1/E2/E3 renormalization.
    Arithmetic16,
    /// 32-bit half-open byte range and a delayed carry.
    Range32,
}

struct Model {
    bounds: [u32; 257],
    total: u32,
}

impl Model {
    fn new(frequencies: &[u64; 256]) -> Result<Self, OperationError> {
        let invalid =
            || parameter("entropy model total must be in 1..=16383").parameter("frequencies");
        let mut bounds = [0; 257];
        let mut total = 0u64;
        for (i, &weight) in frequencies.iter().enumerate() {
            total = total
                .checked_add(weight)
                .filter(|&sum| sum <= MAX_TOTAL)
                .ok_or_else(invalid)?;
            bounds[i + 1] = total as u32;
        }
        if total == 0 {
            return Err(invalid());
        }
        Ok(Self {
            bounds,
            total: total as u32,
        })
    }

    fn interval(&self, symbol: u8) -> Result<(u32, u32), OperationError> {
        let start = self.bounds[symbol as usize];
        let end = self.bounds[symbol as usize + 1];
        if start == end {
            return Err(malformed(
                "input symbol is absent from the probability model",
            ));
        }
        Ok((start, end))
    }

    fn symbol(&self, value: u32) -> Result<(u8, u32, u32), OperationError> {
        if value >= self.total {
            return Err(malformed("probability stream falls outside its model"));
        }
        let i = self.bounds.partition_point(|&v| v <= value) - 1;
        Ok((i as u8, self.bounds[i], self.bounds[i + 1]))
    }
}

const HALF: u32 = 0x8000;
const QUARTER: u32 = HALF / 2;
const THREE_QUARTERS: u32 = 3 * QUARTER;

struct Arithmetic {
    low: u32,
    high: u32,
}

impl Arithmetic {
    fn initial() -> Self {
        Self {
            low: 0,
            high: 2 * HALF - 1,
        }
    }

    fn select(&mut self, from: u32, to: u32, total: u32) {
        let width = self.high - self.low + 1;
        self.high = self.low + width * to / total - 1;
        self.low += width * from / total;
    }

    fn renormalize(&mut self) -> Option<u32> {
        let origin = if self.high < HALF {
            0
        } else if self.low >= HALF {
            HALF
        } else if self.low >= QUARTER && self.high < THREE_QUARTERS {
            QUARTER
        } else {
            return None;
        };
        self.low = (self.low - origin) * 2;
        self.high = (self.high - origin) * 2 + 1;
        Some(origin)
    }
}

fn emit(writer: &mut Writer, bit: u32, pending: &mut usize) -> Result<(), OperationError> {
    writer.write(bit, 1)?;
    for _ in 0..std::mem::take(pending) {
        writer.write(bit ^ 1, 1)?;
    }
    Ok(())
}

fn arithmetic_encode(data: &[u8], model: &Model) -> Result<Vec<u8>, OperationError> {
    let mut state = Arithmetic::initial();
    let mut output = Writer::new(BitOrder::Msb, MAX_WIRE_BYTES);
    let mut pending = 0;
    for &symbol in data {
        let (a, b) = model.interval(symbol)?;
        state.select(a, b, model.total);
        while let Some(origin) = state.renormalize() {
            if origin == QUARTER {
                pending += 1;
            } else {
                emit(&mut output, origin / HALF, &mut pending)?;
            }
        }
    }
    pending += 1;
    emit(&mut output, u32::from(state.low >= QUARTER), &mut pending)?;
    Ok(output.finish().0)
}

fn arithmetic_decode(data: &[u8], count: usize, model: &Model) -> Result<Vec<u8>, OperationError> {
    let mut state = Arithmetic::initial();
    let mut bits = Reader::new(data, data.len() * 8, BitOrder::Msb);
    // Witten–Neal–Cleary decoders extend the flushed prefix with zero bits.
    let mut next = || bits.read(1).unwrap_or(0);
    let mut code = (0..16).fold(0, |value, _| (value << 1) | next());
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        let width = state.high - state.low + 1;
        let scaled = ((code - state.low + 1) * model.total - 1) / width;
        let (symbol, a, b) = model.symbol(scaled)?;
        output.push(symbol);
        state.select(a, b, model.total);
        while let Some(origin) = state.renormalize() {
            code = ((code - origin) << 1) | next();
        }
    }
    Ok(output)
}

const TOP: u32 = 1 << 24;

struct RangeEncoder {
    low: u64,
    width: u32,
    last: u8,
    pending: usize,
    output: Vec<u8>,
}

impl RangeEncoder {
    // A carry can still change a preceding byte and every subsequent 0xff.
    fn emit_byte(&mut self) {
        let word = self.low as u32;
        let carry = (self.low >> 32) as u8;
        if word < 0xff00_0000 || carry != 0 {
            self.output.push(self.last.wrapping_add(carry));
            let count = self.output.len() + self.pending - 1;
            self.output.resize(count, 0xffu8.wrapping_add(carry));
            self.last = (word >> 24) as u8;
            self.pending = 0;
        }
        self.pending += 1;
        self.low = u64::from(word << 8);
    }
}

fn range_encode(input: &[u8], model: &Model) -> Result<Vec<u8>, OperationError> {
    let mut ctx = RangeEncoder {
        low: 0,
        width: u32::MAX,
        last: 0,
        pending: 1,
        output: Vec::new(),
    };
    for &symbol in input {
        let (a, b) = model.interval(symbol)?;
        let unit = ctx.width / model.total;
        ctx.low += u64::from(unit * a);
        ctx.width = unit * (b - a);
        while ctx.width < TOP {
            ctx.width <<= 8;
            ctx.emit_byte();
        }
    }
    for _ in 0..5 {
        ctx.emit_byte();
    }
    Ok(ctx.output)
}

fn range_decode(data: &[u8], count: usize, model: &Model) -> Result<Vec<u8>, OperationError> {
    let invalid = || malformed("truncated carry-buffered range stream");
    let mut source = data.iter().copied();
    if source.next() != Some(0) {
        return Err(invalid());
    }
    let mut next = || source.next().map(u32::from).ok_or_else(invalid);
    let mut code = 0;
    for _ in 0..4 {
        code = (code << 8) | next()?;
    }
    let mut width = u32::MAX;
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        let unit = width / model.total;
        let (symbol, a, b) = model.symbol(code / unit)?;
        output.push(symbol);
        code -= unit * a;
        width = unit * (b - a);
        while width < TOP {
            width <<= 8;
            code = (code << 8) | next()?;
        }
    }
    Ok(output)
}

/// Encode one canonical raw probability stream using explicit fixed counts.
pub fn encode(
    data: &[u8],
    frequencies: &[u64; 256],
    profile: Profile,
) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MAX_SYMBOLS, "probability source exceeds 64 KiB")?;
    let model = Model::new(frequencies)?;
    match profile {
        Profile::Arithmetic16 => arithmetic_encode(data, &model),
        Profile::Range32 => range_encode(data, &model),
    }
}

/// Decode the supplied symbol count and verify canonical re-encoding. This
/// checks framing and termination. It provides no independent payload checksum.
pub fn decode(
    data: &[u8],
    frequencies: &[u64; 256],
    count: usize,
    profile: Profile,
) -> Result<Vec<u8>, OperationError> {
    check_limit(
        data.len(),
        MAX_WIRE_BYTES,
        "probability wire input exceeds its bound",
    )?;
    check_limit(count, MAX_SYMBOLS, "probability output exceeds 64 KiB")?;
    let model = Model::new(frequencies)?;
    let (plain, canonical) = match profile {
        Profile::Arithmetic16 => {
            let plain = arithmetic_decode(data, count, &model)?;
            let canonical = arithmetic_encode(&plain, &model)?;
            (plain, canonical)
        }
        Profile::Range32 => {
            let plain = range_decode(data, count, &model)?;
            let canonical = range_encode(&plain, &model)?;
            (plain, canonical)
        }
    };
    if canonical != data {
        return Err(malformed(
            "probability stream has noncanonical termination or trailing data",
        ));
    }
    Ok(plain)
}
