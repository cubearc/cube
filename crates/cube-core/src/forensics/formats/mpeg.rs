//! ID3v2.3/v2.4 metadata and bounded MPEG Layer III frame extents.
use super::*;
use std::borrow::Cow;

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct TagFrame {
    pub id: String,
    pub range: Extent,
    pub payload: Extent,
    pub flags: u16,
    pub text: Option<String>,
    pub unsupported_payload: bool,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Tag {
    pub version: &'static str,
    pub range: Extent,
    pub extended_header: Option<Extent>,
    pub footer: Option<Extent>,
    pub unsynchronized: bool,
    pub frames: Vec<TagFrame>,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct AudioFrame {
    pub range: Extent,
    pub version: &'static str,
    pub sample_rate: u32,
    pub bitrate_kbps: u32,
    pub channel_mode: &'static str,
    pub samples_per_frame: u16,
    pub crc_present: bool,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub tag: Option<Tag>,
    pub frames: Vec<AudioFrame>,
    pub trailing_id3v1: Option<Extent>,
    pub payload_verified: bool,
}

fn syncsafe(data: &[u8]) -> Result<usize, OperationError> {
    if data.iter().any(|&b| b & 0x80 != 0) {
        return Err(malformed("invalid ID3 synchsafe integer"));
    }
    Ok(data
        .iter()
        .fold(0, |result, &byte| (result << 7) | byte as usize))
}

fn be32(data: &[u8]) -> usize {
    u32::from_be_bytes(data[..4].try_into().expect("checked ID3 header")) as usize
}

struct Logical<'a> {
    data: Cow<'a, [u8]>,
    offsets: Vec<u32>,
}

impl<'a> Logical<'a> {
    fn parse(raw: &'a [u8], transform: bool) -> Self {
        if !transform || !raw.windows(2).any(|pair| pair == [0xff, 0]) {
            return Self {
                data: Cow::Borrowed(raw),
                offsets: Vec::new(),
            };
        }
        let mut data = Vec::with_capacity(raw.len());
        let mut offsets = Vec::with_capacity(raw.len() + 1);
        let mut i = 0;
        while i < raw.len() {
            let byte = raw[i];
            offsets.push(i as u32);
            data.push(byte);
            i += 1;
            if byte == 0xff && raw.get(i) == Some(&0) {
                i += 1;
            }
        }
        offsets.push(raw.len() as u32);
        Self {
            data: Cow::Owned(data),
            offsets,
        }
    }

    fn tag_range(&self, start: usize, end: usize) -> Extent {
        let physical = |i| self.offsets.get(i).copied().map_or(i, |v| v as usize);
        Extent::new(10 + physical(start), physical(end) - physical(start))
    }
}

fn extended(data: &[u8], version: u8) -> Result<(usize, usize), OperationError> {
    let size = slice(data, 0, 4)?;
    if version == 3 {
        let length = be32(size);
        if !matches!(length, 6 | 10) {
            return Err(malformed("invalid ID3v2.3 extended header size"));
        }
        let header = slice(data, 4, length)?;
        let flags = u16::from_be_bytes([header[0], header[1]]);
        if flags & 0x7fff != 0 || (flags & 0x8000 != 0) != (length == 10) {
            return Err(malformed("invalid ID3v2.3 extended header flags"));
        }
        Ok((4 + length, be32(&header[2..])))
    } else {
        let length = syncsafe(size)?;
        let header = slice(data, 0, length)?;
        if length < 6 || header[4] != 1 || header[5] & !0x70 != 0 {
            return Err(malformed("invalid ID3v2.4 extended header"));
        }
        let mut i = 6;
        for (flag, width) in [(0x40, 0), (0x20, 5), (0x10, 1)] {
            if header[5] & flag != 0 {
                let field = slice(header, i, 1 + width)?;
                if field[0] != width as u8 {
                    return Err(malformed("invalid ID3 extended field size"));
                }
                if flag == 0x20 && (field[1] > 15 || syncsafe(&field[1..]).is_err()) {
                    return Err(malformed("invalid ID3 extended CRC encoding"));
                }
                i += field.len();
            }
        }
        if i != length {
            return Err(malformed("trailing ID3 extended header bytes"));
        }
        Ok((length, 0))
    }
}

fn text(data: &[u8], version: u8) -> Result<String, OperationError> {
    let invalid = || malformed("invalid ID3 text encoding");
    let (&encoding, rest) = data.split_first().ok_or_else(invalid)?;
    let value = match encoding {
        0 => rest.iter().copied().map(char::from).collect::<String>(),
        3 if version == 4 => std::str::from_utf8(rest).map_err(|_| invalid())?.to_owned(),
        1 | 2 if encoding == 1 || version == 4 => {
            let (little, bytes) = if encoding == 2 {
                (false, rest)
            } else {
                match rest.get(..2) {
                    Some([0xff, 0xfe]) => (true, &rest[2..]),
                    Some([0xfe, 0xff]) => (false, &rest[2..]),
                    _ => return Err(invalid()),
                }
            };
            if bytes.len() % 2 != 0 {
                return Err(invalid());
            }
            let units = bytes.as_chunks::<2>().0.iter().map(|&chunk| {
                if little {
                    u16::from_le_bytes(chunk)
                } else {
                    u16::from_be_bytes(chunk)
                }
            });
            char::decode_utf16(units)
                .collect::<Result<String, _>>()
                .map_err(|_| invalid())?
        }
        _ => return Err(invalid()),
    };
    Ok(value.trim_end_matches('\0').to_owned())
}

fn tag_frames(
    logical: &Logical<'_>,
    start: usize,
    end: usize,
    version: u8,
    unsync: bool,
) -> Result<Vec<TagFrame>, OperationError> {
    let mut frames = Vec::new();
    let mut retained = 0;
    let mut p = start;
    let data = &logical.data[..end];
    while p < end && data[p] != 0 {
        if frames.len() == 256 {
            return Err(limit("ID3 exceeds 256 tag frames"));
        }
        let header = slice(data, p, 10)?;
        if !header[..4]
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            return Err(malformed("invalid ID3 frame identifier"));
        }
        let id = std::str::from_utf8(&header[..4]).expect("ASCII tag");
        let length = if version == 3 {
            be32(&header[4..])
        } else {
            syncsafe(&header[4..8])?
        };
        let flags = u16::from_be_bytes([header[8], header[9]]);
        let mask = if version == 3 { 0xe0e0 } else { 0x704f };
        if flags & !mask != 0 || length == 0 {
            return Err(malformed("invalid ID3 frame flags or length"));
        }
        let raw = slice(data, p + 10, length)?;
        let unsupported = if version == 3 {
            flags & 0xc0 != 0
        } else {
            flags & 0x0c != 0
        };
        let decode = id.starts_with('T') && !unsupported;
        let value = if version == 4 && (decode || flags & 1 != 0) {
            let frame = Logical::parse(raw, unsync || flags & 2 != 0);
            let mut source = frame.data.as_ref();
            if flags & 0x40 != 0 {
                source = slice(source, 1, source.len().saturating_sub(1))?;
            }
            if flags & 4 != 0 {
                source = slice(source, 1, source.len().saturating_sub(1))?;
            }
            if flags & 1 != 0 {
                let declared = syncsafe(slice(source, 0, 4)?)?;
                source = &source[4..];
                if !unsupported && declared != source.len() {
                    return Err(malformed("ID3 frame data-length indicator mismatch"));
                }
            }
            decode.then(|| text(source, version)).transpose()?
        } else if decode {
            let source = if flags & 0x20 != 0 { &raw[1..] } else { raw };
            Some(text(source, version)?)
        } else {
            None
        };
        if version == 4 && flags & 8 != 0 && flags & 1 == 0 {
            return Err(malformed(
                "compressed ID3v2.4 frame requires a data-length indicator",
            ));
        }
        retained += value.as_ref().map_or(0, String::len);
        if retained > MIB {
            return Err(limit("ID3 retained text exceeds 1 MiB"));
        }
        let next = p + 10 + length;
        frames.push(TagFrame {
            id: id.to_owned(),
            range: logical.tag_range(p, next),
            payload: logical.tag_range(p + 10, next),
            flags,
            text: value,
            unsupported_payload: unsupported || id == "APIC",
        });
        p = next;
    }
    if data[p..].iter().any(|&b| b != 0) {
        return Err(malformed("invalid ID3 tag padding"));
    }
    Ok(frames)
}

