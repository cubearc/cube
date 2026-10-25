use super::*;
struct Element {
    id: u64,
    body: u64,
    end: u64,
    unknown: bool,
}
fn vint<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    at: u64,
    end: u64,
    id: bool,
) -> Result<(u64, usize, bool), OperationError> {
    if at >= end {
        return Err(invalid("truncated EBML variable integer"));
    }
    let first = r.at(at, 1)?[0];
    if first == 0 {
        return Err(invalid("invalid EBML variable integer marker"));
    }
    let width = first.leading_zeros() as usize + 1;
    if width > if id { 4 } else { 8 } {
        return Err(unsupported("unsupported EBML integer width"));
    }
    if end - at < width as u64 {
        return Err(invalid("truncated EBML variable integer"));
    }
    let bytes = r.at(at, width)?;
    let mut n = if id {
        u64::from(bytes[0])
    } else {
        u64::from(bytes[0] & ((1u8 << (8 - width)) - 1))
    };
    for b in &bytes[1..] {
        n = (n << 8) | u64::from(*b);
    }
    if id {
        let payload = n & ((1u64 << (width * 7)) - 1);
        if payload == 0 || payload == (1u64 << (width * 7)) - 1 {
            return Err(invalid("reserved EBML element identifier"));
        }
    }
    let unknown = !id && n == (1u64 << (width * 7)) - 1;
    Ok((n, width, unknown))
}
fn element<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    at: u64,
    end: u64,
    unknown_ok: bool,
) -> Result<Element, OperationError> {
    r.record()?;
    let (id, n, _) = vint(r, at, end, true)?;
    let (size, m, unknown) = vint(r, at + n as u64, end, false)?;
    let body = at + (n + m) as u64;
    let finish = if unknown {
        if !unknown_ok {
            return Err(unsupported("unknown-sized EBML children are unsupported"));
        }
        end
    } else {
        body.checked_add(size)
            .ok_or_else(|| invalid("EBML element extent overflows"))?
    };
    if finish > end {
        return Err(invalid("EBML element exceeds parent/source"));
    }
    Ok(Element {
        id,
        body,
        end: finish,
        unknown,
    })
}
fn uint<R: Read + Seek>(r: &mut Reader<'_, R>, e: &Element) -> Result<u64, OperationError> {
    let n = e.end - e.body;
    if !(1..=8).contains(&n) {
        return Err(invalid("EBML unsigned field requires 1..8 bytes"));
    }
    Ok(r.at(e.body, n as usize)?
        .iter()
        .fold(0, |n, b| (n << 8) | u64::from(*b)))
}
fn float<R: Read + Seek>(r: &mut Reader<'_, R>, e: &Element) -> Result<f64, OperationError> {
    let n = e.end - e.body;
    let bytes = r.at(
        e.body,
        usize::try_from(n).map_err(|_| limit("float length exceeds platform"))?,
    )?;
    let value = match bytes.len() {
        4 => f64::from(f32::from_be_bytes(bytes.try_into().expect("float32"))),
        8 => f64::from_be_bytes(bytes.try_into().expect("float64")),
        _ => return Err(invalid("EBML float requires 4 or 8 bytes")),
    };
    if !value.is_finite() || value < 0.0 {
        return Err(invalid("EBML float is negative or nonfinite"));
    }
    Ok(value)
}
fn string<R: Read + Seek>(r: &mut Reader<'_, R>, e: &Element) -> Result<String, OperationError> {
    let n = e.end - e.body;
    if n > 128 {
        return Err(limit("EBML identifier exceeds 128 bytes"));
    }
    let data = r.at(e.body, n as usize)?;
    let s = std::str::from_utf8(&data)
        .map_err(|_| invalid("EBML identifier is not UTF-8"))?
        .trim_end_matches('\0');
    if s.is_empty() || !s.bytes().all(|b| (32..=126).contains(&b)) {
        return Err(invalid("EBML identifier must be nonempty printable ASCII"));
    }
    Ok(s.into())
}
fn children<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    parent: &Element,
    mut f: impl FnMut(&mut Reader<'_, R>, Element) -> Result<(), OperationError>,
) -> Result<(), OperationError> {
    let mut at = parent.body;
    let mut seen = std::collections::BTreeSet::new();
    while at < parent.end {
        let e = element(r, at, parent.end, false)?;
        at = e.end;
        if matches!(
            e.id,
            0x4282
                | 0x42f7
                | 0x4285
                | 0x42f2
                | 0x42f3
                | 0xd7
                | 0x73c5
                | 0x83
                | 0x86
                | 0x23314f
                | 0x6d80
                | 0xe0
                | 0xe1
                | 0xb0
                | 0xba
                | 0x9f
                | 0xb5
                | 0x2ad7b1
                | 0x4489
        ) && !seen.insert(e.id)
        {
            return Err(invalid("duplicate singleton WebM metadata field"));
        }
        f(r, e)?;
    }
    Ok(())
}
fn track<R: Read + Seek>(r: &mut Reader<'_, R>, entry: &Element) -> Result<Track, OperationError> {
    let mut t = Track::default();
    let mut uid = None;
    children(r, entry, |r, e| {
        match e.id {
            0xd7 => t.id = uint(r, &e)?,
            0x73c5 => uid = Some(uint(r, &e)?),
            0x83 => {
                t.kind = match uint(r, &e)? {
                    1 => "video",
                    2 => "audio",
                    _ => "other",
                }
            }
            0x86 => t.codec_id = Some(string(r, &e)?),
            0x23314f => {
                if float(r, &e)? != 1.0 {
                    return Err(unsupported(
                        "non-default WebM track timestamp scale is unsupported",
                    ));
                }
            }
            0x6d80 => {
                return Err(unsupported(
                    "WebM content encodings/encryption are unsupported",
                ));
            }
            0xe0 => children(r, &e, |r, e| {
                match e.id {
                    0xb0 | 0xba => {
                        let n = uint(r, &e)?;
                        let n =
                            u32::try_from(n).map_err(|_| invalid("WebM dimensions exceed u32"))?;
                        if n == 0 {
                            return Err(invalid("WebM dimensions must be positive"));
                        }
                        if e.id == 0xb0 {
                            t.width = Some(n)
                        } else {
                            t.height = Some(n)
                        }
                    }
                    _ => r.skipped += 1,
                }
                Ok(())
            })?,
            0xe1 => {
                t.channels = Some(1);
                t.sample_rate = Some(8000.0);
                children(r, &e, |r, e| {
                    match e.id {
                        0x9f => {
                            let n = uint(r, &e)?;
                            if n == 0 {
                                return Err(invalid("WebM channels must be positive"));
                            }
                            t.channels = Some(n)
                        }
                        0xb5 => {
                            let value = float(r, &e)?;
                            if value == 0.0 {
                                return Err(invalid("WebM sample rate must be positive"));
                            }
                            t.sample_rate = Some(value)
                        }
                        _ => r.skipped += 1,
                    }
                    Ok(())
                })?
            }
            _ => r.skipped += 1,
        }
        Ok(())
    })?;
    if uid.is_none_or(|n| n == 0) || t.kind == "unknown" || t.codec_id.is_none() {
        return Err(invalid("WebM track metadata is incomplete"));
    }
    if t.kind == "video" && (t.width.is_none() || t.height.is_none()) {
        return Err(invalid("WebM video dimensions are absent"));
    }
    Ok(t)
}
pub(super) fn inspect<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    info: &mut Info,
) -> Result<(), OperationError> {
    let header = element(r, 0, r.size, false)?;
    if header.id != 0x1a45dfa3 {
        return Err(invalid("WebM EBML header is absent"));
    }
    let mut doctype = None;
    children(r, &header, |r, e| {
        match e.id {
            0x4282 => doctype = Some(string(r, &e)?),
            0x42f7 => {
                if uint(r, &e)? != 1 {
                    return Err(unsupported("unsupported EBML read version"));
                }
            }
            0x4285 => {
                if !(1..=2).contains(&uint(r, &e)?) {
                    return Err(unsupported("unsupported WebM document read version"));
                }
            }
            0x42f2 => {
                if uint(r, &e)? != 4 {
                    return Err(unsupported("unsupported EBML ID width"));
                }
            }
            0x42f3 => {
                if !(1..=8).contains(&uint(r, &e)?) {
                    return Err(unsupported("unsupported EBML size width"));
                }
            }
            _ => r.skipped += 1,
        }
        Ok(())
    })?;
    if doctype.as_deref() != Some("webm") {
        return Err(unsupported(
            "only WebM DocType is supported; general Matroska is unsupported",
        ));
    }
    let segment = element(r, header.end, r.size, true)?;
    if segment.id != 0x18538067 || segment.end != r.size {
        return Err(unsupported("one complete WebM Segment is required"));
    }
    let _unknown_segment = segment.unknown;
    let mut scale = 1000000;
    let mut seen_info = false;
    let mut seen_tracks = false;
    let mut at = segment.body;
    while at < segment.end {
        let e = element(r, at, segment.end, false)?;
        at = e.end;
        match e.id {
            0x1549a966 => {
                if seen_info {
                    return Err(invalid("duplicate WebM Info"));
                }
                seen_info = true;
                children(r, &e, |r, e| {
                    match e.id {
                        0x2ad7b1 => {
                            scale = uint(r, &e)?;
                            if scale == 0 {
                                return Err(invalid("WebM timestamp scale must be positive"));
                            }
                        }
                        0x4489 => info.duration_ticks = Some(Duration::Float(float(r, &e)?)),
                        _ => r.skipped += 1,
                    }
                    Ok(())
                })?
            }
            0x1654ae6b => {
                if seen_tracks {
                    return Err(invalid("duplicate WebM Tracks"));
                }
                seen_tracks = true;
                let mut at = e.body;
                while at < e.end {
                    let entry = element(r, at, e.end, false)?;
                    at = entry.end;
                    if entry.id == 0xae {
                        if info.tracks.len() == MAX_TRACKS {
                            return Err(limit("container exceeds 32 tracks"));
                        }
                        info.tracks.push(track(r, &entry)?);
                    } else {
                        r.skipped += 1
                    }
                }
            }
            _ => r.skipped += 1,
        }
    }
    if !seen_info || !seen_tracks {
        return Err(invalid("WebM Info/Tracks metadata is absent"));
    }
    info.container = "webm";
    info.time_base = Some(TimeBase {
        numerator: scale,
        denominator: 1000000000,
    });
    for t in &mut info.tracks {
        t.time_base = info.time_base;
    }
    Ok(())
}
