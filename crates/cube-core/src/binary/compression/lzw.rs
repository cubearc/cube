//! GIF89a and TIFF 6.0 dictionary bitstreams, without container framing.
use super::{
    MIB, WORK_LIMIT,
    bitstream::{Reader, Writer},
    check_limit, malformed, parameter,
};
use crate::{binary::packing::BitOrder, errors::OperationError};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

const ENTRY_LIMIT: u16 = 4096;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(tag = "profile", rename_all = "snake_case", deny_unknown_fields)]
pub enum Profile {
    Gif {
        #[schemars(range(min = 2, max = 8))]
        min_code_size: u8,
    },
    Tiff,
}

#[derive(Clone, Copy)]
struct Spec {
    bits: u8,
    order: BitOrder,
    change: u16,
    clear_at: u16,
}

impl TryFrom<Profile> for Spec {
    type Error = OperationError;

    fn try_from(value: Profile) -> Result<Self, Self::Error> {
        match value {
            Profile::Gif {
                min_code_size: bits @ 2..=8,
            } => Ok(Self {
                bits,
                order: BitOrder::Lsb,
                change: 0,
                clear_at: ENTRY_LIMIT,
            }),
            Profile::Tiff => Ok(Self {
                bits: 8,
                order: BitOrder::Msb,
                change: 1,
                clear_at: ENTRY_LIMIT - 2,
            }),
            _ => {
                Err(parameter("GIF minimum code size must be in 2..=8").parameter("min_code_size"))
            }
        }
    }
}

impl Spec {
    fn clear(self) -> u16 {
        1 << self.bits
    }
    fn end(self) -> u16 {
        self.clear() + 1
    }
    fn first(self) -> u16 {
        self.end() + 1
    }
    fn stream(self) -> Stream {
        Stream {
            spec: self,
            width: self.bits + 1,
            next: self.first(),
            previous: None,
        }
    }
}

struct Stream {
    spec: Spec,
    width: u8,
    next: u16,
    previous: Option<u16>,
}

impl Stream {
    fn advance(&mut self, code: u16) {
        if self.previous.replace(code).is_some() && self.next < ENTRY_LIMIT {
            self.next += 1;
            if self.width < 12 && self.next + self.spec.change == 1 << self.width {
                self.width += 1;
            }
        }
    }

    fn write(&mut self, writer: &mut Writer, code: u16) -> Result<(), OperationError> {
        writer.write(u32::from(code), self.width)?;
        self.advance(code);
        Ok(())
    }
}

/// Emit initial clear and explicit end codes. GIF freezes a full table. TIFF
/// clears before its last two dictionary slots and uses early changes.
pub fn encode(input: &[u8], profile: Profile) -> Result<Vec<u8>, OperationError> {
    check_limit(input.len(), MIB, "LZW source exceeds 1 MiB")?;
    let spec = Spec::try_from(profile)?;
    if input.iter().any(|&value| u16::from(value) >= spec.clear()) {
        return Err(malformed(
            "LZW input symbol is outside the selected alphabet",
        ));
    }
    let mut writer = Writer::new(spec.order, 16 * MIB);
    let mut stream = spec.stream();
    writer.write(u32::from(spec.clear()), stream.width)?;
    if let Some((&head, tail)) = input.split_first() {
        let mut prefix = u16::from(head);
        let mut table = HashMap::new();
        let mut next = spec.first();
        for &byte in tail {
            if let Some(&code) = table.get(&(prefix, byte)) {
                prefix = code;
                continue;
            }
            stream.write(&mut writer, prefix)?;
            if next < ENTRY_LIMIT {
                table.insert((prefix, byte), next);
                next += 1;
            }
            if spec.change == 1 && next == spec.clear_at {
                writer.write(u32::from(spec.clear()), stream.width)?;
                table.clear();
                next = spec.first();
                stream = spec.stream();
            }
            prefix = u16::from(byte);
        }
        stream.write(&mut writer, prefix)?;
    }
    writer.write(u32::from(spec.end()), stream.width)?;
    Ok(writer.finish().0)
}

#[derive(Clone, Copy, Default)]
struct Entry {
    prefix: u16,
    byte: u8,
}

fn phrase(mut code: u16, spec: Spec, table: &[Entry], buffer: &mut Vec<u8>) -> u8 {
    while code >= spec.first() {
        let entry = table[code as usize];
        buffer.push(entry.byte);
        code = entry.prefix;
    }
    let byte = code as u8;
    buffer.push(byte);
    byte
}

/// Expand an exact stream including deferred GIF clears. Accept at most seven
/// residual zero bits following the end code.
pub fn decode(input: &[u8], profile: Profile) -> Result<Vec<u8>, OperationError> {
    check_limit(input.len(), MIB, "LZW compressed input exceeds 1 MiB")?;
    let spec = Spec::try_from(profile)?;
    let mut stream = spec.stream();
    let mut reader = Reader::new(input, input.len() * 8, spec.order);
    let invalid = || malformed("invalid or truncated LZW code stream");
    if reader.read(stream.width) != Some(u32::from(spec.clear())) {
        return Err(invalid());
    }
    let mut table = vec![Entry::default(); ENTRY_LIMIT as usize];
    let mut buffer = Vec::with_capacity(ENTRY_LIMIT as usize);
    let mut output = Vec::new();
    let mut work = 1;
    loop {
        let code = reader.read(stream.width).ok_or_else(invalid)? as u16;
        work += 1;
        if code == spec.end() {
            if !reader.zero_padding() {
                return Err(invalid());
            }
            return Ok(output);
        }
        if code == spec.clear() {
            check_limit(work, WORK_LIMIT, "LZW work limit exceeded")?;
            stream = spec.stream();
            continue;
        }
        buffer.clear();
        let first = if code < stream.next {
            phrase(code, spec, &table, &mut buffer)
        } else if code == stream.next && code < ENTRY_LIMIT {
            let preceding = stream.previous.ok_or_else(invalid)?;
            buffer.push(0);
            let first = phrase(preceding, spec, &table, &mut buffer);
            buffer[0] = first;
            first
        } else {
            return Err(invalid());
        };
        if let Some(prefix) = stream.previous
            && stream.next < ENTRY_LIMIT
        {
            table[stream.next as usize] = Entry {
                prefix,
                byte: first,
            };
        }
        work += buffer.len();
        check_limit(work, WORK_LIMIT, "LZW work limit exceeded")?;
        check_limit(
            output.len() + buffer.len(),
            16 * MIB,
            "LZW output exceeds 16 MiB",
        )?;
        output.extend(buffer.iter().rev());
        stream.advance(code);
    }
}
