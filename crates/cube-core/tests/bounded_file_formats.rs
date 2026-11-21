use cube_core::{
    errors::ErrorCode,
    forensics::formats::{bmp, executable, mpeg, ogg, tiff, webp},
};

#[test]
fn bitmap_top_down_raster_is_structurally_bounded() {
    let mut file = vec![0; 58];
    file[0..2].copy_from_slice(b"BM");
    file[2..6].copy_from_slice(&58u32.to_le_bytes());
    file[10..14].copy_from_slice(&54u32.to_le_bytes());
    file[14..18].copy_from_slice(&40u32.to_le_bytes());
    file[18..22].copy_from_slice(&1i32.to_le_bytes());
    file[22..26].copy_from_slice(&(-1i32).to_le_bytes());
    file[26..28].copy_from_slice(&1u16.to_le_bytes());
    file[28..30].copy_from_slice(&24u16.to_le_bytes());
    let report = bmp::inspect(&file).unwrap();
    assert_eq!((report.width, report.height, report.top_down), (1, 1, true));
    assert_eq!((report.raster.offset, report.raster.length), (54, 4));
    file[10] = 50;
    assert_eq!(
        bmp::inspect(&file).unwrap_err().code,
        ErrorCode::InvalidInput
    );
}

#[test]
fn webp_checked_riff_padding_and_dimensions() {
    let mut file = b"RIFF\x12\x00\x00\x00WEBPVP8L\x05\x00\x00\x00\x2f\x00\x00\x00\x00\x00".to_vec();
    let report = webp::inspect(&file).unwrap();
    assert_eq!(
        (report.width, report.height, report.logical_end),
        (1, 1, 26)
    );
    file[25] = 1;
    assert!(webp::inspect(&file).is_err());
}

#[test]
fn tiff_retains_inline_fields_and_stops_directory_cycles() {
    let mut file = b"II\x2a\x00\x08\x00\x00\x00\x01\x00\x00\x01\x03\x00\x01\x00\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00".to_vec();
    let report = tiff::inspect(&file).unwrap();
    assert_eq!(report.directories[0].entries[0].tag, 256);
    assert_eq!(
        report.directories[0].entries[0].inline_hex.as_deref(),
        Some("1000")
    );
    file[22] = 8;
    assert!(tiff::inspect(&file).is_err());
}

#[test]
fn executable_metadata_checks_table_extents_before_parse() {
    let mut elf = vec![0; 64];
    elf[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    elf[16..20].copy_from_slice(&[2, 0, 62, 0]);
    elf[20] = 1;
    elf[24..32].copy_from_slice(&0x1122_3344_5566_7788u64.to_le_bytes());
    elf[52] = 64;
    elf[54] = 56;
    elf[58] = 64;
    let report = executable::inspect(&elf).unwrap();
    assert_eq!(report.architecture, "x86_64");
    assert_eq!(report.entry_point, "0x1122334455667788");
    elf[56] = 1;
    assert!(executable::inspect(&elf).is_err());
}

#[test]
fn parses_id3v23_text_and_mpeg_layer_three() {
    let mut file = b"ID3\x03\x00\x00\x00\x00\x00\x10TIT2\x00\x00\x00\x06\x00\x00\x00Title".to_vec();
    file.extend([0xff, 0xfb, 0x90, 0x00]);
    file.resize(26 + 417, 0);
    let report = mpeg::inspect(&file).unwrap();
    let tag = report.tag.unwrap();
    assert_eq!(tag.frames[0].text.as_deref(), Some("Title"));
    assert_eq!(
        (report.frames[0].sample_rate, report.frames[0].range.length),
        (44_100, 417)
    );
    file.pop();
    assert!(mpeg::inspect(&file).is_err());
}

fn page(flags: u8, sequence: u32, granule: u64, lace: &[u8], data: &[u8]) -> Vec<u8> {
    let mut raw = b"OggS\x00".to_vec();
    raw.push(flags);
    raw.extend(granule.to_le_bytes());
    raw.extend(7_u32.to_le_bytes());
    raw.extend(sequence.to_le_bytes());
    raw.extend([0; 4]);
    raw.push(lace.len() as u8);
    raw.extend(lace);
    raw.extend(data);
    // Direct bit-by-bit oracle, independent of the implementation's lookup table.
    let mut crc = 0u32;
    for &byte in &raw {
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            crc = (crc << 1)
                ^ if crc & 0x8000_0000 != 0 {
                    0x04c1_1db7
                } else {
                    0
                };
        }
    }
    raw[22..26].copy_from_slice(&crc.to_le_bytes());
    raw
}

#[test]
fn ogg_checks_checksum_sequence_and_packet_continuations() {
    let start = page(2, 0, u64::MAX, &[255], &[1; 255]);
    let end = page(5, 1, 222, &[2], &[2; 2]);
    let chain = [start.clone(), end].concat();
    let report = ogg::inspect(&chain).unwrap();
    assert_eq!(
        report.pages[0].fragments[0].packet,
        report.pages[1].fragments[0].packet
    );
    assert!(report.pages[1].fragments[0].complete);
    assert!(ogg::inspect(&[start, page(4, 1, 222, &[1], &[1])].concat()).is_err());
    let mut damaged = chain;
    *damaged.last_mut().unwrap() ^= 1;
    assert!(ogg::inspect(&damaged).is_err());
}

