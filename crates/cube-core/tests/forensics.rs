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
