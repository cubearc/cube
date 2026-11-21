//! Deterministic binary Huffman lengths and MSB-first canonical byte codes.
use super::{
    MIB, WORK_LIMIT,
    bitstream::{Reader, Writer},
    check_limit, exhausted, malformed, parameter,
};
use crate::{binary::packing::BitOrder, errors::OperationError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BinaryHeap};

const MAX_BITS: u8 = 32;
const MAX_OUTPUT: usize = 8 * MIB;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Length {
    pub symbol: u8,
    #[schemars(range(min = 1, max = 32))]
    pub length: u8,
}

#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct Code {
    pub symbol: u8,
    pub length: u8,
    pub code: u32,
}

#[derive(Debug)]
pub struct Encoded {
    pub data: Vec<u8>,
    pub bit_count: usize,
    pub codes: Vec<Code>,
}

enum Node {
    Leaf(u8),
    Pair(usize, usize),
}

/// Construct lengths from the exact 256-element table. Zero means absent.
pub fn construct(frequencies: &[u64; 256]) -> Result<Vec<Code>, OperationError> {
    let mut heap = BinaryHeap::new();
    let mut nodes = Vec::new();
    for (symbol, &count) in frequencies
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
    {
        let symbol = symbol as u8;
        heap.push(Reverse((count, symbol, nodes.len())));
        nodes.push(Node::Leaf(symbol));
    }
    while heap.len() > 1 {
        let Reverse((first, a, left)) = heap.pop().expect("nonempty heap");
        let Reverse((second, b, right)) = heap.pop().expect("second node");
        let total = first.checked_add(second).ok_or_else(|| {
            parameter("Huffman frequencies overflow u64").parameter("frequencies")
        })?;
        heap.push(Reverse((total, a.min(b), nodes.len())));
        nodes.push(Node::Pair(left, right));
    }
    let mut lengths = Vec::new();
    if let Some(Reverse((_, _, root))) = heap.pop() {
        let mut stack = vec![(root, 0)];
        while let Some((index, depth)) = stack.pop() {
            if depth > MAX_BITS {
                return Err(exhausted("optimal Huffman tree exceeds 32-bit codes"));
            }
            match nodes[index] {
                Node::Leaf(symbol) => lengths.push(Length {
                    symbol,
                    length: depth.max(1),
                }),
                Node::Pair(left, right) => {
                    stack.push((left, depth + 1));
                    stack.push((right, depth + 1));
                }
            }
        }
    }
    canonical(&lengths)
}

/// Validate an externally provided length table and return canonical codes.
pub fn canonical(lengths: &[Length]) -> Result<Vec<Code>, OperationError> {
    check_limit(lengths.len(), 256, "Huffman alphabet exceeds 256 symbols")?;
    let invalid = || parameter("invalid canonical Huffman code lengths").parameter("lengths");
    let mut seen = [false; 256];
    for entry in lengths {
        if !(1..=MAX_BITS).contains(&entry.length)
            || std::mem::replace(&mut seen[entry.symbol as usize], true)
        {
            return Err(invalid());
        }
    }
    if lengths.len() == 1 && lengths[0].length != 1 {
        return Err(invalid());
    }
    let mut sorted = lengths.to_vec();
    sorted.sort_unstable_by_key(|entry| (entry.length, entry.symbol));
    let mut code = 0u64;
    let mut previous = 0;
    let mut result = Vec::with_capacity(sorted.len());
    for Length { symbol, length } in sorted {
        code <<= length - previous;
        if code >= 1u64 << length {
            return Err(invalid());
        }
        result.push(Code {
            symbol,
            length,
            code: code as u32,
        });
        code += 1;
        previous = length;
    }
    if result.len() > 1 && code != 1u64 << previous {
        return Err(invalid());
    }
    Ok(result)
}

/// Encode bytes with a supplied frequency model. Return the exact bit count.
pub fn encode(data: &[u8], frequencies: &[u64; 256]) -> Result<Encoded, OperationError> {
    check_limit(data.len(), MIB, "Huffman source exceeds 1 MiB")?;
    let codes = construct(frequencies)?;
    let mut by_symbol = [None; 256];
    for &code in &codes {
        by_symbol[code.symbol as usize] = Some(code);
    }
    let mut writer = Writer::new(BitOrder::Msb, MAX_OUTPUT);
    for &symbol in data {
        let code = by_symbol[symbol as usize]
            .ok_or_else(|| malformed("input symbol is absent from the Huffman alphabet"))?;
        writer.write(code.code, code.length)?;
    }
    let (data, bit_count) = writer.finish();
    Ok(Encoded {
        data,
        bit_count,
        codes,
    })
}

/// Decode exactly the declared bit and symbol counts with zero byte padding.
pub fn decode(
    data: &[u8],
    lengths: &[Length],
    bit_count: usize,
    symbol_count: usize,
) -> Result<Vec<u8>, OperationError> {
    check_limit(data.len(), MIB, "Huffman input exceeds 1 MiB")?;
    check_limit(bit_count, WORK_LIMIT, "Huffman bit work limit exceeded")?;
    check_limit(symbol_count, MAX_OUTPUT, "Huffman output exceeds 8 MiB")?;
    let invalid = || malformed("Huffman bits do not match the declared stream");
    if data.len() != bit_count.div_ceil(8)
        || symbol_count > bit_count
        || bit_count > symbol_count * MAX_BITS as usize
    {
        return Err(invalid());
    }
    let codes = canonical(lengths)?;
    let mut spans = [(0, 0); 33];
    for (index, entry) in codes.iter().enumerate() {
        let (start, count) = &mut spans[entry.length as usize];
        if *count == 0 {
            *start = index;
        }
        *count += 1;
    }
    let mut reader = Reader::new(data, bit_count, BitOrder::Msb);
    let mut output = Vec::with_capacity(symbol_count);
    for _ in 0..symbol_count {
        let mut word = 0;
        let mut found = None;
        for &(start, count) in &spans[1..] {
            word = (word << 1) | reader.read(1).ok_or_else(invalid)?;
            if count > 0
                && let Some(offset) = word.checked_sub(codes[start].code)
                && (offset as usize) < count
            {
                found = Some(codes[start + offset as usize].symbol);
                break;
            }
        }
        output.push(found.ok_or_else(invalid)?);
    }
    if reader.remaining() != 0 || !reader.zero_padding() {
        return Err(invalid());
    }
    Ok(output)
}