#[test]
fn bitmap_validates_nonoverlapping_bitfield_masks() {
    let mut data = vec![0; 70];
    data[..2].copy_from_slice(b"BM");
    for (at, value) in [
        (2, 70u32),
        (10, 66),
        (14, 40),
        (18, 1),
        (22, 1),
        (30, 3),
        (34, 4),
        (54, 0xf800),
        (58, 0x07e0),
        (62, 0x001f),
    ] {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    data[26] = 1;
    data[28] = 16;
    let report = bmp::inspect(&data).unwrap();
    assert_eq!(report.compression, "bi_bitfields");
    assert_eq!(
        report.channel_masks.unwrap(),
        ["0000f800", "000007e0", "0000001f"]
    );
    data[58..62].copy_from_slice(&0xf800u32.to_le_bytes());
    assert!(bmp::inspect(&data).is_err());
}

#[test]
fn tiff_big_endian_links_and_unknown_type_are_inert() {
    let mut bytes = b"MM\x00\x2a\x00\x00\x00\x08".to_vec();
    bytes.extend(2u16.to_be_bytes());
    for (tag, kind, count, data) in [(330u16, 13u16, 1u32, 38u32), (400, 99, 0xffff, 9)] {
        bytes.extend(tag.to_be_bytes());
        bytes.extend(kind.to_be_bytes());
        bytes.extend(count.to_be_bytes());
        bytes.extend(data.to_be_bytes());
    }
    bytes.extend([0; 10]); // next offset, child count and child next pointer
    let report = tiff::inspect(&bytes).unwrap();
    assert_eq!(report.directories.len(), 2);
    assert_eq!(report.directories[0].sub_ifds, [38]);
    assert!(report.directories[0].entries[1].unsupported_type);
    bytes[21] = 8;
    assert!(tiff::inspect(&bytes).is_err());
}

#[test]
fn parses_bounded_pe32_and_pe32_plus_header_and_directories() {
    for (magic, base, count_offset, machine, format) in [
        (0x10bu16, 96usize, 92usize, 0x14cu16, "pe32"),
        (0x20bu16, 112usize, 108usize, 0x8664u16, "pe32_plus"),
    ] {
        let mut bytes = vec![0u8; 1024];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60] = 64;
        let nt = 64;
        let opt = nt + 24;
        bytes[nt..nt + 4].copy_from_slice(b"PE\0\0");
        bytes[nt + 4..nt + 6].copy_from_slice(&machine.to_le_bytes());
        bytes[nt + 6] = 1;
        bytes[nt + 20..nt + 22].copy_from_slice(&((base + 8) as u16).to_le_bytes());
        bytes[opt..opt + 2].copy_from_slice(&magic.to_le_bytes());
        for (j, n) in [
            (16, 0x1000u32),
            (56, 0x2000),
            (60, 512),
            (count_offset, 1),
            (base, 0x1010),
            (base + 4, 16),
        ] {
            bytes[opt + j..opt + j + 4].copy_from_slice(&n.to_le_bytes());
        }
        let section = opt + base + 8;
        for (j, n) in [(8, 0x100u32), (12, 0x1000), (16, 512), (20, 512)] {
            bytes[section + j..section + j + 4].copy_from_slice(&n.to_le_bytes());
        }
        let report = executable::inspect(&bytes).unwrap();
        assert_eq!(report.format, format);
        assert_eq!(report.entry_point, "0x1000");
        assert_eq!(report.data_directories[0].file.unwrap().offset, 528);
        bytes[opt + base + 1] ^= 0x20;
        assert!(executable::inspect(&bytes).is_err());
    }
}

#[test]
fn mpeg_tag_unsynchronization_preserves_offsets_and_handles_v24_footer() {
    let data = b"ID3\x03\x00\x80\x00\x00\x00\x0eTIT2\x00\x00\x00\x03\x00\x00\x00\xff\x00\xe0";
    let report = mpeg::inspect(data).unwrap();
    let tag = report.tag.unwrap();
    assert!(tag.unsynchronized);
    assert_eq!(tag.frames[0].payload.length, 4);
    assert_eq!(tag.frames[0].text.as_deref(), Some("ÿà"));

    let v24 = b"ID3\x04\x00\x10\x00\x00\x00\x0dTIT2\x00\x00\x00\x03\x00\x00\x03hi3DI\x04\x00\x10\x00\x00\x00\x0d";
    let tag = mpeg::inspect(v24).unwrap().tag.unwrap();
    assert_eq!(tag.version, "2.4");
    assert_eq!(tag.footer.unwrap().offset, 23);
    assert_eq!(tag.frames[0].text.as_deref(), Some("hi"));
}
