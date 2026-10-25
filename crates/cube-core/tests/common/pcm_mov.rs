// Original uncompressed sample tables/payloads; no copyrighted media or codec.
fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
    out.extend(kind);
    out.extend(body);
    out
}
fn table(kind: &[u8; 4], rows: &[Vec<u8>]) -> Vec<u8> {
    let mut out = vec![0; 4];
    out.extend((rows.len() as u32).to_be_bytes());
    for row in rows {
        out.extend(row);
    }
    atom(kind, &out)
}
pub fn movie(payloads: &[Vec<u8>], channels: u16, rate: u32, wide: bool) -> Vec<u8> {
    chunked(payloads, channels, rate, wide, None)
}
pub fn chunked(
    payloads: &[Vec<u8>],
    channels: u16,
    rate: u32,
    wide: bool,
    groups: Option<&[usize]>,
) -> Vec<u8> {
    let mut result = atom(b"ftyp", b"qt  \0\0\0\0");
    let mut offset = result.len() as u64 + 8;
    result.extend(atom(b"mdat", &payloads.concat()));
    let mut mvhd = vec![0; 100];
    mvhd[12..16].copy_from_slice(&rate.to_be_bytes());
    mvhd[16..20]
        .copy_from_slice(&((payloads[0].len() / usize::from(channels)) as u32).to_be_bytes());
    let mut moov = atom(b"mvhd", &mvhd);
    for (i, payload) in payloads.iter().enumerate() {
        let frames = payload.len() / usize::from(channels);
        let groups = groups.map(Vec::from).unwrap_or_else(|| vec![frames]);
        assert_eq!(groups.iter().sum::<usize>(), frames);
        let mut tkhd = vec![0; 84];
        tkhd[12..16].copy_from_slice(&((i + 1) as u32).to_be_bytes());
        let mut mdhd = vec![0; 24];
        mdhd[12..16].copy_from_slice(&rate.to_be_bytes());
        mdhd[16..20].copy_from_slice(&(frames as u32).to_be_bytes());
        let mut hdlr = vec![0; 24];
        hdlr[8..12].copy_from_slice(b"soun");
        let mut entry = vec![0; 28];
        entry[6..8].copy_from_slice(&1u16.to_be_bytes());
        entry[16..18].copy_from_slice(&channels.to_be_bytes());
        entry[18..20].copy_from_slice(&8u16.to_be_bytes());
        entry[24..28].copy_from_slice(&(rate << 16).to_be_bytes());
        let mut tables = table(b"stsd", &[atom(b"raw ", &entry)]);
        tables.extend(atom(
            b"stsz",
            &[
                0u32.to_be_bytes(),
                u32::from(channels).to_be_bytes(),
                (frames as u32).to_be_bytes(),
            ]
            .concat(),
        ));
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
        let mut at = offset;
        let offsets: Vec<_> = groups
            .iter()
            .map(|n| {
                let old = at;
                at += (*n * usize::from(channels)) as u64;
                if wide {
                    old.to_be_bytes().to_vec()
                } else {
                    (old as u32).to_be_bytes().to_vec()
                }
            })
            .collect();
        tables.extend(table(if wide { b"co64" } else { b"stco" }, &offsets));
        tables.extend(table(
            b"stts",
            &[[(frames as u32).to_be_bytes(), 1u32.to_be_bytes()].concat()],
        ));
        let mut minf = atom(b"dinf", &table(b"dref", &[atom(b"url ", &[0, 0, 0, 1])]));
        minf.extend(atom(b"stbl", &tables));
        let mut mdia = atom(b"mdhd", &mdhd);
        mdia.extend(atom(b"hdlr", &hdlr));
        mdia.extend(atom(b"minf", &minf));
        let mut trak = atom(b"tkhd", &tkhd);
        trak.extend(atom(b"mdia", &mdia));
        moov.extend(atom(b"trak", &trak));
        offset += payload.len() as u64;
    }
    result.extend(atom(b"moov", &moov));
    result
}
