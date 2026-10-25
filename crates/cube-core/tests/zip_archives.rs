use cube_core::{
    errors::ErrorCode,
    forensics::{boundaries, zip},
};
use std::io::Cursor;
fn put16(b: &mut [u8], p: usize, n: u16) {
    b[p..p + 2].copy_from_slice(&n.to_le_bytes());
}
fn put32(b: &mut [u8], p: usize, n: u32) {
    b[p..p + 4].copy_from_slice(&n.to_le_bytes());
}
fn fixture(names: &[&[u8]], descriptor: bool, extra: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    for name in names {
        let offset = bytes.len() as u32;
        let data = b"Cube";
        let crc = u32::from_str_radix(&cube_core::hashing::crc32(data), 16).unwrap();
        let mut local = vec![0; 30];
        local[..4].copy_from_slice(b"PK\x03\x04");
        put16(&mut local, 4, 20);
        put16(&mut local, 6, if descriptor { 8 } else { 0 });
        if !descriptor {
            put32(&mut local, 14, crc);
            put32(&mut local, 18, 4);
            put32(&mut local, 22, 4);
        }
        put16(&mut local, 26, name.len() as u16);
        put16(&mut local, 28, extra.len() as u16);
        bytes.extend(local);
        bytes.extend(*name);
        bytes.extend(extra);
        bytes.extend(data);
        if descriptor {
            bytes.extend(b"PK\x07\x08");
            bytes.extend(crc.to_le_bytes());
            bytes.extend(4u32.to_le_bytes());
            bytes.extend(4u32.to_le_bytes());
        }
        let mut h = vec![0; 46];
        h[..4].copy_from_slice(b"PK\x01\x02");
        put16(&mut h, 4, 20);
        put16(&mut h, 6, 20);
        put16(&mut h, 8, if descriptor { 8 } else { 0 });
        put32(&mut h, 16, crc);
        put32(&mut h, 20, 4);
        put32(&mut h, 24, 4);
        put16(&mut h, 28, name.len() as u16);
        put16(&mut h, 30, extra.len() as u16);
        put32(&mut h, 42, offset);
        central.extend(h);
        central.extend(*name);
        central.extend(extra);
    }
    let start = bytes.len();
    let size = central.len();
    bytes.extend(central);
    let mut end = vec![0; 22];
    end[..4].copy_from_slice(b"PK\x05\x06");
    put16(&mut end, 8, names.len() as u16);
    put16(&mut end, 10, names.len() as u16);
    put32(&mut end, 12, size as u32);
    put32(&mut end, 16, start as u32);
    bytes.extend(end);
    bytes
}
fn check(bytes: &[u8]) -> Result<zip::Archive, cube_core::errors::OperationError> {
    zip::validate(&mut Cursor::new(bytes), 0)
}
fn cd(bytes: &[u8]) -> usize {
    bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap()
}
#[test]
fn embedded_empty_and_multiple_entry_archives_have_exact_bounds() {
    for archive in [
        fixture(&[], false, &[]),
        fixture(&[b"a", b"b"], false, &[]),
        fixture(&[b"a"], true, &[]),
    ] {
        let mut bytes = b"host".to_vec();
        bytes.extend(&archive);
        bytes.extend(b"tail");
        let report = zip::validate(&mut Cursor::new(&bytes), 4).unwrap();
        assert_eq!(report.offset, 4);
        assert_eq!(report.length, archive.len() as u64);
        assert_eq!(report.logical_end, 4 + archive.len() as u64);
        assert_eq!(report.trailing_bytes, 4);
        assert!(!report.payload_verified);
        for entry in report.entries {
            assert!(entry.local_offset >= 4);
            assert!(entry.data_offset + entry.compressed_size <= report.logical_end);
        }
        let boundary = boundaries::inspect(&mut Cursor::new(&archive)).unwrap();
        assert_eq!(boundary.format, "zip");
        assert_eq!(boundary.logical_end, archive.len() as u64);
        for n in 0..archive.len() {
            assert!(check(&archive[..n]).is_err(), "prefix {n}");
        }
        assert!(check(&bytes).is_err());
        assert!(zip::validate(&mut Cursor::new(&bytes), u64::MAX).is_err());
    }
}
#[test]
fn end_signatures_in_comments_are_candidates_not_boundaries() {
    let mut bytes = fixture(&[b"a"], false, &[]);
    let end = bytes.len() - 22;
    let comment = b"fake PK\x05\x06 end";
    put16(&mut bytes, end + 20, comment.len() as u16);
    bytes.extend(comment);
    bytes.extend(b"tail");
    let r = check(&bytes).unwrap();
    assert_eq!(r.trailing_bytes, 4);
    assert_eq!(r.length, bytes.len() as u64 - 4);
    assert!(check(b"PK\x03\x04isolated signature").is_err());
    let mut bomb = fixture(&[b"a"], false, &[]);
    let end = bomb.len() - 22;
    put16(&mut bomb, end + 20, 65 * 4);
    for _ in 0..65 {
        bomb.extend(b"PK\x05\x06");
    }
    assert_eq!(check(&bomb).unwrap_err().code, ErrorCode::ResourceLimit);
}
#[test]
fn central_local_descriptor_and_extent_relationships_must_agree() {
    let valid = fixture(&[b"same", b"same"], false, &[]);
    let c = cd(&valid);
    let end = valid.len() - 22;
    for (pos, val) in [
        (c, 0),
        (c + 16, 0),
        (c + 28, 255),
        (c + 42, 1),
        (0, 0),
        (26, 255),
        (30, 0xff),
        (end + 8, 3),
        (end + 12, 0),
    ] {
        let mut bad = valid.clone();
        bad[pos] = val;
        assert!(check(&bad).is_err(), "byte {pos}");
    }
    let mut overlap = valid.clone();
    put32(&mut overlap, c + 46 + 4 + 42, 0);
    assert!(check(&overlap).unwrap_err().message.contains("overlap"));
    let mut signed = fixture(&[b"a"], true, &[]);
    let descriptor = signed.windows(4).position(|b| b == b"PK\x07\x08").unwrap();
    signed[descriptor + 4] ^= 1;
    assert!(check(&signed).is_err());
    signed[descriptor + 4] ^= 1;
    signed[descriptor] = 0;
    assert_eq!(
        check(&signed).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
}
#[test]
fn unsupported_encryption_zip64_multidisk_and_features_are_explicit() {
    let base = fixture(&[b"a"], false, &[]);
    let c = cd(&base);
    let end = base.len() - 22;
    for (pos, value) in [
        (c + 8, 1),
        (c + 6, 45),
        (end + 4, 1),
        (c + 10, 99),
        (c + 8, 0x10),
    ] {
        let mut bad = base.clone();
        bad[pos] = value;
        assert_eq!(check(&bad).unwrap_err().code, ErrorCode::UnsupportedFormat);
    }
    let mut sentinel = base.clone();
    put32(&mut sentinel, c + 20, u32::MAX);
    assert_eq!(
        check(&sentinel).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    assert_eq!(
        check(&fixture(&[b"a"], false, &[1, 0, 0, 0]))
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedFormat
    );
    assert!(check(&fixture(&[b"a"], false, &[0x99, 0x99, 4, 0])).is_err());
    let mut utf8 = fixture(&[&[255]], false, &[]);
    let c = cd(&utf8);
    put16(&mut utf8, 6, 0x800);
    put16(&mut utf8, c + 8, 0x800);
    assert!(check(&utf8).is_err());
}
#[test]
fn declared_sizes_entries_metadata_and_origin_are_bounded() {
    let base = fixture(&[b"a"], false, &[]);
    let c = cd(&base);
    let end = base.len() - 22;
    let mut huge = base.clone();
    put16(&mut huge, c + 10, 8);
    put32(&mut huge, c + 24, 64 * 1024 * 1024 + 1);
    assert_eq!(check(&huge).unwrap_err().code, ErrorCode::ResourceLimit);
    let mut count = base.clone();
    put16(&mut count, end + 8, 1001);
    put16(&mut count, end + 10, 1001);
    assert_eq!(check(&count).unwrap_err().code, ErrorCode::ResourceLimit);
    let name = vec![b'x'; 65535];
    let names = vec![name.as_slice(); 9];
    assert_eq!(
        check(&fixture(&names, false, &[])).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut oversized = b"PK\x03\x04".to_vec();
    oversized.resize(zip::MAX_ARCHIVE_BYTES as usize + 1, 0);
    assert_eq!(
        check(&oversized).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert!(zip::validate(&mut Cursor::new(&base), base.len() as u64 + 1).is_err());
}
#[test]
fn mutation_probes_preserve_absolute_bounds_without_panics() {
    let seed = fixture(&[b"a"], true, &[]);
    for i in 0..seed.len() {
        for byte in [0, 255] {
            let mut bytes = seed.clone();
            bytes[i] = byte;
            if let Ok(r) = check(&bytes) {
                assert!(r.logical_end <= bytes.len() as u64);
                assert_eq!(r.length, r.logical_end);
                for entry in r.entries {
                    assert!(entry.data_offset + entry.compressed_size <= r.logical_end);
                }
            }
        }
    }
}
#[test]
fn bounded_search_does_not_claim_the_entire_suffix_was_searched() {
    let mut bytes = fixture(&[b"a"], false, &[]);
    bytes.resize(zip::MAX_ARCHIVE_BYTES as usize + 1, 0);
    let r = check(&bytes).unwrap();
    assert_eq!(r.scanned_bytes, zip::MAX_ARCHIVE_BYTES);
    assert!(!r.candidate_scan_complete);
    assert!(!r.payload_verified);
    assert!(r.trailing_bytes > 0);
}
#[test]
fn embedded_structure_errors_keep_absolute_source_offsets() {
    let mut archive = fixture(&[b"a"], false, &[]);
    let end = archive.len() - 22;
    archive.pop();
    let mut bytes = b"prefix!".to_vec();
    bytes.extend(&archive);
    let error = zip::validate(&mut Cursor::new(&bytes), 7).unwrap_err();
    assert_eq!(error.offset, Some(7 + end as u64));
    let archive = fixture(&[b"a"], false, &[1, 0, 0, 0]);
    let extra = cd(&archive) + 46 + 1;
    let mut bytes = b"prefix!".to_vec();
    bytes.extend(&archive);
    let error = zip::validate(&mut Cursor::new(&bytes), 7).unwrap_err();
    assert_eq!(error.offset, Some(7 + extra as u64));
}