fn read_tag(source: &[u8]) -> Result<Tag, OperationError> {
    let header = slice(source, 0, 10)?;
    let version = header[3];
    if !matches!(version, 3 | 4) || header[4] == 255 {
        return Err(unsupported("only ID3v2.3 and ID3v2.4 are supported"));
    }
    let flags = header[5];
    if flags & (if version == 3 { 0x1f } else { 0x0f }) != 0 {
        return Err(malformed("reserved ID3 header flags"));
    }
    let length = syncsafe(&header[6..])?;
    if length > MIB {
        return Err(limit("ID3 tag exceeds 1 MiB"));
    }
    let raw = slice(source, 10, length)?;
    let unsynchronized = flags & 0x80 != 0;
    let logical = Logical::parse(raw, version == 3 && unsynchronized);
    let (start, padding) = if flags & 0x40 != 0 {
        extended(&logical.data, version)?
    } else {
        (0, 0)
    };
    let end = logical
        .data
        .len()
        .checked_sub(padding)
        .filter(|&p| p >= start)
        .ok_or_else(|| malformed("invalid ID3 tag padding size"))?;
    if logical.data[end..].iter().any(|&b| b != 0) {
        return Err(malformed("nonzero ID3v2.3 declared padding"));
    }
    let physical_end = 10 + length;
    let footer = if version == 4 && flags & 0x10 != 0 {
        let footer = slice(source, physical_end, 10)?;
        if &footer[..3] != b"3DI" || footer[3..] != header[3..] {
            return Err(malformed("invalid ID3v2.4 tag footer"));
        }
        Some(Extent::new(physical_end, 10))
    } else {
        None
    };
    Ok(Tag {
        version: if version == 3 { "2.3" } else { "2.4" },
        range: Extent::new(0, physical_end + footer.map_or(0, |_| 10)),
        extended_header: (start != 0).then(|| logical.tag_range(0, start)),
        footer,
        unsynchronized,
        frames: tag_frames(&logical, start, end, version, unsynchronized)?,
    })
}

