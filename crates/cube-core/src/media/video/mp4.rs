use super::*;
fn u32be(data: &[u8]) -> u32 {
    u32::from_be_bytes(data.try_into().expect("four bytes"))
}
fn u64be(data: &[u8]) -> u64 {
    u64::from_be_bytes(data.try_into().expect("eight bytes"))
}
struct Atom {
    kind: [u8; 4],
    body: u64,
    end: u64,
}
fn atom<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    start: u64,
    end: u64,
    top: bool,
) -> Result<Atom, OperationError> {
    r.record()?;
    if end - start < 8 {
        return Err(invalid("truncated MP4 atom header"));
    }
    let bytes = r.at(start, 8)?;
    let mut length = u64::from(u32be(&bytes[..4]));
    let mut header = 8;
    if length == 1 {
        if end - start < 16 {
            return Err(invalid("truncated extended MP4 atom"));
        }
        length = u64be(&r.at(start + 8, 8)?);
        header = 16;
    } else if length == 0 {
        if !top {
            return Err(unsupported("zero-sized nested MP4 atoms are unsupported"));
        }
        length = end - start;
    }
    if length < header {
        return Err(invalid("MP4 atom is smaller than its header"));
    }
    let finish = start
        .checked_add(length)
        .ok_or_else(|| invalid("MP4 atom extent overflows"))?;
    if finish > end {
        return Err(invalid("MP4 atom extends beyond parent"));
    }
    Ok(Atom {
        kind: bytes[4..8].try_into().expect("four bytes"),
        body: start + header,
        end: finish,
    })
}
fn prefix<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
    n: usize,
) -> Result<Vec<u8>, OperationError> {
    if a.end - a.body < n as u64 {
        return Err(invalid("truncated MP4 metadata field"));
    }
    r.at(a.body, n)
}
fn timing<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
) -> Result<(TimeBase, Option<Duration>), OperationError> {
    let first = prefix(r, a, 4)?;
    let minimum = match (&a.kind, first[0]) {
        (b"mvhd", 0) => 100,
        (b"mvhd", 1) => 112,
        (b"mdhd", 0) => 24,
        (b"mdhd", 1) => 36,
        _ => 0,
    };
    if a.end - a.body < minimum {
        return Err(invalid("truncated MP4 time header"));
    }
    let (scale, duration) = match first[0] {
        0 => {
            let b = prefix(r, a, 20)?;
            let n = u32be(&b[16..20]);
            (
                u32be(&b[12..16]),
                (n != u32::MAX).then_some(Duration::Integer(u64::from(n))),
            )
        }
        1 => {
            let b = prefix(r, a, 32)?;
            let n = u64be(&b[24..32]);
            (
                u32be(&b[20..24]),
                (n != u64::MAX).then_some(Duration::Integer(n)),
            )
        }
        _ => return Err(unsupported("unsupported MP4 time header version")),
    };
    if scale == 0 {
        return Err(invalid("MP4 time scale must be positive"));
    }
    Ok((
        TimeBase {
            numerator: 1,
            denominator: u64::from(scale),
        },
        duration,
    ))
}
fn sample_description<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
    t: &mut Track,
) -> Result<(), OperationError> {
    if t.kind == "unknown" {
        return Err(unsupported("MP4 handler must precede sample descriptions"));
    }
    let b = prefix(r, a, 8)?;
    if b[0] != 0 {
        return Err(unsupported("unsupported MP4 sample description version"));
    }
    if u32be(&b[4..8]) != 1 {
        return Err(unsupported(
            "exactly one MP4 sample description per track is supported",
        ));
    }
    let entry = atom(r, a.body + 8, a.end, false)?;
    if entry.end != a.end {
        return Err(invalid("unexpected trailing MP4 sample descriptions"));
    }
    if [*b"encv", *b"enca", *b"drms", *b"drmi"].contains(&entry.kind) {
        return Err(unsupported("encrypted MP4 sample entries are unsupported"));
    }
    t.codec_id = Some(fourcc(&entry.kind));
    // Versioned codec payload/configuration is deliberately not parsed or decoded.
    match t.kind {
        "video" => {
            let b = prefix(r, &entry, 78)?;
            let w = u32::from(u16::from_be_bytes(b[24..26].try_into().expect("width")));
            let h = u32::from(u16::from_be_bytes(b[26..28].try_into().expect("height")));
            if w == 0 || h == 0 {
                return Err(invalid("MP4 coded dimensions must be positive"));
            }
            t.width = Some(w);
            t.height = Some(h);
        }
        "audio" => {
            let b = prefix(r, &entry, 28)?;
            if b[8..10] != [0, 0] {
                return Err(unsupported(
                    "versioned QuickTime audio sample entries are unsupported",
                ));
            }
            t.channels = Some(u64::from(u16::from_be_bytes(
                b[16..18].try_into().expect("channels"),
            )));
            t.sample_rate = Some(f64::from(u32be(&b[24..28])) / 65536.0);
        }
        _ => {
            prefix(r, &entry, 8)?;
        }
    }
    Ok(())
}
fn walk<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    info: &mut Info,
    start: u64,
    end: u64,
    parent: [u8; 4],
    depth: usize,
    current: Option<usize>,
) -> Result<(), OperationError> {
    if depth > 8 {
        return Err(limit("MP4 metadata exceeds depth 8"));
    }
    let mut at = start;
    let mut seen = std::collections::BTreeSet::new();
    while at < end {
        let a = atom(r, at, end, depth == 0)?;
        at = a.end;
        if [*b"moof", *b"mvex", *b"senc", *b"sinf"].contains(&a.kind) {
            return Err(unsupported(
                "fragmented/encrypted MP4 metadata profiles are unsupported",
            ));
        }
        let relevant = matches!(
            (&parent, &a.kind),
            (b"root", b"ftyp" | b"moov")
                | (b"moov", b"mvhd")
                | (b"trak", b"tkhd" | b"mdia")
                | (b"mdia", b"mdhd" | b"hdlr" | b"minf")
                | (b"minf", b"stbl")
                | (b"stbl", b"stsd")
        );
        if relevant && !seen.insert(a.kind) {
            return Err(invalid("duplicate MP4 metadata field/container"));
        }
        match (&parent, &a.kind) {
            (b"root", b"ftyp") => {
                let b = prefix(r, &a, 8)?;
                info.major_brand = Some(fourcc(&b[..4]));
            }
            (b"root", b"moov") | (b"trak", b"mdia") | (b"mdia", b"minf") | (b"minf", b"stbl") => {
                walk(r, info, a.body, a.end, a.kind, depth + 1, current)?
            }
            (b"moov", b"trak") => {
                if info.tracks.len() == MAX_TRACKS {
                    return Err(limit("container exceeds 32 tracks"));
                }
                let index = info.tracks.len();
                info.tracks.push(Track::default());
                walk(r, info, a.body, a.end, a.kind, depth + 1, Some(index))?;
            }
            (b"moov", b"mvhd") => {
                let (base, duration) = timing(r, &a)?;
                info.time_base = Some(base);
                info.duration_ticks = duration;
            }
            (b"trak", b"tkhd") => {
                let t =
                    &mut info.tracks[current.ok_or_else(|| invalid("track context missing"))?];
                let b = prefix(r, &a, 4)?;
                let b = prefix(
                    r,
                    &a,
                    match b[0] {
                        0 => 84,
                        1 => 96,
                        _ => return Err(unsupported("unsupported MP4 track header version")),
                    },
                )?;
                let id_offset = if b[0] == 0 { 12 } else { 20 };
                t.id = u64::from(u32be(&b[id_offset..id_offset + 4]));
                t.display_dimensions_fixed_16_16 = Some([
                    u32be(&b[b.len() - 8..b.len() - 4]),
                    u32be(&b[b.len() - 4..]),
                ]);
            }
            (b"mdia", b"mdhd") => {
                let t =
                    &mut info.tracks[current.ok_or_else(|| invalid("track context missing"))?];
                let (base, duration) = timing(r, &a)?;
                t.time_base = Some(base);
                t.duration_ticks = duration;
            }
            (b"mdia", b"hdlr") => {
                let b = prefix(r, &a, 12)?;
                if b[0] != 0 {
                    return Err(unsupported("unsupported MP4 handler version"));
                }
                info.tracks[current.ok_or_else(|| invalid("track context missing"))?].kind =
                    match &b[8..12] {
                        b"vide" => "video",
                        b"soun" => "audio",
                        _ => "other",
                    };
            }
            (b"stbl", b"stsd") => {
                let t =
                    &mut info.tracks[current.ok_or_else(|| invalid("track context missing"))?];
                if t.codec_id.is_some() {
                    return Err(invalid("duplicate sample description"));
                }
                sample_description(r, &a, t)?;
            }
            _ => r.skipped += 1,
        }
    }
    if parent == *b"root" && (!seen.contains(b"moov")) {
        return Err(invalid("MP4/MOV movie metadata is absent"));
    }
    Ok(())
}
pub(super) fn inspect<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    info: &mut Info,
) -> Result<(), OperationError> {
    walk(r, info, 0, r.size, *b"root", 0, None)?;
    if info.time_base.is_none() {
        return Err(invalid("MP4 movie time metadata is absent"));
    }
    for t in &info.tracks {
        if t.time_base.is_none() || t.kind == "unknown" || t.codec_id.is_none() {
            return Err(invalid(
                "MP4 track metadata is incomplete or out of supported order",
            ));
        }
    }
    info.container = if info.major_brand.as_deref().is_none_or(|b| b == "qt  ") {
        "mov"
    } else {
        "mp4"
    };
    Ok(())
}
