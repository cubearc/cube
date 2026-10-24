mod common;
use common::Workspace;
use cube_core::{
    errors::{ErrorCode, OperationError},
    forensics::{self, ByteRange, inspect, signatures},
};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

#[test]
fn signature_rules_use_exact_evidence_not_names() {
    for (data, format) in [
        (&b"\x89PNG\r\n\x1a\n"[..], "png"),
        (&b"\xff\xd8\xff"[..], "jpeg"),
        (&b"GIF87a"[..], "gif"),
        (&b"GIF89a"[..], "gif"),
        (&b"%PDF-1.7"[..], "pdf"),
        (&b"PK\x03\x04"[..], "zip"),
        (&b"PK\x05\x06"[..], "zip"),
        (&b"\x1f\x8b\x08"[..], "gzip"),
        (&b"RIFF1234WAVE"[..], "wave"),
        (&b"II\x2a\0"[..], "tiff"),
        (&b"MM\0\x2a"[..], "tiff"),
    ] {
        let result = signatures::detect(data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].format, format);
        assert_eq!(result[0].offset, 0);
        assert!(!result[0].evidence_hex.is_empty());
        for length in 0..data.len().min(if format == "pdf" { 5 } else { data.len() }) {
            if format != "pdf" {
                assert!(signatures::detect(&data[..length]).is_empty());
            }
        }
    }
    assert!(signatures::detect(b"RIFF1234NOPE").is_empty());
    assert!(signatures::detect(b"unknown").is_empty());
}

#[test]
fn embedded_signatures_cross_buffers_once_and_report_limits() {
    let mut bytes = vec![0; 65533];
    bytes.extend_from_slice(b"\x89PNG\r\n\x1a\n%PDF-");
    let scan = signatures::scan(&mut Cursor::new(&bytes), ByteRange::default(), None, 10).unwrap();
    assert!(scan.complete);
    assert_eq!(scan.scanned_bytes, bytes.len() as u64);
    assert_eq!(
        scan.candidates.iter().map(|c| c.offset).collect::<Vec<_>>(),
        [65533, 65541]
    );
    let result = signatures::scan(
        &mut Cursor::new(&bytes),
        ByteRange {
            offset: 65530,
            length: None,
        },
        Some("pdf"),
        10,
    )
    .unwrap();
    assert_eq!(result.candidates[0].offset, 65541);
    let result = signatures::scan(&mut Cursor::new(&bytes), ByteRange::default(), None, 1).unwrap();
    assert!(!result.complete);
    assert_eq!(result.candidates.len(), 1);
    assert!(
        signatures::scan(
            &mut Cursor::new(&bytes),
            ByteRange::default(),
            Some("missing"),
            10
        )
        .is_err()
    );
    for cap in [0, 10001] {
        assert!(
            signatures::scan(&mut Cursor::new(&bytes), ByteRange::default(), None, cap).is_err()
        );
    }
    assert!(
        signatures::scan(&mut Cursor::new(b""), ByteRange::default(), None, 10)
            .unwrap()
            .complete
    );
}

