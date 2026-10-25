use cube_core::{
    errors::ErrorCode,
    forensics::{self, boundaries},
};
use std::io::{Cursor, Read, Seek, SeekFrom};
fn segment(out: &mut Vec<u8>, marker: u8, data: &[u8]) {
    out.extend([255, marker]);
    out.extend(((data.len() + 2) as u16).to_be_bytes());
    out.extend(data);
}
fn jpeg() -> Vec<u8> {
    let mut v = vec![255, 216];
    segment(&mut v, 0xe1, b"false end: \xff\xd9");
    let mut q = vec![0];
    q.extend([1; 64]);
    segment(&mut v, 0xdb, &q);
    for id in [0, 16] {
        let mut h = vec![id, 1];
        h.extend([0; 15]);
        h.push(0);
        segment(&mut v, 0xc4, &h);
    }
    segment(&mut v, 0xc0, &[8, 0, 1, 0, 1, 1, 1, 0x11, 0]);
    segment(&mut v, 0xda, &[1, 1, 0, 0, 63, 0]);
    v.extend([0x3f, 255, 217]);
    v
}
fn gif() -> Vec<u8> {
    b"GIF89a\x01\0\x01\0\x80\0\0\0\0\0\xff\xff\xff\x2c\0\0\0\0\x01\0\x01\0\0\x02\x02\x44\x01\0\x3b"
        .to_vec()
}
#[test]
fn additional_boundaries_are_exact_and_truncation_never_looks_complete() {
    for (format, bytes) in [("jpeg", jpeg()), ("gif", gif())] {
        for n in 0..bytes.len() {
            assert!(
                boundaries::inspect(&mut Cursor::new(&bytes[..n])).is_err(),
                "{format} prefix {n}"
            );
        }
        let mut v = bytes.clone();
        v.extend(b"\0PK\x03\x04tail");
        let r = boundaries::inspect(&mut Cursor::new(&v)).unwrap();
        assert_eq!(r.format, format);
        assert_eq!(r.logical_end, bytes.len() as u64);
        assert_eq!(r.trailing_offset, r.logical_end);
        assert_eq!(r.trailing_bytes, 9);
        assert!(r.evidence.contains("not decoded"));
        assert!(r.validated_chunks > 0);
        assert_eq!(
            boundaries::inspect(&mut Cursor::new(bytes))
                .unwrap()
                .trailing_bytes,
            0
        );
    }
}
#[test]
fn gif_lengths_palettes_controls_and_embedded_markers_are_checked() {
    let base = gif();
    let mut comment = base[..19].to_vec();
    comment.extend(b"\x21\xfe\x02\x3b\0\0");
    comment.extend(&base[19..]);
    assert_eq!(
        boundaries::inspect(&mut Cursor::new(&comment))
            .unwrap()
            .logical_end,
        comment.len() as u64
    );
    let mut control = base[..19].to_vec();
    control.extend(b"\x21\xf9\x04\0\0\0\0\0");
    control.extend(&base[19..]);
    assert!(boundaries::inspect(&mut Cursor::new(&control)).is_ok());
    control[22] = 1;
    control[25] = 2;
    assert!(boundaries::inspect(&mut Cursor::new(control)).is_err());
    for (index, value) in [(6, 0), (10, 0), (23, 2), (28, 0x18), (29, 1), (30, 0)] {
        let mut bad = base.clone();
        bad[index] = value;
        assert!(
            boundaries::inspect(&mut Cursor::new(bad)).is_err(),
            "index {index}"
        );
    }
    let mut text = base[..19].to_vec();
    text.extend(b"\x21\x01");
    text.extend(&base[19..]);
    assert_eq!(
        boundaries::inspect(&mut Cursor::new(text))
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedFormat
    );
    let mut many = base[..19].to_vec();
    for _ in 0..10000 {
        many.extend(b"\x21\xfe\0");
    }
    many.extend(&base[19..]);
    assert_eq!(
        boundaries::inspect(&mut Cursor::new(many))
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}
#[test]
fn jpeg_markers_require_tables_frame_and_complete_scans() {
    for bytes in [
        vec![255, 216, 255, 217],
        vec![255, 216, 255, 0],
        vec![255, 216, 255, 0xe0, 0, 1],
        vec![255, 216, 255, 0xc2],
    ] {
        assert!(boundaries::inspect(&mut Cursor::new(bytes)).is_err());
    }
    let base = jpeg();
    for marker in [0xdb, 0xc4, 0xc0, 0xda] {
        let pos = base.windows(2).position(|x| x == [255, marker]).unwrap();
        let len = u16::from_be_bytes([base[pos + 2], base[pos + 3]]) as usize;
        let mut missing = base.clone();
        missing.drain(pos..pos + 2 + len);
        assert!(
            boundaries::inspect(&mut Cursor::new(missing)).is_err(),
            "marker {marker}"
        );
    }
    let scan = base.windows(2).position(|x| x == [255, 0xda]).unwrap();
    for (offset, val) in [(4, 0), (5, 9), (6, 0x44), (8, 62)] {
        let mut bad = base.clone();
        bad[scan + offset] = val;
        assert!(boundaries::inspect(&mut Cursor::new(bad)).is_err());
    }
    let mut empty = base.clone();
    empty.remove(base.len() - 3);
    assert!(boundaries::inspect(&mut Cursor::new(empty)).is_err());
    // Stuffing protects an entropy FF byte from being interpreted as a marker.
    let mut stuffed = base[..base.len() - 3].to_vec();
    stuffed.extend([255, 0, 0x3f, 255, 217]);
    assert_eq!(
        boundaries::inspect(&mut Cursor::new(&stuffed))
            .unwrap()
            .logical_end,
        stuffed.len() as u64
    );
}
#[test]
fn png_boundary_results_and_unsupported_formats_remain_explicit() {
    let image = cube_core::image::Image::from_samples(
        1,
        1,
        cube_core::image::ColorModel::Gray,
        vec![7],
        vec![],
        vec![],
    )
    .unwrap();
    let mut bytes = image.encode_png().unwrap();
    bytes.extend(b"tail");
    let old = forensics::png::appended(&mut Cursor::new(&bytes)).unwrap();
    let new = boundaries::inspect(&mut Cursor::new(&bytes)).unwrap();
    assert_eq!(
        serde_json::to_value(old).unwrap(),
        serde_json::to_value(new).unwrap()
    );
    assert_eq!(
        boundaries::inspect(&mut Cursor::new(b"%PDF-1.7%%EOF"))
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedFormat
    );
}
struct LargeJpeg {
    pos: u64,
}
impl Read for LargeJpeg {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        for byte in out.iter_mut() {
            *byte = match self.pos {
                0 => 255,
                1 => 216,
                n => match (n - 2) % 65537 {
                    0 => 255,
                    1 => 0xe1,
                    2 | 3 => 255,
                    _ => 1,
                },
            };
            self.pos += 1;
        }
        Ok(out.len())
    }
}
impl Seek for LargeJpeg {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        self.pos = match from {
            SeekFrom::Start(n) => n,
            SeekFrom::End(n) => (u64::MAX as i128 + n as i128) as u64,
            SeekFrom::Current(n) => (self.pos as i128 + n as i128) as u64,
        };
        Ok(self.pos)
    }
}
#[test]
fn framing_scan_budget_is_enforced_without_full_file_allocation() {
    let e = boundaries::inspect(&mut LargeJpeg { pos: 0 }).unwrap_err();
    assert_eq!(e.code, ErrorCode::ResourceLimit);
    assert_eq!(e.operation, Some("file.appended.detect"));
}
#[test]
fn mutated_small_frames_never_escape_bounds_or_panic() {
    for seed in [jpeg(), gif()] {
        for i in 0..seed.len() {
            for byte in [0, 255] {
                let mut bytes = seed.clone();
                bytes[i] = byte;
                if let Ok(r) = boundaries::inspect(&mut Cursor::new(&bytes)) {
                    assert!(r.logical_end <= bytes.len() as u64);
                    assert_eq!(r.logical_end + r.trailing_bytes, bytes.len() as u64);
                }
            }
        }
    }
}