fn audio_frame(data: &[u8], offset: usize) -> Result<AudioFrame, OperationError> {
    let b = slice(data, offset, 4)?;
    let invalid = || malformed("invalid MPEG Layer III frame header").offset(offset as u64);
    if b[0] != 255 || b[1] & 0xe0 != 0xe0 {
        return Err(invalid());
    }
    if b[1] & 6 != 2 {
        return Err(unsupported("only MPEG Layer III framing is supported").offset(offset as u64));
    }
    let (version, divisor) = match (b[1] >> 3) & 3 {
        3 => ("mpeg1", 1),
        2 => ("mpeg2", 2),
        0 => ("mpeg2_5", 4),
        _ => return Err(invalid()),
    };
    let index = (b[2] >> 4) as usize;
    let rate = ((b[2] >> 2) & 3) as usize;
    if index == 0 {
        return Err(unsupported("free-format MPEG audio is unsupported"));
    }
    if index == 15 || rate == 3 || b[3] & 3 == 2 {
        return Err(invalid());
    }
    let kbps = if divisor == 1 {
        [
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
        ][index]
    } else {
        [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160][index]
    };
    let hz = [44_100, 48_000, 32_000][rate] / divisor;
    let samples = if divisor == 1 { 1152 } else { 576 };
    let n = (u32::from(samples) * 125 * kbps / hz + u32::from((b[2] >> 1) & 1)) as usize;
    slice(data, offset, n)?;
    let mode = ["stereo", "joint_stereo", "dual_channel", "mono"][(b[3] >> 6) as usize];
    Ok(AudioFrame {
        range: Extent::new(offset, n),
        version,
        sample_rate: hz,
        bitrate_kbps: kbps,
        channel_mode: mode,
        samples_per_frame: samples,
        crc_present: b[1] & 1 == 0,
    })
}

/// Inspect a leading tag and/or contiguous MPEG Layer III audio frames.
pub fn inspect(data: &[u8]) -> Result<Report, OperationError> {
    check_source(data)?;
    let tag = data
        .starts_with(b"ID3")
        .then(|| read_tag(data))
        .transpose()?;
    let mut offset = tag.as_ref().map_or(0, |tag| tag.range.length as usize);
    let mut frames = Vec::new();
    let mut trailing = None;
    while offset < data.len() {
        if data.len() - offset == 128 && data[offset..].starts_with(b"TAG") {
            trailing = Some(Extent::new(offset, 128));
            break;
        }
        if frames.len() == 10_000 {
            return Err(limit("MPEG exceeds 10000 audio frames"));
        }
        let frame = audio_frame(data, offset)?;
        offset += frame.range.length as usize;
        frames.push(frame);
    }
    if tag.is_none() && frames.is_empty() {
        return Err(malformed("source has no ID3v2 tag or MPEG audio frame"));
    }
    bounded_report(
        Report {
            tag,
            frames,
            trailing_id3v1: trailing,
            payload_verified: false,
        },
        MAX_REPORT,
    )
}
