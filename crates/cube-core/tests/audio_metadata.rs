use cube_core::{
    errors::ErrorCode,
    media::metadata::{self, MAX_FIELD},
};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
fn block(kind: u8, last: bool, data: &[u8]) -> Vec<u8> {
    let mut out = vec![
        kind | if last { 128 } else { 0 },
        (data.len() >> 16) as u8,
        (data.len() >> 8) as u8,
        data.len() as u8,
    ];
    out.extend(data);
    out
}
fn flac(samples: u64, comments: Option<&[&str]>) -> Vec<u8> {
    let mut info = [0u8; 34];
    info[..2].copy_from_slice(&16u16.to_be_bytes());
    info[2..4].copy_from_slice(&16u16.to_be_bytes());
    let packed = (8000u64 << 44) | (1u64 << 41) | (15u64 << 36) | samples;
    info[10..18].copy_from_slice(&packed.to_be_bytes());
    let mut out = b"fLaC".to_vec();
    out.extend(block(0, comments.is_none(), &info));
    if let Some(fields) = comments {
        let mut data = 4u32.to_le_bytes().to_vec();
        data.extend(b"Cube");
        data.extend((fields.len() as u32).to_le_bytes());
        for field in fields {
            data.extend((field.len() as u32).to_le_bytes());
            data.extend(field.as_bytes());
        }
        out.extend(block(4, true, &data));
    }
    out
}
struct Counted {
    inner: Cursor<Vec<u8>>,
    read: usize,
}
impl Read for Counted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read += n;
        Ok(n)
    }
}
impl Seek for Counted {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
#[test]
fn native_flac_metadata_reports_declared_fields_tags_and_unknowns() {
    let input = flac(1600, Some(&["TITLE=First", "TITLE=Second", "X-CUBE=雪"]));
    let r = metadata::inspect(&mut Cursor::new(&input)).unwrap();
    assert_eq!(
        (r.sample_rate, r.channels, r.bits_per_sample),
        (8000, 2, 16)
    );
    assert_eq!(r.declared_total_samples, Some(1600));
    assert_eq!(r.duration_milliseconds, Some(200));
    assert_eq!(r.comments.len(), 3);
    assert_eq!(r.comments[0].value, "First");
    assert_eq!(r.comments[1].value, "Second");
    assert_eq!(r.comments[2].value, "雪");
    assert_eq!(r.vendor.as_deref(), Some("Cube"));
    assert!(r.declared_md5.is_none());
    assert_eq!(r.verification, "metadata_only_audio_payload_unverified");
    assert_eq!(r.metadata_bytes, input.len());
    let r = metadata::inspect(&mut Cursor::new(flac(0, None))).unwrap();
    assert!(r.declared_total_samples.is_none());
    assert!(r.duration_milliseconds.is_none());
    assert!(r.vendor.is_none());
    assert!(r.comments.is_empty());
    let mut bytes = flac(8000, None);
    let metadata_size = bytes.len();
    bytes.extend(vec![255; 100000]);
    let mut counted = Counted {
        inner: Cursor::new(bytes),
        read: 0,
    };
    let r = metadata::inspect(&mut counted).unwrap();
    assert_eq!(counted.read, metadata_size);
    assert_eq!(r.file_bytes, 100000 + metadata_size as u64);
}
#[test]
fn malformed_truncated_duplicate_and_unsupported_metadata_fails_safely() {
    let input = flac(1600, Some(&["TITLE=x"]));
    for n in 0..input.len() {
        assert!(
            metadata::inspect(&mut Cursor::new(&input[..n])).is_err(),
            "prefix {n}"
        );
    }
    for prefix in [b"ID3\0".to_vec(), b"OggS".to_vec(), b"not!".to_vec()] {
        assert_eq!(
            metadata::inspect(&mut Cursor::new(prefix))
                .unwrap_err()
                .code,
            ErrorCode::UnsupportedFormat
        );
    }
    let mut dup = flac(1, None);
    dup[4] = 0;
    dup.extend(block(0, true, &[0; 34]));
    assert!(metadata::inspect(&mut Cursor::new(dup)).is_err());
    let mut bad = flac(1, None);
    bad[4] = 0;
    bad.extend(block(7, true, &[]));
    assert_eq!(
        metadata::inspect(&mut Cursor::new(bad)).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut bad = flac(1, None);
    bad[18..26].fill(0);
    assert!(metadata::inspect(&mut Cursor::new(bad)).is_err());
    let mut dup = flac(1, Some(&[]));
    dup[42] &= 127;
    dup.extend(block(4, true, &[0; 8]));
    assert!(metadata::inspect(&mut Cursor::new(dup)).is_err());
    for field in ["BROKEN", "=empty", "BAD\nKEY=value"] {
        assert!(metadata::inspect(&mut Cursor::new(flac(1, Some(&[field])))).is_err());
    }
}
#[test]
fn declared_lengths_record_limits_and_export_budgets_are_enforced() {
    let mut huge = b"fLaC".to_vec();
    huge.extend([128, 255, 255, 255]);
    let mut reader = Counted {
        inner: Cursor::new(huge),
        read: 0,
    };
    assert!(metadata::inspect(&mut reader).is_err());
    assert_eq!(reader.read, 8);
    let mut many = flac(1, None);
    many[4] = 0;
    for _ in 0..256 {
        many.extend(block(1, false, &[]));
    }
    assert_eq!(
        metadata::inspect(&mut Cursor::new(many)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let fields = vec!["K=x"; 257];
    assert_eq!(
        metadata::inspect(&mut Cursor::new(flac(1, Some(&fields))))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let field = format!("K={}", "x".repeat(MAX_FIELD));
    assert_eq!(
        metadata::inspect(&mut Cursor::new(flac(1, Some(&[&field]))))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let field = format!("K={}", "x".repeat(16384));
    assert_eq!(
        metadata::inspect(&mut Cursor::new(flac(1, Some(&[&field]))))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let mut artwork = flac(1, None);
    artwork[4] = 0;
    artwork.extend(block(6, true, &[255; 16]));
    let r = metadata::inspect(&mut Cursor::new(artwork)).unwrap();
    assert!(!r.blocks[1].interpreted);
    assert_eq!(r.artwork, "not_decoded_or_exported");
    assert!(
        cube_core::operations::require("audio.metadata.inspect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
#[test]
fn physical_file_caps_and_disabled_picture_comments_remain_bounded() {
    struct Oversize;
    impl Read for Oversize {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            panic!("must reject physical length before read")
        }
    }
    impl Seek for Oversize {
        fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
            Ok(cube_core::media::wave::MAX_FILE_BYTES + 1)
        }
    }
    assert_eq!(
        metadata::inspect(&mut Oversize).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let r = metadata::inspect(&mut Cursor::new(flac(
        1,
        Some(&["METADATA_BLOCK_PICTURE=%%%"]),
    )))
    .unwrap();
    assert!(r.comments.is_empty());
    assert_eq!(r.artwork, "not_decoded_or_exported");
}
