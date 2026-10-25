/// Original metadata-only fixtures. They contain no encoded audio/video packets.
pub fn atom(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut b = ((data.len() + 8) as u32).to_be_bytes().to_vec();
    b.extend(kind);
    b.extend(data);
    b
}
pub fn mp4(tracks: usize, version1: bool) -> Vec<u8> {
    let mut mvhd = vec![0u8; if version1 { 112 } else { 100 }];
    if version1 {
        mvhd[0] = 1;
        mvhd[20..24].copy_from_slice(&1000u32.to_be_bytes());
        mvhd[24..32].copy_from_slice(&2000u64.to_be_bytes());
    } else {
        mvhd[12..16].copy_from_slice(&1000u32.to_be_bytes());
        mvhd[16..20].copy_from_slice(&2000u32.to_be_bytes());
    }
    let mut moov = atom(b"mvhd", &mvhd);
    for i in 0..tracks {
        let mut tkhd = vec![0; 84];
        tkhd[12..16].copy_from_slice(&((i + 1) as u32).to_be_bytes());
        tkhd[76..80].copy_from_slice(&(1920u32 << 16).to_be_bytes());
        tkhd[80..84].copy_from_slice(&(1080u32 << 16).to_be_bytes());
        let mut mdhd = vec![0; 24];
        mdhd[12..16].copy_from_slice(&48000u32.to_be_bytes());
        mdhd[16..20].copy_from_slice(&96000u32.to_be_bytes());
        let mut hdlr = vec![0; 24];
        hdlr[8..12].copy_from_slice(b"vide");
        let mut entry = vec![0; 78];
        entry[24..26].copy_from_slice(&1280u16.to_be_bytes());
        entry[26..28].copy_from_slice(&720u16.to_be_bytes());
        let mut stsd = vec![0; 4];
        stsd.extend(1u32.to_be_bytes());
        stsd.extend(atom(b"avc1", &entry));
        let mut mdia = atom(b"mdhd", &mdhd);
        mdia.extend(atom(b"hdlr", &hdlr));
        mdia.extend(atom(b"minf", &atom(b"stbl", &atom(b"stsd", &stsd))));
        let mut trak = atom(b"tkhd", &tkhd);
        trak.extend(atom(b"mdia", &mdia));
        moov.extend(atom(b"trak", &trak));
    }
    let mut b = atom(b"ftyp", b"isom\0\0\0\0isom");
    b.extend(atom(b"free", b"opaque"));
    b.extend(atom(b"moov", &moov));
    b.extend(atom(b"mdat", b"unverified payload"));
    b
}
pub fn element(id: &[u8], data: &[u8]) -> Vec<u8> {
    let mut b = id.to_vec();
    if data.len() < 127 {
        b.push(0x80 | data.len() as u8);
    } else {
        b.extend(((data.len() as u16) | 0x4000).to_be_bytes());
    }
    b.extend(data);
    b
}
pub fn webm() -> Vec<u8> {
    let header = element(&[0x42, 0x82], b"webm");
    let mut info = element(&[0x2a, 0xd7, 0xb1], &1000000u32.to_be_bytes());
    info.extend(element(&[0x44, 0x89], &2500f64.to_be_bytes()));
    let mut track = element(&[0xd7], &[1]);
    track.extend(element(&[0x73, 0xc5], &[5]));
    track.extend(element(&[0x83], &[1]));
    track.extend(element(&[0x86], b"V_VP9"));
    let mut video = element(&[0xb0], &640u16.to_be_bytes());
    video.extend(element(&[0xba], &480u16.to_be_bytes()));
    track.extend(element(&[0xe0], &video));
    let mut segment = element(&[0x15, 0x49, 0xa9, 0x66], &info);
    segment.extend(element(
        &[0x16, 0x54, 0xae, 0x6b],
        &element(&[0xae], &track),
    ));
    segment.extend(element(&[0x1f, 0x43, 0xb6, 0x75], b"unverified payload"));
    let mut b = element(&[0x1a, 0x45, 0xdf, 0xa3], &header);
    b.extend(element(&[0x18, 0x53, 0x80, 0x67], &segment));
    b
}