#[test]
fn range_validation_and_exact_binary_copy() {
    for range in [
        ByteRange {
            offset: u64::MAX,
            length: Some(2),
        },
        ByteRange {
            offset: 5,
            length: Some(0),
        },
        ByteRange {
            offset: 1,
            length: Some(4),
        },
    ] {
        assert_eq!(
            range.resolve(4, 10).unwrap_err().code,
            ErrorCode::InvalidParameter
        );
    }
    assert_eq!(
        ByteRange::default().resolve(100, 10).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut cursor = Cursor::new([0, 255, 128, 10]);
    assert_eq!(
        forensics::read_range(
            &mut cursor,
            ByteRange {
                offset: 1,
                length: Some(2)
            }
        )
        .unwrap(),
        [255, 128]
    );
    let mut output = Vec::new();
    assert_eq!(
        forensics::copy_range(&mut cursor, &mut output, 0, 4).unwrap(),
        4
    );
    assert_eq!(output, [0, 255, 128, 10]);
    forensics::copy_range(&mut cursor, &mut output, 4, 0).unwrap();
    assert!(forensics::copy_range(&mut cursor, &mut output, 4, 1).is_err());
}

#[test]
fn extraction_never_overwrites_sources_aliases_or_existing_destinations() {
    let dir = Workspace::new();
    let source = dir.path("source.bin");
    let target = dir.path("target.bin");
    std::fs::write(&source, [0, 255, 10]).unwrap();
    assert_eq!(forensics::extract_range(&source, &target, 1, 2).unwrap(), 2);
    assert_eq!(std::fs::read(&target).unwrap(), [255, 10]);
    assert!(forensics::extract_range(&source, &target, 0, 1).is_err());
    assert!(forensics::extract_range(&source, &source, 0, 1).is_err());
    let hard = dir.path("hard");
    std::fs::hard_link(&source, &hard).unwrap();
    assert!(forensics::extract_range(&source, &hard, 0, 1).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), [0, 255, 10]);
    let invalid = dir.path("invalid");
    assert!(forensics::extract_range(&source, &invalid, 100, 1).is_err());
    assert!(!invalid.exists());
    assert!(forensics::open_regular(&dir.path("missing")).is_err());
    assert!(forensics::open_regular(source.parent().unwrap()).is_err());
    let info = inspect::file_info(&source).unwrap();
    assert_eq!(info.size, 3);
    assert!(info.regular_file);
    assert!(info.signatures.is_empty());
    #[cfg(unix)]
    {
        let link = dir.path("link");
        std::os::unix::fs::symlink(&source, &link).unwrap();
        assert!(forensics::open_regular(&link).is_err());
        assert!(forensics::extract_range(&source, &link, 0, 1).is_err());
    }
}

struct FailingWriter;
impl Write for FailingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct ShortReader(Cursor<Vec<u8>>);
impl Read for ShortReader {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.0.read(b)
    }
}
impl Seek for ShortReader {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        if matches!(p, SeekFrom::End(0)) {
            Ok(100)
        } else {
            self.0.seek(p)
        }
    }
}
#[test]
fn copying_detects_short_reads_and_writer_failures() {
    assert_eq!(
        forensics::copy_range(&mut Cursor::new([1]), &mut FailingWriter, 0, 1)
            .unwrap_err()
            .code,
        ErrorCode::Io
    );
    assert!(
        forensics::copy_range(
            &mut ShortReader(Cursor::new(vec![1])),
            &mut Vec::new(),
            0,
            20
        )
        .is_err()
    );
    let error = OperationError::new(ErrorCode::UnsupportedFormat, "unsupported");
    assert_eq!(error.code.as_str(), "unsupported_format");
}

