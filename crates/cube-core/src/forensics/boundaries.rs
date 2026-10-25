//! Format-aware framing boundaries. This does not decode compressed image data.
use super::{MAX_SCAN_BYTES, limit, malformed, png::Boundary, size};
use crate::errors::{ErrorCode, OperationError};
use std::io::{BufReader, Read, Seek, SeekFrom};

struct Framing<R> {
    reader: BufReader<R>,
    pos: u64,
    total: u64,
    records: usize,
}
impl<R: Read + Seek> Framing<R> {
    fn advance(&mut self, n: usize) -> Result<u64, OperationError> {
        let end = self
            .pos
            .checked_add(n as u64)
            .ok_or_else(|| malformed("framing offset overflow"))?;
        if end > self.total {
            return Err(malformed("truncated image framing").offset(self.pos));
        }
        if end > MAX_SCAN_BYTES {
            return Err(limit("image boundary exceeds 64 MiB").offset(self.pos));
        }
        self.pos = end;
        Ok(end)
    }
    fn bytes(&mut self, n: usize) -> Result<Vec<u8>, OperationError> {
        self.advance(n)?;
        let mut bytes = vec![0; n];
        self.reader
            .read_exact(&mut bytes)
            .map_err(|e| malformed("truncated image framing").caused_by(e))?;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8, OperationError> {
        self.advance(1)?;
        let mut b = [0];
        self.reader
            .read_exact(&mut b)
            .map_err(|e| malformed("truncated image framing").caused_by(e))?;
        Ok(b[0])
    }
    fn skip(&mut self, n: usize) -> Result<(), OperationError> {
        let end = self.advance(n)?;
        self.reader.seek(SeekFrom::Start(end))?;
        Ok(())
    }
    fn record(&mut self) -> Result<(), OperationError> {
        self.records += 1;
        if self.records > 10000 {
            return Err(limit("image framing exceeds 10000 records"));
        }
        Ok(())
    }
    fn finish(&self, format: &'static str, evidence: &'static str) -> Boundary {
        Boundary {
            format,
            logical_end: self.pos,
            trailing_offset: self.pos,
            trailing_bytes: self.total - self.pos,
            validated_chunks: self.records,
            evidence,
        }
    }
    fn subblocks(&mut self) -> Result<(), OperationError> {
        loop {
            self.record()?;
            let n = self.byte()? as usize;
            if n == 0 {
                return Ok(());
            }
            self.skip(n)?;
        }
    }
}
fn unsupported(message: &str) -> OperationError {
    OperationError::new(ErrorCode::UnsupportedFormat, message)
}

/// Detect the logical end of strict PNG framing, baseline sequential Huffman
/// JPEG, or GIF raster framing; ZIP reuses the separate validator. At most 64 MiB and 10000 framing records are
/// traversed. Compressed pixels/entropy codes are not validated. Unsupported
/// JPEG processes and GIF plain-text rendering are explicit errors.
/// PNG returns the existing report unchanged; `validated_chunks` counts framing
/// records (segments/blocks) for the additional formats.
pub fn inspect(reader: &mut (impl Read + Seek)) -> Result<Boundary, OperationError> {
    let result = (|| {
        let total = size(reader)?;
        reader.seek(SeekFrom::Start(0))?;
        let mut header = [0; 8];
        let n = total.min(8) as usize;
        reader.read_exact(&mut header[..n])?;
        reader.seek(SeekFrom::Start(0))?;
        if n == 8 && &header == b"\x89PNG\r\n\x1a\n" {
            return super::png::appended(reader);
        }
        if n >= 4 && (&header[..4] == b"PK\x03\x04" || &header[..4] == b"PK\x05\x06") {
            let archive = super::zip::validate(reader, 0)?;
            return Ok(Boundary {
                format: "zip",
                logical_end: archive.logical_end,
                trailing_offset: archive.logical_end,
                trailing_bytes: archive.trailing_bytes,
                validated_chunks: archive.entries.len() * 2 + 1,
                evidence: archive.evidence,
            });
        }
        let mut f = Framing {
            reader: BufReader::new(reader),
            pos: 0,
            total,
            records: 0,
        };
        if n >= 2 && header[..2] == [255, 216] {
            jpeg(&mut f)
        } else if n >= 6 && [&b"GIF87a"[..], &b"GIF89a"[..]].contains(&&header[..6]) {
            gif(&mut f)
        } else {
            Err(unsupported(
                "boundary inspection supports PNG, baseline JPEG, GIF and classic ZIP framing",
            ))
        }
    })();
    result.map_err(|e: OperationError| e.operation("file.appended.detect"))
}

fn gif(f: &mut Framing<impl Read + Seek>) -> Result<Boundary, OperationError> {
    let header = f.bytes(13)?;
    let word = |s: &[u8]| u16::from_le_bytes([s[0], s[1]]) as u32;
    let (width, height) = (word(&header[6..]), word(&header[8..]));
    if width == 0 || height == 0 {
        return Err(malformed("GIF dimensions must be positive"));
    }
    let global = if header[10] & 0x80 != 0 {
        1usize << ((header[10] & 7) + 1)
    } else {
        0
    };
    if global > 0 {
        if header[11] as usize >= global {
            return Err(malformed("GIF background index exceeds palette"));
        }
        f.skip(global * 3)?;
    }
    let mut images = 0;
    let mut control = None;
    loop {
        f.record()?;
        match f.byte()? {
            0x3b => {
                if images == 0 || control.is_some() {
                    return Err(malformed("GIF has no raster or dangling graphic control"));
                }
                return Ok(f.finish("gif","GIF header, palettes, raster/extension block lengths and trailer; LZW pixels are not decoded"));
            }
            0x2c => {
                let image = f.bytes(9)?;
                let (x, y, w, h) = (
                    word(&image),
                    word(&image[2..]),
                    word(&image[4..]),
                    word(&image[6..]),
                );
                if w == 0 || h == 0 || x + w > width || y + h > height || image[8] & 0x18 != 0 {
                    return Err(malformed("invalid GIF raster dimensions or reserved flags"));
                }
                let palette = if image[8] & 0x80 != 0 {
                    let count = 1usize << ((image[8] & 7) + 1);
                    f.skip(count * 3)?;
                    count
                } else {
                    global
                };
                if palette == 0
                    || control
                        .take()
                        .flatten()
                        .is_some_and(|index| index as usize >= palette)
                {
                    return Err(malformed("GIF raster has no valid palette/index"));
                }
                if !(2..=8).contains(&f.byte()?) {
                    return Err(malformed("invalid GIF LZW minimum code size"));
                }
                // A raster must have compressed data, not just an empty terminator.
                let n = f.byte()? as usize;
                if n == 0 {
                    return Err(malformed("GIF raster has no compressed data"));
                }
                f.skip(n)?;
                f.subblocks()?;
                images += 1;
            }
            0x21 => match f.byte()? {
                0xf9 => {
                    if control.is_some() || f.byte()? != 4 {
                        return Err(malformed("invalid GIF graphic control block"));
                    }
                    let data = f.bytes(4)?;
                    if data[0] & 0xe0 != 0 || (data[0] >> 2) & 7 > 3 || f.byte()? != 0 {
                        return Err(malformed("invalid GIF graphic control flags/terminator"));
                    }
                    control = Some((data[0] & 1 != 0).then_some(data[3]));
                }
                0xff => {
                    if f.byte()? != 11 {
                        return Err(malformed("invalid GIF application header length"));
                    }
                    f.skip(11)?;
                    f.subblocks()?;
                }
                0xfe => f.subblocks()?,
                0x01 => {
                    return Err(unsupported(
                        "GIF plain-text rendering boundaries are not supported",
                    ));
                }
                _ => return Err(unsupported("unknown GIF extension label")),
            },
            _ => return Err(malformed("invalid GIF block introducer")),
        }
    }
}

fn jpeg(f: &mut Framing<impl Read + Seek>) -> Result<Boundary, OperationError> {
    f.skip(2)?;
    let mut components = Vec::new();
    let mut scanned = Vec::new();
    let mut quant = [false; 4];
    let mut huffman = [[false; 4]; 2];
    let mut restart = false;
    loop {
        f.record()?;
        if f.byte()? != 0xff {
            return Err(malformed("expected JPEG marker prefix"));
        }
        let mut marker = f.byte()?;
        while marker == 0xff {
            marker = f.byte()?;
        }
        if marker == 0xd9 {
            if components.is_empty() || scanned.len() != components.len() {
                return Err(malformed(
                    "JPEG ends before all frame components are scanned",
                ));
            }
            return Ok(f.finish("jpeg","Baseline JPEG marker/table/frame/scan framing through EOI; entropy codes and pixels are not decoded"));
        }
        if matches!(marker, 0 | 0xd0..=0xd8 | 0x01) {
            return Err(malformed("unexpected standalone JPEG marker"));
        }
        if !matches!(
            marker,
            0xc0 | 0xc4 | 0xdb | 0xdd | 0xda | 0xe0..=0xef | 0xfe
        ) {
            return Err(unsupported(
                "only baseline sequential Huffman JPEG framing is supported",
            ));
        }
        let length = f.bytes(2)?;
        let length = u16::from_be_bytes([length[0], length[1]]) as usize;
        if length < 2 {
            return Err(malformed("invalid JPEG segment length"));
        }
        let data = f.bytes(length - 2)?;
        match marker {
            0xc0 => {
                if !components.is_empty()
                    || data.len() < 6
                    || data[0] != 8
                    || data[1..3] == [0, 0]
                    || data[3..5] == [0, 0]
                {
                    return Err(malformed("invalid baseline JPEG frame"));
                }
                let count = data[5] as usize;
                if !matches!(count, 1 | 3) {
                    return Err(unsupported("JPEG framing supports one or three components"));
                }
                if data.len() != 6 + 3 * count {
                    return Err(malformed("invalid JPEG component table length"));
                }
                let mut blocks = 0;
                for c in data[6..].as_chunks::<3>().0 {
                    let (h, v) = (c[1] >> 4, c[1] & 15);
                    if !(1..=4).contains(&h)
                        || !(1..=4).contains(&v)
                        || c[2] > 3
                        || components.iter().any(|&(id, _)| id == c[0])
                    {
                        return Err(malformed("invalid JPEG frame component"));
                    }
                    blocks += h as usize * v as usize;
                    components.push((c[0], c[2]));
                }
                if blocks > 10 {
                    return Err(malformed("JPEG sampling exceeds baseline MCU limit"));
                }
            }
            0xdb => {
                let mut p = 0;
                if data.is_empty() {
                    return Err(malformed("empty JPEG quantization segment"));
                }
                while p < data.len() {
                    let id = data[p];
                    p += 1;
                    if id >> 4 != 0 {
                        return Err(unsupported(
                            "baseline JPEG requires 8-bit quantization tables",
                        ));
                    }
                    if id > 3 || data.len() - p < 64 || data[p..p + 64].contains(&0) {
                        return Err(malformed("invalid JPEG quantization table"));
                    }
                    quant[id as usize] = true;
                    p += 64;
                }
            }
            0xc4 => {
                let mut p = 0;
                if data.is_empty() {
                    return Err(malformed("empty JPEG Huffman segment"));
                }
                while p < data.len() {
                    if data.len() - p < 17 {
                        return Err(malformed("truncated JPEG Huffman table"));
                    }
                    let id = data[p];
                    p += 1;
                    if id >> 4 > 1 || id & 15 > 1 {
                        return Err(malformed("invalid JPEG Huffman table identifier"));
                    }
                    let counts = &data[p..p + 16];
                    p += 16;
                    let n: usize = counts.iter().map(|&b| b as usize).sum();
                    let mut available = 1i32;
                    for &count in counts {
                        available = available * 2 - count as i32;
                        if available < 0 {
                            return Err(malformed("oversubscribed JPEG Huffman table"));
                        }
                    }
                    if n == 0 || n > 256 || data.len() - p < n {
                        return Err(malformed("invalid JPEG Huffman symbol count"));
                    }
                    if available == 0
                        || data[p..p + n].iter().any(|&symbol| {
                            if id >> 4 == 0 {
                                symbol > 11
                            } else {
                                symbol & 15 > 10
                                    || (symbol & 15 == 0 && symbol != 0 && symbol != 0xf0)
                            }
                        })
                    {
                        return Err(malformed("invalid baseline JPEG Huffman symbols/codes"));
                    }
                    huffman[(id >> 4) as usize][(id & 15) as usize] = true;
                    p += n;
                }
            }
            0xdd => {
                if data.len() != 2 {
                    return Err(malformed("invalid JPEG restart interval"));
                }
                restart = data != [0, 0];
            }
            0xda => {
                if data.is_empty() {
                    return Err(malformed("empty JPEG scan header"));
                }
                let count = data[0] as usize;
                if count == 0
                    || count > components.len()
                    || data.len() != 4 + 2 * count
                    || data[data.len() - 3..] != [0, 63, 0]
                {
                    return Err(malformed("invalid baseline JPEG scan header"));
                }
                for c in data[1..1 + 2 * count].as_chunks::<2>().0 {
                    let Some(&(_, q)) = components.iter().find(|&&(id, _)| id == c[0]) else {
                        return Err(malformed("JPEG scan references unknown component"));
                    };
                    if scanned.contains(&c[0])
                        || c[1] >> 4 > 1
                        || c[1] & 15 > 1
                        || !quant[q as usize]
                        || !huffman[0][(c[1] >> 4) as usize]
                        || !huffman[1][(c[1] & 15) as usize]
                    {
                        return Err(malformed(
                            "JPEG scan has duplicate components or missing tables",
                        ));
                    }
                    scanned.push(c[0]);
                }
                let mut entropy = 0;
                let mut next_restart = 0;
                loop {
                    if f.byte()? != 0xff {
                        entropy += 1;
                        continue;
                    }
                    let marker_pos = f.pos - 1;
                    let mut next = f.byte()?;
                    while next == 0xff {
                        next = f.byte()?;
                    }
                    if next == 0 {
                        entropy += 1;
                        continue;
                    }
                    if (0xd0..=0xd7).contains(&next) {
                        f.record()?;
                        if !restart || next != 0xd0 + next_restart {
                            return Err(malformed("invalid JPEG restart sequence"));
                        }
                        next_restart = (next_restart + 1) % 8;
                        continue;
                    }
                    if entropy == 0 {
                        return Err(malformed("JPEG scan has no entropy bytes"));
                    }
                    f.reader.seek(SeekFrom::Start(marker_pos))?;
                    f.pos = marker_pos;
                    break;
                }
            }
            _ => (),
        }
    }
}

/// Existing ID and result shape retained; the legacy semantic type name is compatible.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
    id: "file.appended.detect",
    category: "file",
    description: "Find trailing bytes after supported PNG, baseline JPEG, GIF or classic ZIP framing; no pixel/entropy decoding.",
    input_type: "byte_reader",
    output_type: "file.png_boundary",
    command: "file appended",
    example: "cube file appended sample.png",
    parameters: &[],
    deterministic: true,
    reversible: None,
}];
