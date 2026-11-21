//! Bounded LZ77 and LZ78 token transforms.
use super::{MIB, WORK_LIMIT, check_limit, exhausted, malformed, parameter};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MAX_SOURCE: usize = 65_536;
pub const MAX_TOKENS: usize = 65_536;
pub const DEFAULT_WINDOW: u16 = 4096;
const MATCH_MIN: usize = 3;
const MATCH_MAX: usize = 258;

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Lz77Token {
    Literal {
        byte: u8,
    },
    Match {
        #[schemars(range(min = 1, max = 4096))]
        distance: u16,
        #[schemars(range(min = 3, max = 258))]
        length: u16,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Lz78Token {
    #[schemars(range(max = 65_535))]
    pub index: u16,
    pub byte: Option<u8>,
}

fn window(window: u16) -> Result<usize, OperationError> {
    if !(1..=DEFAULT_WINDOW).contains(&window) {
        return Err(parameter("LZ77 window must be in 1..=4096").parameter("window"));
    }
    Ok(window as usize)
}

fn key(data: &[u8], i: usize) -> Option<[u8; 3]> {
    data.get(i..i + MATCH_MIN)
        .map(|key| key.try_into().expect("three byte key"))
}

fn search(
    data: &[u8],
    start: usize,
    range: usize,
    mut prior: Option<usize>,
    links: &[Option<usize>],
    work: &mut usize,
) -> Result<(usize, usize), OperationError> {
    let mut best = (0, 0);
    let max = MATCH_MAX.min(data.len() - start);
    while let Some(position) = prior {
        let distance = start - position;
        if distance > range {
            break;
        }
        let mut length = MATCH_MIN;
        while length < max && data[start + length] == data[position + length] {
            length += 1;
        }
        *work += length;
        check_limit(*work, WORK_LIMIT, "LZ77 search exceeds the work budget")?;
        if length > best.1 {
            best = (distance, length);
        }
        if length == max {
            break;
        }
        prior = links[position];
    }
    Ok(best)
}

/// Emit literal and overlapping match tokens. Prefer the longest match and
/// then the smallest distance.
pub fn lz77_encode(data: &[u8], window_size: u16) -> Result<Vec<Lz77Token>, OperationError> {
    let window = window(window_size)?;
    check_limit(data.len(), MAX_SOURCE, "LZ77 source exceeds 64 KiB")?;
    let mut heads = HashMap::new();
    let mut links = vec![None; data.len()];
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut work = data.len();
    while start < data.len() {
        let prior = key(data, start).and_then(|bytes| heads.get(&bytes).copied());
        let (distance, length) = search(data, start, window, prior, &links, &mut work)?;
        let width = if length >= MATCH_MIN {
            tokens.push(Lz77Token::Match {
                distance: distance as u16,
                length: length as u16,
            });
            length
        } else {
            tokens.push(Lz77Token::Literal { byte: data[start] });
            1
        };
        for (i, link) in links.iter_mut().enumerate().skip(start).take(width) {
            if let Some(bytes) = key(data, i) {
                *link = heads.insert(bytes, i);
            }
        }
        start += width;
    }
    Ok(tokens)
}

/// Decode explicit LZ77 tokens, checking distance and size before copying.
pub fn lz77_decode(tokens: &[Lz77Token], window_size: u16) -> Result<Vec<u8>, OperationError> {
    let window = window(window_size)?;
    check_limit(tokens.len(), MAX_TOKENS, "LZ77 token limit exceeded")?;
    let mut total = 0;
    for token in tokens {
        let width = match *token {
            Lz77Token::Literal { .. } => 1,
            Lz77Token::Match { distance, length } => {
                let distance = distance as usize;
                let length = length as usize;
                if distance == 0
                    || distance > total.min(window)
                    || !(MATCH_MIN..=MATCH_MAX).contains(&length)
                {
                    return Err(malformed("invalid LZ77 distance or match length"));
                }
                length
            }
        };
        total += width;
        check_limit(total, MIB, "LZ77 decoded output exceeds 1 MiB")?;
    }
    let mut output = Vec::with_capacity(total);
    for token in tokens {
        match *token {
            Lz77Token::Literal { byte } => output.push(byte),
            Lz77Token::Match { distance, length } => {
                for _ in 0..length {
                    output.push(output[output.len() - distance as usize]);
                }
            }
        }
    }
    Ok(output)
}

/// Emit LZ78 dictionary references with an absent byte only for a final suffix.
pub fn lz78_encode(data: &[u8]) -> Result<Vec<Lz78Token>, OperationError> {
    check_limit(data.len(), MAX_SOURCE, "LZ78 source exceeds 64 KiB")?;
    let mut map = HashMap::new();
    let mut tokens = Vec::new();
    let mut index = 0;
    for &byte in data {
        let next =
            u16::try_from(tokens.len() + 1).map_err(|_| exhausted("LZ78 dictionary is full"))?;
        match map.entry((index, byte)) {
            std::collections::hash_map::Entry::Occupied(known) => index = *known.get(),
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(next);
                tokens.push(Lz78Token {
                    index,
                    byte: Some(byte),
                });
                index = 0;
            }
        }
    }
    if index != 0 {
        tokens.push(Lz78Token { index, byte: None });
    }
    Ok(tokens)
}

/// Decode LZ78 tokens with a cumulative expansion bound and no dictionary reset.
pub fn lz78_decode(tokens: &[Lz78Token]) -> Result<Vec<u8>, OperationError> {
    check_limit(tokens.len(), MAX_TOKENS, "LZ78 token limit exceeded")?;
    let mut lengths = Vec::with_capacity(tokens.len() + 1);
    lengths.push(0);
    let mut total = 0;
    for (i, token) in tokens.iter().enumerate() {
        let index = token.index as usize;
        if index > i || (token.byte.is_none() && (i + 1 != tokens.len() || index == 0)) {
            return Err(malformed("invalid LZ78 index or terminal suffix"));
        }
        let length = lengths[index] + usize::from(token.byte.is_some());
        lengths.push(length);
        total += length;
        check_limit(total, MIB, "LZ78 decoded output exceeds 1 MiB")?;
    }
    let mut output = Vec::with_capacity(total);
    for token in tokens {
        let offset = output.len();
        let mut current = token;
        loop {
            if let Some(byte) = current.byte {
                output.push(byte);
            }
            if current.index == 0 {
                break;
            }
            current = &tokens[current.index as usize - 1];
        }
        output[offset..].reverse();
    }
    Ok(output)
}
