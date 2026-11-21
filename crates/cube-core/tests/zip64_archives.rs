use cube_core::forensics::zip;
use std::io::Cursor;

#[test]
fn python_force_zip64_local_extra_is_accepted() {
    let data = include_bytes!("fixtures/formats/python-force-zip64.zip");
    let report = zip::validate(&mut Cursor::new(data), 0).unwrap();
    assert_eq!(report.entries[0].uncompressed_size, 5);
    assert_eq!(report.entries[0].compressed_size, 5);
    assert!(!report.payload_verified);
    let mut altered = data.to_vec();
    let extra = 30 + b"hello.txt".len();
    altered[extra + 4] ^= 1;
    assert!(zip::validate(&mut Cursor::new(altered), 0).is_err());
}

#[test]
fn an_empty_zip64_eocd_locator_is_exact() {
    let mut data = Vec::new();
    data.extend(b"PK\x06\x06");
    data.extend(44u64.to_le_bytes());
    data.extend(45u16.to_le_bytes());
    data.extend(45u16.to_le_bytes());
    data.extend([0; 40]);
    data.extend(b"PK\x06\x07");
    data.extend([0; 12]);
    data.extend(1u32.to_le_bytes());
    data.extend(b"PK\x05\x06");
    data.extend([0; 4]);
    data.extend([255; 12]);
    data.extend([0; 2]);
    let report = zip::validate(&mut Cursor::new(&data), 0).unwrap();
    assert_eq!(report.logical_end, data.len() as u64);
    assert!(report.entries.is_empty());
}

#[test]
fn zip64_seeks_past_four_gib_without_reading_the_payload() {
    use std::io::{self, Read, Seek, SeekFrom};
    struct Sparse {
        start: Vec<u8>,
        end: Vec<u8>,
        central: u64,
        total: u64,
        pos: u64,
        bytes_read: u64,
    }
    impl Seek for Sparse {
        fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
            self.pos = match from {
                SeekFrom::Start(p) => p,
                SeekFrom::End(p) => self.total.checked_add_signed(p).unwrap(),
                SeekFrom::Current(p) => self.pos.checked_add_signed(p).unwrap(),
            };
            Ok(self.pos)
        }
    }
    impl Read for Sparse {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let available = self.total.saturating_sub(self.pos).min(out.len() as u64) as usize;
            out[..available].fill(0);
            for (start, region) in [
                (0u64, self.start.as_slice()),
                (self.central, self.end.as_slice()),
            ] {
                let before = start.max(self.pos);
                let after = (start + region.len() as u64).min(self.pos + available as u64);
                if after > before {
                    out[(before - self.pos) as usize..(after - self.pos) as usize].copy_from_slice(
                        &region[(before - start) as usize..(after - start) as usize],
                    );
                }
            }
            self.pos += available as u64;
            self.bytes_read += available as u64;
            Ok(available)
        }
    }
    let compressed = (1u64 << 32) + 23;
    let mut start = vec![0; 30];
    start[..4].copy_from_slice(b"PK\x03\x04");
    start[4..6].copy_from_slice(&45u16.to_le_bytes());
    start[8] = 8; // Deflate framing only. No payload integrity claim.
    start[18..26].fill(255);
    start[26] = 1;
    start[28] = 20;
    start.extend([b'a', 1, 0, 16, 0]);
    start.extend(5u64.to_le_bytes());
    start.extend(compressed.to_le_bytes());
    let central = start.len() as u64 + compressed;
    let mut end = vec![0; 46];
    end[..4].copy_from_slice(b"PK\x01\x02");
    end[6..8].copy_from_slice(&45u16.to_le_bytes());
    end[10] = 8;
    end[20..28].fill(255);
    end[28] = 1;
    end[30] = 20;
    end.extend([b'a', 1, 0, 16, 0]);
    end.extend(5u64.to_le_bytes());
    end.extend(compressed.to_le_bytes());
    let bytes = end.len() as u64;
    let eocd = central + bytes;
    end.extend(b"PK\x06\x06");
    end.extend(44u64.to_le_bytes());
    end.extend(45u16.to_le_bytes());
    end.extend(45u16.to_le_bytes());
    end.extend([0; 8]);
    end.extend(1u64.to_le_bytes());
    end.extend(1u64.to_le_bytes());
    end.extend(bytes.to_le_bytes());
    end.extend(central.to_le_bytes());
    end.extend(b"PK\x06\x07");
    end.extend([0; 4]);
    end.extend(eocd.to_le_bytes());
    end.extend(1u32.to_le_bytes());
    end.extend(b"PK\x05\x06");
    end.extend([0; 4]);
    end.extend([255; 12]);
    end.extend([0; 2]);
    let total = central + end.len() as u64;
    let mut source = Sparse {
        start,
        end,
        central,
        total,
        pos: 0,
        bytes_read: 0,
    };
    let result = zip::validate(&mut source, 0).unwrap();
    assert_eq!(result.entries[0].compressed_size, compressed);
    assert_eq!(result.logical_end, total);
    assert!(!result.payload_verified);
    assert!(!result.candidate_scan_complete);
    assert!(source.bytes_read < zip::MAX_ARCHIVE_BYTES + 66_000);
}
