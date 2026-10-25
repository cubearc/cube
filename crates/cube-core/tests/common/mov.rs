// Original 2x1 gray baseline JPEG fixtures generated with image 0.25.10, quality 100.
// No external image, encoder runtime or redistributability restriction is required.
pub const JPEGS: [&str; 2] = [
    "/9j/4AAQSkZJRgABAgAAAQABAAD/wAALCAABAAIBAREA/9sAQwABAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9oACAEBAAA/AP5/6//Z",
    "/9j/4AAQSkZJRgABAgAAAQABAAD/wAALCAABAAIBAREA/9sAQwABAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9oACAEBAAA/AP6AK//Z",
];
pub fn jpeg(which: usize) -> Vec<u8> {
    cube_core::encoding::base64_decode(JPEGS[which]).unwrap()
}
pub fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut b = ((body.len() + 8) as u32).to_be_bytes().to_vec();
    b.extend(kind);
    b.extend(body);
    b
}
fn table(kind: &[u8; 4], rows: &[Vec<u8>]) -> Vec<u8> {
    let mut b = vec![0; 4];
    b.extend((rows.len() as u32).to_be_bytes());
    for row in rows {
        b.extend(row);
    }
    atom(kind, &b)
}
pub fn movie(frames: &[Vec<u8>], wide: bool, fixed: bool) -> Vec<u8> {
    movie_with_chunks(frames, wide, fixed, &vec![1; frames.len()], false)
}
pub fn movie_with_chunks(
    frames: &[Vec<u8>],
    wide: bool,
    fixed: bool,
    groups: &[usize],
    sync: bool,
) -> Vec<u8> {
    assert_eq!(groups.iter().sum::<usize>(), frames.len());
    let mut out = atom(b"ftyp", b"qt  \0\0\0\0");
    let first = out.len() as u64 + 8;
    let mut payload = Vec::new();
    let mut offsets = Vec::new();
    for frame in frames {
        offsets.push(first + payload.len() as u64);
        payload.extend(frame);
    }
    out.extend(atom(b"mdat", &payload));
    let duration = (1..=frames.len() as u32).map(|i| i * 10).sum::<u32>();
    let mut mvhd = vec![0; 100];
    mvhd[12..16].copy_from_slice(&1000u32.to_be_bytes());
    mvhd[16..20].copy_from_slice(&duration.to_be_bytes());
    let mut tkhd = vec![0; 84];
    tkhd[12..16].copy_from_slice(&1u32.to_be_bytes());
    let mut mdhd = vec![0; 24];
    mdhd[12..16].copy_from_slice(&1000u32.to_be_bytes());
    mdhd[16..20].copy_from_slice(&duration.to_be_bytes());
    let mut hdlr = vec![0; 24];
    hdlr[8..12].copy_from_slice(b"vide");
    let mut entry = vec![0; 78];
    entry[6..8].copy_from_slice(&1u16.to_be_bytes());
    entry[24..26].copy_from_slice(&2u16.to_be_bytes());
    entry[26..28].copy_from_slice(&1u16.to_be_bytes());
    entry[74..76].copy_from_slice(&8u16.to_be_bytes());
    entry[76..78].copy_from_slice(&(-1i16).to_be_bytes());
    let mut tables = table(b"stsd", &[atom(b"jpeg", &entry)]);
    let mut stsz = vec![0; 4];
    stsz.extend((if fixed { frames[0].len() as u32 } else { 0 }).to_be_bytes());
    stsz.extend((frames.len() as u32).to_be_bytes());
    if !fixed {
        for f in frames {
            stsz.extend((f.len() as u32).to_be_bytes());
        }
    }
    tables.extend(atom(b"stsz", &stsz));
    tables.extend(table(
        b"stsc",
        &groups
            .iter()
            .enumerate()
            .map(|(i, n)| {
                [
                    ((i + 1) as u32).to_be_bytes(),
                    (*n as u32).to_be_bytes(),
                    1u32.to_be_bytes(),
                ]
                .concat()
            })
            .collect::<Vec<_>>(),
    ));
    let mut frame = 0;
    let offsets: Vec<_> = groups
        .iter()
        .map(|n| {
            let offset = offsets[frame];
            frame += n;
            offset
        })
        .collect();
    tables.extend(table(
        if wide { b"co64" } else { b"stco" },
        &offsets
            .iter()
            .map(|n| {
                if wide {
                    n.to_be_bytes().to_vec()
                } else {
                    (*n as u32).to_be_bytes().to_vec()
                }
            })
            .collect::<Vec<_>>(),
    ));
    if sync {
        tables.extend(table(
            b"stss",
            &(1..=frames.len() as u32)
                .map(|i| i.to_be_bytes().to_vec())
                .collect::<Vec<_>>(),
        ));
    }
    tables.extend(table(
        b"stts",
        &(1..=frames.len() as u32)
            .map(|i| [1u32.to_be_bytes(), (i * 10).to_be_bytes()].concat())
            .collect::<Vec<_>>(),
    ));
    let mut minf = atom(b"dinf", &table(b"dref", &[atom(b"url ", &[0, 0, 0, 1])]));
    minf.extend(atom(b"stbl", &tables));
    let mut mdia = atom(b"mdhd", &mdhd);
    mdia.extend(atom(b"hdlr", &hdlr));
    mdia.extend(atom(b"minf", &minf));
    let mut trak = atom(b"tkhd", &tkhd);
    trak.extend(atom(b"mdia", &mdia));
    let mut moov = atom(b"mvhd", &mvhd);
    moov.extend(atom(b"trak", &trak));
    out.extend(atom(b"moov", &moov));
    out
}