#[test]
fn hex_rows_preserve_raw_bytes_offsets_and_ascii() {
    use forensics::ranges::hex_rows;
    let rows = hex_rows(
        &mut Cursor::new(b"\0 A\xffB"),
        ByteRange {
            offset: 1,
            length: Some(4),
        },
        3,
    )
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].offset, 1);
    assert_eq!(rows[0].hex, "2041ff");
    assert_eq!(rows[0].ascii, " A.");
    assert_eq!(rows[0].bytes, b" A\xff");
    assert_eq!(rows[1].offset, 4);
    assert_eq!(rows[1].bytes, b"B");
    for width in [0, 257] {
        assert!(hex_rows(&mut Cursor::new(b""), ByteRange::default(), width).is_err());
    }
    assert!(
        hex_rows(&mut Cursor::new(b""), ByteRange::default(), 16)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn printable_runs_include_eof_and_join_buffer_boundaries() {
    use forensics::ranges::strings;
    let mut data = vec![0; 65534];
    data.extend_from_slice(b"hello world\0short\xfftail");
    let result = strings(&mut Cursor::new(&data), ByteRange::default(), 4, 10).unwrap();
    assert!(result.complete);
    assert_eq!(
        result
            .strings
            .iter()
            .map(|s| (s.offset, s.length, s.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (65534, 11, "hello world"),
            (65546, 5, "short"),
            (65552, 4, "tail")
        ]
    );
    let result = strings(&mut Cursor::new(b"a\0bb\0ccc"), ByteRange::default(), 2, 1).unwrap();
    assert!(!result.complete);
    assert_eq!(result.strings[0].text, "bb");
    assert!(
        strings(&mut Cursor::new(b""), ByteRange::default(), 1, 1)
            .unwrap()
            .complete
    );
    assert!(
        strings(&mut Cursor::new(b"\xff\0"), ByteRange::default(), 1, 1)
            .unwrap()
            .strings
            .is_empty()
    );
    assert!(strings(&mut Cursor::new(b""), ByteRange::default(), 0, 1).is_err());
    assert_eq!(
        strings(
            &mut Cursor::new(vec![b'a'; 1024 * 1024 + 1]),
            ByteRange::default(),
            1,
            1
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn binary_comparison_counts_all_differences_with_bounded_listing() {
    use forensics::ranges::compare;
    let result = compare(
        &mut Cursor::new([0, 255, 128]),
        &mut Cursor::new([0, 0]),
        ByteRange::default(),
        1,
    )
    .unwrap();
    assert!(!result.equal);
    assert_eq!(result.differing_bytes, 2);
    assert!(!result.complete);
    assert_eq!(result.differences[0].offset, 1);
    assert_eq!(result.differences[0].left, Some(255));
    assert_eq!(result.differences[0].right, Some(0));
    let result = compare(
        &mut Cursor::new([0, 255, 128]),
        &mut Cursor::new([0, 0]),
        ByteRange {
            offset: 2,
            length: Some(1),
        },
        10,
    )
    .unwrap();
    assert_eq!(result.differences[0].right, None);
    assert!(result.complete);
    assert!(
        compare(
            &mut Cursor::new(b""),
            &mut Cursor::new(b""),
            ByteRange::default(),
            10
        )
        .unwrap()
        .equal
    );
    let mut first = vec![0; 65537];
    let second = first.clone();
    first[65536] = 255;
    let result = compare(
        &mut Cursor::new(first),
        &mut Cursor::new(second),
        ByteRange::default(),
        10,
    )
    .unwrap();
    assert_eq!(result.differences[0].offset, 65536);
}

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut result = (data.len() as u32).to_be_bytes().to_vec();
    result.extend(kind);
    result.extend(data);
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(kind);
    hasher.update(data);
    result.extend(hasher.finalize().to_be_bytes());
    result
}
fn png_fixture(trailer: &[u8]) -> Vec<u8> {
    let mut result = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = vec![];
    header.extend(1u32.to_be_bytes());
    header.extend(1u32.to_be_bytes());
    header.extend([8, 2, 0, 0, 0]);
    result.extend(png_chunk(b"IHDR", &header));
    result.extend(png_chunk(
        b"IDAT",
        &[0x78, 0x9c, 0x63, 0x60, 0x60, 0x60, 0, 0, 4, 0, 1],
    ));
    result.extend(png_chunk(b"IEND", b""));
    result.extend(trailer);
    result
}
#[test]
fn png_boundaries_require_framing_crc_and_order_evidence() {
    use forensics::png::appended;
    for trailer in [b"".as_slice(), b"\0\xff", b"IENDmore"] {
        let data = png_fixture(trailer);
        let result = appended(&mut Cursor::new(&data)).unwrap();
        assert_eq!(result.logical_end, (data.len() - trailer.len()) as u64);
        assert_eq!(result.trailing_bytes, trailer.len() as u64);
        assert_eq!(result.validated_chunks, 3);
    }
    let data = png_fixture(b"");
    for length in [0, 1, 7, 8, 12, 32, data.len() - 1] {
        assert!(appended(&mut Cursor::new(&data[..length])).is_err());
    }
    let mut bad = data.clone();
    bad[29] ^= 1;
    assert_eq!(
        appended(&mut Cursor::new(&bad)).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    let mut bad = data.clone();
    bad[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(appended(&mut Cursor::new(bad)).is_err());
    assert_eq!(
        appended(&mut Cursor::new(b"not png!!!")).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let header = &data[..33];
    for chunks in [
        [png_chunk(b"IEND", b"")].concat(),
        [
            png_chunk(b"IDAT", b""),
            png_chunk(b"tEXt", b"IEND"),
            png_chunk(b"IDAT", b""),
            png_chunk(b"IEND", b""),
        ]
        .concat(),
        [
            png_chunk(b"ABCD", b""),
            png_chunk(b"IDAT", b""),
            png_chunk(b"IEND", b""),
        ]
        .concat(),
        [
            png_chunk(b"acTL", b"12345678"),
            png_chunk(b"IDAT", b""),
            png_chunk(b"IEND", b""),
        ]
        .concat(),
    ] {
        let mut bad = header.to_vec();
        bad.extend(chunks);
        assert!(appended(&mut Cursor::new(bad)).is_err());
    }
    let mut palette = header.to_vec();
    palette.extend(png_chunk(b"PLTE", b"\0\0\0"));
    palette.extend(png_chunk(b"IDAT", b""));
    palette.extend(png_chunk(b"IEND", b""));
    assert!(appended(&mut Cursor::new(palette)).is_ok());
}

fn tiff_fixture(little: bool) -> Vec<u8> {
    let u16_bytes = |n: u16| {
        if little {
            n.to_le_bytes()
        } else {
            n.to_be_bytes()
        }
    };
    let u32_bytes = |n: u32| {
        if little {
            n.to_le_bytes()
        } else {
            n.to_be_bytes()
        }
    };
    let fields: Vec<(u16, u16, u32, Vec<u8>)> = vec![
        (0x010f, 2, 5, b"Cube\0".to_vec()),
        (0x011a, 5, 1, [u32_bytes(72), u32_bytes(1)].concat()),
        (0x0128, 3, 1, u16_bytes(2).to_vec()),
        (0xc000, 1, 3, vec![0, 128, 255]),
        (0xc000, 6, 1, vec![255]),
        (0xc001, 7, 3, vec![1, 2, 3]),
        (0xc002, 8, 1, u16_bytes(u16::MAX).to_vec()),
        (0xc003, 9, 1, u32_bytes(u32::MAX).to_vec()),
        (0xc004, 10, 1, [u32_bytes(u32::MAX), u32_bytes(2)].concat()),
        (0xc005, 11, 1, u32_bytes(f32::NAN.to_bits()).to_vec()),
        (
            0xc006,
            12,
            1,
            if little {
                f64::INFINITY.to_bits().to_le_bytes().to_vec()
            } else {
                f64::INFINITY.to_bits().to_be_bytes().to_vec()
            },
        ),
        (0xc007, 4, 1, u32_bytes(123).to_vec()),
        (0xc008, 99, 1, vec![0]),
    ];
    let base = 8 + 2 + fields.len() * 12 + 4;
    let mut result = if little {
        b"II\x2a\0".to_vec()
    } else {
        b"MM\0\x2a".to_vec()
    };
    result.extend(u32_bytes(8));
    result.extend(u16_bytes(fields.len() as u16));
    let mut payload = Vec::new();
    for (tag, kind, count, mut data) in fields {
        result.extend(u16_bytes(tag));
        result.extend(u16_bytes(kind));
        result.extend(u32_bytes(count));
        if data.len() <= 4 {
            data.resize(4, 0);
            result.extend(data);
        } else {
            result.extend(u32_bytes((base + payload.len()) as u32));
            payload.extend(data);
        }
    }
    result.extend(u32_bytes(0));
    result.extend(payload);
    result
}
#[test]
fn exif_preserves_types_duplicates_endianness_and_units() {
    use forensics::exif::{TagValue, inspect, read};
    for little in [false, true] {
        let tiff = tiff_fixture(little);
        let result = inspect(&tiff).unwrap();
        assert!(result.present);
        assert_eq!(result.little_endian, Some(little));
        assert_eq!(result.tags.len(), 13);
        assert!(matches!(&result.tags[0].value,TagValue::Ascii(v) if v[0]==b"Cube"));
        assert!(matches!(&result.tags[1].value,TagValue::Rational(v) if v[0]==(72,1)));
        assert!(result.tags[1].display.contains("72"));
        assert_eq!(result.tags.iter().filter(|t| t.tag == 0xc000).count(), 2);
        let mut jpeg = b"\xff\xd8\xff\xe1".to_vec();
        jpeg.extend(((tiff.len() + 8) as u16).to_be_bytes());
        jpeg.extend(b"Exif\0\0");
        jpeg.extend(tiff);
        jpeg.extend(b"\xff\xd9");
        assert_eq!(read(&mut Cursor::new(jpeg)).unwrap().tags.len(), 13);
    }
    assert!(!inspect(b"\xff\xd8\xff\xd9").unwrap().present);
    assert_eq!(
        inspect(b"unknown").unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn exif_rejects_malformed_and_excessive_metadata() {
    use forensics::exif::{MAX_METADATA_BYTES, inspect};
    for bytes in [
        b"II".as_slice(),
        b"MMxx0000",
        b"\xff\xd8",
        b"\xff\xd8\xff\xe1\0\x01",
        b"\xff\xd8\0",
        b"\xff\xd8\xff\xe1\0\x20",
        b"II\x2a\0\xff\xff\xff\x7f",
    ] {
        assert!(inspect(bytes).is_err());
    }
    let mut cyclic = b"II\x2a\0\x08\0\0\0\0\0\x08\0\0\0".to_vec();
    assert!(inspect(&cyclic).is_err());
    cyclic.resize(MAX_METADATA_BYTES + 1, 0);
    assert_eq!(inspect(&cyclic).unwrap_err().code, ErrorCode::ResourceLimit);
    let excessive = b"II\x2a\0\x08\0\0\0\x01\x10";
    assert_eq!(
        inspect(excessive).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut bad = tiff_fixture(true);
    bad[14..18].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(inspect(&bad).unwrap_err().code, ErrorCode::ResourceLimit);
    let mut bad = tiff_fixture(true);
    bad[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(inspect(&bad).unwrap_err().code, ErrorCode::InvalidInput);
    let absent = b"II\x2a\0\0\0\0\0";
    assert!(!inspect(absent).unwrap().present);
}

#[test]
fn large_sparse_inputs_fail_before_unbounded_reads_or_allocations() {
    let dir = Workspace::new();
    let path = dir.path("large");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    file.set_len(forensics::MAX_SCAN_BYTES + 128).unwrap();
    assert_eq!(
        forensics::read_range(&mut file, ByteRange::default())
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        forensics::ranges::hex_rows(&mut file, ByteRange::default(), 16)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        forensics::ranges::compare(&mut file, &mut Cursor::new(b""), ByteRange::default(), 1)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let header = png_fixture(b"");
    file.seek(SeekFrom::Start(0)).unwrap();
    file.write_all(&header[..33]).unwrap();
    file.write_all(&(forensics::MAX_SCAN_BYTES as u32).to_be_bytes())
        .unwrap();
    file.write_all(b"IDAT").unwrap();
    assert_eq!(
        forensics::png::appended(&mut file).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn png_palette_rules_reject_invalid_color_models_and_index_depths() {
    let base = png_fixture(b"");
    for (depth, color, palette) in [
        (8, 0, b"\0\0\0".as_slice()),
        (1, 3, b"123456789".as_slice()),
    ] {
        let mut header = base[16..29].to_vec();
        header[8] = depth;
        header[9] = color;
        let mut data = b"\x89PNG\r\n\x1a\n".to_vec();
        data.extend(png_chunk(b"IHDR", &header));
        data.extend(png_chunk(b"PLTE", palette));
        data.extend(png_chunk(b"IDAT", b""));
        data.extend(png_chunk(b"IEND", b""));
        assert_eq!(
            forensics::png::appended(&mut Cursor::new(data))
                .unwrap_err()
                .code,
            ErrorCode::InvalidInput
        );
    }
}
