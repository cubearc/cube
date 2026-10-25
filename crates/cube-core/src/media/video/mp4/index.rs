//! Checked sample-table indexing, separate from declared metadata inspection.
use super::*;

#[derive(Clone, Debug)]
pub(in crate::media::video) struct Sample {
    pub offset: u64,
    pub length: u32,
    pub start: u64,
    pub duration: u32,
}
pub(in crate::media::video) struct Index {
    pub samples: Vec<Sample>,
    pub width: u32,
    pub height: u32,
    pub time_base: TimeBase,
}
fn children<R: Read + Seek>(r: &mut Reader<'_, R>, a: Atom) -> Result<Vec<Atom>, OperationError> {
    let mut result = Vec::new();
    let mut at = a.body;
    while at < a.end {
        let child = atom(r, at, a.end, a.kind == *b"root")?;
        at = child.end;
        result.push(child);
    }
    Ok(result)
}
fn one(atoms: &[Atom], kind: &[u8; 4]) -> Result<Atom, OperationError> {
    let mut found = atoms.iter().filter(|a| &a.kind == kind);
    let result = *found
        .next()
        .ok_or_else(|| invalid("required extraction atom is absent"))?;
    if found.next().is_some() {
        return Err(invalid("duplicate extraction atom"));
    }
    Ok(result)
}
fn full<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
    size: usize,
) -> Result<Vec<u8>, OperationError> {
    let bytes = prefix(r, a, size)?;
    if bytes[..4] != [0; 4] {
        return Err(unsupported(
            "extraction requires version-zero tables with zero flags",
        ));
    }
    Ok(bytes)
}
fn table_count<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
    width: usize,
) -> Result<usize, OperationError> {
    let data = full(r, a, 8)?;
    let count = u32be(&data[4..8]) as usize;
    if count == 0 || count > MAX_RECORDS {
        return Err(limit("sample table must contain 1..=10000 records"));
    }
    if a.end - a.body != 8 + count as u64 * width as u64 {
        return Err(invalid(
            "sample table length disagrees with its record count",
        ));
    }
    Ok(count)
}
fn row<R: Read + Seek>(
    r: &mut Reader<'_, R>,
    a: &Atom,
    index: usize,
    width: usize,
) -> Result<Vec<u8>, OperationError> {
    r.record()?;
    r.at(a.body + 8 + index as u64 * width as u64, width)
}
pub(in crate::media::video) fn jpeg<R: Read + Seek>(
    source: &mut R,
    info: &Info,
    track_id: u64,
) -> Result<Index, OperationError> {
    if !["mp4", "mov"].contains(&info.container) {
        return Err(unsupported(
            "frame extraction supports only the documented MOV/JPEG profile",
        ));
    }
    let track = info
        .tracks
        .iter()
        .find(|t| t.id == track_id)
        .ok_or_else(|| invalid("selected track ID is unavailable"))?;
    if track.kind != "video" || track.codec_id.as_deref() != Some("jpeg") {
        return Err(unsupported(
            "selected track must contain independent baseline JPEG samples (jpeg)",
        ));
    }
    let mut r = Reader {
        source,
        size: info.file_bytes,
        records: info.records_inspected,
        read_bytes: info.metadata_bytes_read,
        skipped: info.skipped_records,
    };
    let root = children(
        &mut r,
        Atom {
            kind: *b"root",
            body: 0,
            end: info.file_bytes,
        },
    )?;
    let media: Vec<_> = root
        .iter()
        .filter(|a| a.kind == *b"mdat")
        .map(|a| (a.body, a.end))
        .collect();
    let moov = children(&mut r, one(&root, b"moov")?)?;
    let mut selected = None;
    for a in moov.iter().filter(|a| a.kind == *b"trak") {
        let items = children(&mut r, *a)?;
        let header = one(&items, b"tkhd")?;
        let version = prefix(&mut r, &header, 4)?[0];
        let offset = if version == 0 { 12 } else { 20 }; // Versions were checked by inspect.
        let bytes = prefix(&mut r, &header, offset + 4)?;
        if u64::from(u32be(&bytes[offset..offset + 4])) == track_id {
            selected = Some(items);
        }
    }
    let items = selected.ok_or_else(|| invalid("selected track structure is absent"))?;
    if items.iter().any(|a| [*b"edts", *b"tref"].contains(&a.kind)) {
        return Err(unsupported(
            "edit lists and track references are unsupported for extraction",
        ));
    }
    let mdia = children(&mut r, one(&items, b"mdia")?)?;
    let minf = children(&mut r, one(&mdia, b"minf")?)?;
    let dinf = children(&mut r, one(&minf, b"dinf")?)?;
    let dref = one(&dinf, b"dref")?;
    let bytes = full(&mut r, &dref, 8)?;
    if u32be(&bytes[4..8]) != 1 {
        return Err(unsupported(
            "exactly one self-contained data reference is required",
        ));
    }
    let reference = atom(&mut r, dref.body + 8, dref.end, false)?;
    if reference.kind != *b"url "
        || reference.end != dref.end
        || reference.end - reference.body != 4
        || prefix(&mut r, &reference, 4)? != [0, 0, 0, 1]
    {
        return Err(unsupported(
            "external or unsupported data references are never opened",
        ));
    }
    let tables = children(&mut r, one(&minf, b"stbl")?)?;
    for a in &tables {
        if ![
            *b"stsd", *b"stsz", *b"stsc", *b"stco", *b"co64", *b"stts", *b"stss", *b"free",
        ]
        .contains(&a.kind)
        {
            return Err(unsupported(
                "unsupported sample-table feature; no composition offsets, encryption or sample groups",
            ));
        }
    }
    let stsd = one(&tables, b"stsd")?;
    let bytes = full(&mut r, &stsd, 8)?;
    if u32be(&bytes[4..8]) != 1 {
        return Err(unsupported("one sample description is required"));
    }
    let entry = atom(&mut r, stsd.body + 8, stsd.end, false)?;
    let bytes = prefix(&mut r, &entry, 78)?;
    if entry.kind != *b"jpeg"
        || entry.end != stsd.end
        || entry.end - entry.body != 78
        || bytes[6..8] != [0, 1]
        || bytes[8..12] != [0; 4]
    {
        return Err(unsupported(
            "only one version-zero self-contained JPEG sample description without extensions is supported",
        ));
    }
    let stsz = one(&tables, b"stsz")?;
    let bytes = full(&mut r, &stsz, 12)?;
    let fixed = u32be(&bytes[4..8]);
    let n = u32be(&bytes[8..12]) as usize;
    if n == 0 || n > MAX_RECORDS {
        return Err(limit("video sample count must be 1..=10000"));
    }
    let expected = 12 + if fixed == 0 { n as u64 * 4 } else { 0 };
    if stsz.end - stsz.body != expected {
        return Err(invalid("sample-size table length mismatch"));
    }
    let mut sizes = Vec::with_capacity(n);
    for i in 0..n {
        let size = if fixed == 0 {
            r.record()?;
            u32be(&r.at(stsz.body + 12 + i as u64 * 4, 4)?)
        } else {
            fixed
        };
        if size == 0 {
            return Err(invalid("zero-length media sample"));
        }
        sizes.push(size);
    }
    let mut chunks = None;
    for kind in [b"stco", b"co64"] {
        if tables.iter().any(|a| &a.kind == kind) {
            if chunks.is_some() {
                return Err(invalid("both stco and co64 are ambiguous"));
            }
            let a = one(&tables, kind)?;
            let width = if kind == b"stco" { 4 } else { 8 };
            let count = table_count(&mut r, &a, width)?;
            let mut offsets = Vec::with_capacity(count);
            for i in 0..count {
                let b = row(&mut r, &a, i, width)?;
                offsets.push(if width == 4 {
                    u64::from(u32be(&b))
                } else {
                    u64be(&b)
                });
            }
            chunks = Some(offsets);
        }
    }
    let chunks = chunks.ok_or_else(|| invalid("chunk offset table is absent"))?;
    let stsc = one(&tables, b"stsc")?;
    let count = table_count(&mut r, &stsc, 12)?;
    let mut mappings = Vec::with_capacity(count);
    for i in 0..count {
        let b = row(&mut r, &stsc, i, 12)?;
        let first = u32be(&b[..4]) as usize;
        let per = u32be(&b[4..8]) as usize;
        if first == 0
            || first > chunks.len()
            || per == 0
            || per > n
            || u32be(&b[8..12]) != 1
            || mappings
                .last()
                .is_some_and(|&(previous, _)| first <= previous)
        {
            return Err(invalid("invalid sample-to-chunk relationship"));
        }
        mappings.push((first, per));
    }
    if mappings[0].0 != 1 {
        return Err(invalid("first sample-to-chunk run must start at chunk one"));
    }
    let mut samples = Vec::with_capacity(n);
    let mut mapping = 0;
    for (chunk, &offset) in chunks.iter().enumerate() {
        while mapping + 1 < mappings.len() && mappings[mapping + 1].0 <= chunk + 1 {
            mapping += 1;
        }
        let per = mappings[mapping].1;
        if per > n - samples.len() {
            return Err(invalid("chunks contain more samples than stsz"));
        }
        let mut at = offset;
        for &length in &sizes[samples.len()..samples.len() + per] {
            let end = at
                .checked_add(u64::from(length))
                .ok_or_else(|| invalid("sample extent overflows"))?;
            if !media.iter().any(|&(a, b)| at >= a && end <= b) {
                return Err(invalid("sample lies outside mdat payloads"));
            }
            samples.push(Sample {
                offset: at,
                length,
                start: 0,
                duration: 0,
            });
            at = end;
        }
    }
    if samples.len() != n {
        return Err(invalid("chunks do not account for every media sample"));
    }
    let mut extents: Vec<_> = samples
        .iter()
        .map(|s| (s.offset, s.offset + u64::from(s.length)))
        .collect();
    extents.sort_unstable();
    if extents.windows(2).any(|s| s[1].0 < s[0].1) {
        return Err(invalid("media sample ranges overlap"));
    }
    let stts = one(&tables, b"stts")?;
    let count = table_count(&mut r, &stts, 8)?;
    let mut position = 0usize;
    let mut time = 0u64;
    for i in 0..count {
        let b = row(&mut r, &stts, i, 8)?;
        let count = u32be(&b[..4]) as usize;
        let duration = u32be(&b[4..8]);
        if count == 0 || count > n - position || duration == 0 {
            return Err(invalid("invalid time-to-sample relationship"));
        }
        for sample in &mut samples[position..position + count] {
            sample.start = time;
            sample.duration = duration;
            time = time
                .checked_add(u64::from(duration))
                .ok_or_else(|| invalid("sample duration overflows"))?;
        }
        position += count;
    }
    if position != n || !matches!(track.duration_ticks,Some(Duration::Integer(t)) if t==time) {
        return Err(invalid(
            "sample times disagree with sample count or declared track duration",
        ));
    }
    if tables.iter().any(|a| a.kind == *b"stss") {
        let a = one(&tables, b"stss")?;
        if self::table_count(&mut r, &a, 4)? != n {
            return Err(unsupported("every JPEG frame must be a sync sample"));
        }
        for i in 0..n {
            if u32be(&row(&mut r, &a, i, 4)?) as usize != i + 1 {
                return Err(invalid("invalid JPEG sync-sample indexes"));
            }
        }
    }
    Ok(Index {
        samples,
        width: track.width.ok_or_else(|| invalid("missing coded width"))?,
        height: track
            .height
            .ok_or_else(|| invalid("missing coded height"))?,
        time_base: track
            .time_base
            .ok_or_else(|| invalid("missing track time base"))?,
    })
}
