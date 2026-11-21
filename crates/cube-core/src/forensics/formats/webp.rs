//! Checked RIFF/WebP chunk extents and still/extended image dimensions.
use super::*;

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Chunk {
    pub fourcc: String,
    pub payload: Extent,
    pub padded_length: u64,
}
#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub width: u32,
    pub height: u32,
    pub declared_size: u32,
    pub logical_end: u64,
    pub trailing_bytes: u64,
    pub chunks: Vec<Chunk>,
    pub unsupported_features: Vec<&'static str>,
    pub payload_verified: bool,
}

fn le24(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])
}

fn dimensions(id: &[u8], data: &[u8]) -> Option<(u32, u32)> {
    match id {
        b"VP8 " if data.len() >= 10 && data[0] & 1 == 0 && data[3..6] == [0x9d, 1, 0x2a] => {
            let size = le24(data) >> 5;
            (size as usize <= data.len() - 10).then(|| {
                (
                    u32::from(le16(data, 6) & 0x3fff),
                    u32::from(le16(data, 8) & 0x3fff),
                )
            })
        }
        b"VP8L" if data.len() >= 5 && data[0] == 0x2f && data[4] >> 5 == 0 => {
            let info = le32(data, 1);
            Some(((info & 0x3fff) + 1, ((info >> 14) & 0x3fff) + 1))
        }
        b"VP8X" if data.len() == 10 && data[0] & 0xc1 == 0 && data[1..4] == [0; 3] => {
            let (w, h) = (le24(&data[4..]) + 1, le24(&data[7..]) + 1);
            (u64::from(w) * u64::from(h) <= u64::from(u32::MAX)).then_some((w, h))
        }
        _ => None,
    }
}

/// Inspect top-level chunks, dimensions and feature flags without expanding pixels.
pub fn inspect(bytes: &[u8]) -> Result<Report, OperationError> {
    check_source(bytes)?;
    let header = slice(bytes, 0, 12)?;
    if &header[..4] != b"RIFF" || &header[8..] != b"WEBP" {
        return Err(unsupported("expected RIFF/WEBP file framing"));
    }
    let declared = le32(header, 4);
    let end =
        usize::try_from(u64::from(declared) + 8).map_err(|_| malformed("WebP size overflow"))?;
    if end < 12 || end > bytes.len() || end % 2 != 0 {
        return Err(malformed("invalid RIFF/WebP declared size"));
    }
    let mut offset = 12;
    let mut chunks = Vec::new();
    let mut canvas = None;
    let mut picture = None;
    let mut unsupported_features = Vec::new();
    let mut features = 0;
    let mut actual = 0;
    let mut animation_frame = false;
    while offset < end {
        if chunks.len() == 10_000 {
            return Err(limit("WebP exceeds 10000 chunks"));
        }
        let chunk = slice(&bytes[..end], offset, 8)?;
        let id = &chunk[..4];
        if !id.iter().all(|&b| (32..=126).contains(&b)) {
            return Err(malformed("invalid WebP chunk FourCC"));
        }
        let length = le32(chunk, 4) as usize;
        let padded = length
            .checked_add(length % 2)
            .ok_or_else(|| malformed("WebP chunk padding overflows"))?;
        let payload = slice(&bytes[..end], offset + 8, padded)?;
        if length != padded && payload[length] != 0 {
            return Err(malformed("nonzero WebP chunk pad byte"));
        }
        let data = &payload[..length];
        match id {
            b"VP8X" if chunks.is_empty() => {
                canvas = dimensions(id, data);
                if canvas.is_none() {
                    return Err(malformed("invalid VP8X canvas"));
                }
                features = data[0];
            }
            b"VP8 " | b"VP8L" if picture.is_none() && features & 2 == 0 => {
                picture = dimensions(id, data);
                if picture.is_none_or(|(w, h)| w == 0 || h == 0) {
                    return Err(malformed("invalid WebP image dimensions"));
                }
                if id == b"VP8L" && data[4] & 0x10 != 0 {
                    actual |= 0x10;
                }
            }
            b"VP8X" | b"VP8 " | b"VP8L" => {
                return Err(malformed("misplaced or repeated WebP image header"));
            }
            b"ICCP" => actual |= 0x20,
            b"ALPH" => actual |= 0x10,
            b"EXIF" => actual |= 8,
            b"XMP " => actual |= 4,
            b"ANIM" if length == 6 && actual & 2 == 0 => actual |= 2,
            b"ANMF" if length >= 16 && actual & 2 != 0 => animation_frame = true,
            b"ANIM" | b"ANMF" => return Err(malformed("invalid WebP animation chunk")),
            _ if !unsupported_features.contains(&"unknown_chunk") => {
                unsupported_features.push("unknown_chunk")
            }
            _ => {}
        }
        chunks.push(Chunk {
            fourcc: std::str::from_utf8(id).expect("ASCII FourCC").to_owned(),
            payload: Extent::new(offset + 8, length),
            padded_length: padded as u64,
        });
        offset += 8 + padded;
    }
    if features & 2 != 0 {
        unsupported_features.push("animation_frame_payloads");
    }
    if let Some(canvas) = canvas {
        if (features & !0x10 != actual & !0x10)
            || (features & 2 != 0 && !animation_frame)
            || (features & 2 == 0 && picture != Some(canvas))
        {
            return Err(malformed(
                "WebP feature flags or canvas contradict the chunks",
            ));
        }
    } else if actual & !0x10 != 0 || chunks.len() != 1 {
        return Err(malformed("simple WebP requires one image chunk"));
    }
    let (width, height) = canvas
        .or(picture)
        .ok_or_else(|| malformed("WebP has no image dimensions"))?;
    bounded_report(
        Report {
            width,
            height,
            declared_size: declared,
            logical_end: end as u64,
            trailing_bytes: (bytes.len() - end) as u64,
            chunks,
            unsupported_features,
            payload_verified: false,
        },
        MIB,
    )
}
