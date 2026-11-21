//! Classic and ZIP64 records over checked relative offsets.
use super::*;

pub(super) struct Source<'a, R> {
    pub reader: &'a mut R,
    pub base: u64,
    pub length: u64,
    pub budget: Budget,
}

struct End {
    count: u64,
    directory: u64,
    directory_length: u64,
    logical_end: u64,
    zip64: bool,
}
struct Central {
    needed: u16,
    flags: u16,
    method: u16,
    timestamp: [u8; 4],
    crc: u32,
    compressed: u64,
    uncompressed: u64,
    local: u64,
    name: Vec<u8>,
    wide_sizes: bool,
    zip64: bool,
}

impl<R: Read + Seek> Source<'_, R> {
    fn read(&mut self, offset: u64, data: &mut [u8]) -> Result<(), OperationError> {
        self.budget.charge(data.len())?;
        if offset
            .checked_add(data.len() as u64)
            .is_none_or(|n| n > self.length)
        {
            return Err(malformed("ZIP record extends beyond its source").offset(offset));
        }
        self.reader.seek(SeekFrom::Start(self.base + offset))?;
        self.reader.read_exact(data).map_err(|error| {
            malformed("truncated ZIP record")
                .offset(offset)
                .caused_by(error)
        })
    }

    fn record<const N: usize>(&mut self, offset: u64) -> Result<[u8; N], OperationError> {
        let mut out = [0; N];
        self.read(offset, &mut out)?;
        Ok(out)
    }

    fn variable(&mut self, offset: u64, count: usize) -> Result<Vec<u8>, OperationError> {
        let mut data = vec![0; count];
        self.read(offset, &mut data)?;
        Ok(data)
    }

    fn end(&mut self, pos: u64, prefix: u64, tail: bool) -> Result<End, OperationError> {
        let raw = self.record::<22>(pos)?;
        let comment = u64::from(word(&raw, 20));
        let logical_end = (pos + 22)
            .checked_add(comment)
            .ok_or_else(|| malformed("ZIP comment extent overflows"))?;
        if logical_end > self.length || (!tail && logical_end > prefix) {
            return Err(malformed("truncated ZIP end-record comment"));
        }
        self.budget.charge(comment as usize)?;
        let count = [word(&raw, 8), word(&raw, 10)];
        let dsize = dword(&raw, 12);
        let start = dword(&raw, 16);
        let locator = if pos >= 20 {
            let candidate = self.record::<20>(pos - 20)?;
            (&candidate[..4] == b"PK\x06\x07").then_some(candidate)
        } else {
            None
        };
        let disks = [word(&raw, 4), word(&raw, 6)];
        if disks
            .into_iter()
            .any(|n| n != 0 && (n != u16::MAX || locator.is_none()))
        {
            return Err(unsupported("multi-disk ZIP archives are not supported"));
        }
        let (count, directory, directory_length, boundary) = if let Some(locator) = locator {
            if dword(&locator, 4) != 0 || dword(&locator, 16) != 1 {
                return Err(unsupported("multi-disk ZIP64 archives are not supported"));
            }
            let offset = qword(&locator, 8);
            let wide = self.record::<56>(offset)?;
            let length = qword(&wide, 4);
            if &wide[..4] != b"PK\x06\x06"
                || length < 44
                || offset.checked_add(12).and_then(|n| n.checked_add(length)) != Some(pos - 20)
                || word(&wide, 14) < 45
            {
                return Err(malformed("invalid ZIP64 EOCD record or locator"));
            }
            if dword(&wide, 16) != 0
                || dword(&wide, 20) != 0
                || qword(&wide, 24) != qword(&wide, 32)
            {
                return Err(unsupported("multi-disk ZIP64 archives are not supported"));
            }
            let seen = [qword(&wide, 32), qword(&wide, 40), qword(&wide, 48)];
            if count
                .into_iter()
                .any(|n| n != u16::MAX && u64::from(n) != seen[0])
                || (dsize != u32::MAX && u64::from(dsize) != seen[1])
                || (start != u32::MAX && u64::from(start) != seen[2])
            {
                return Err(malformed("ZIP and ZIP64 end records disagree"));
            }
            let extensible = usize::try_from(length - 44)
                .map_err(|_| limit("ZIP64 extensible data exceeds budget"))?;
            if extensible > MAX_METADATA {
                return Err(limit("ZIP64 extensible data exceeds budget"));
            }
            self.budget.charge(extensible)?;
            (seen[0], seen[2], seen[1], offset)
        } else {
            if tail {
                return Err(malformed(
                    "outside-prefix ZIP candidate has no ZIP64 locator",
                ));
            }
            if count.contains(&u16::MAX) || [dsize, start].contains(&u32::MAX) {
                return Err(malformed("ZIP64 end-record fields require a locator"));
            }
            if count[0] != count[1] {
                return Err(unsupported("multi-disk ZIP archives are not supported"));
            }
            (u64::from(count[1]), u64::from(start), u64::from(dsize), pos)
        };
        if count > MAX_ENTRIES as u64 {
            return Err(limit("ZIP archive exceeds 1000 entries"));
        }
        if directory.checked_add(directory_length) != Some(boundary) {
            return Err(malformed("ZIP central directory does not end at EOCD"));
        }
        Ok(End {
            count,
            directory,
            directory_length,
            logical_end,
            zip64: locator.is_some(),
        })
    }

    fn central(&mut self, offset: u64, stop: u64) -> Result<(Central, u64), OperationError> {
        let header = self.record::<46>(offset)?;
        if &header[..4] != b"PK\x01\x02" {
            return Err(malformed("invalid ZIP central-directory record"));
        }
        let (needed, flags, method) = (word(&header, 6), word(&header, 8), word(&header, 10));
        if flags & 0x2041 != 0 {
            return Err(unsupported(
                "encrypted ZIP entries are not supported; payloads are not verified",
            ));
        }
        if flags & !0x080e != 0 || !matches!(method, 0 | 8) || needed > 45 {
            return Err(unsupported(
                "unsupported ZIP flags, compression, version or disk",
            ));
        }
        if needed < 10 || (method == 8 && needed < 20) || (method == 0 && flags & 6 != 0) {
            return Err(malformed("inconsistent ZIP version/method flags"));
        }
        let (name, extra, comment) = (
            word(&header, 28) as usize,
            word(&header, 30) as usize,
            word(&header, 32) as usize,
        );
        let end = offset
            .checked_add(46 + (name + extra + comment) as u64)
            .filter(|&end| end <= stop)
            .ok_or_else(|| malformed("ZIP entry extends beyond central directory"))?;
        let data = self.variable(offset + 46, name + extra + comment)?;
        let (crc, compressed, expanded, local) = (
            dword(&header, 16),
            dword(&header, 20),
            dword(&header, 24),
            dword(&header, 42),
        );
        let disk = word(&header, 34);
        let wanted = [
            expanded == u32::MAX,
            compressed == u32::MAX,
            local == u32::MAX,
            disk == u16::MAX,
        ];
        let values = extras(&data[name..name + extra], wanted)
            .map_err(|error| error.offset(offset + 46 + name as u64))?;
        let [expanded, compressed, local, disk] = [
            values[0].unwrap_or(u64::from(expanded)),
            values[1].unwrap_or(u64::from(compressed)),
            values[2].unwrap_or(u64::from(local)),
            values[3].unwrap_or(u64::from(disk)),
        ];
        if disk != 0 {
            return Err(unsupported("multi-disk ZIP entries are not supported"));
        }
        if wanted.into_iter().any(|v| v) && needed < 45 {
            return Err(malformed("ZIP64 entry requires version 4.5"));
        }
        if method == 0 && compressed != expanded {
            return Err(malformed("stored ZIP entry sizes differ"));
        }
        if flags & 0x800 != 0
            && (std::str::from_utf8(&data[..name]).is_err()
                || std::str::from_utf8(&data[name + extra..]).is_err())
        {
            return Err(malformed(
                "ZIP UTF-8 flag contradicts filename/comment bytes",
            ));
        }
        Ok((
            Central {
                needed,
                flags,
                method,
                timestamp: header[12..16].try_into().expect("ZIP timestamp"),
                crc,
                compressed,
                uncompressed: expanded,
                local,
                name: data[..name].to_vec(),
                wide_sizes: wanted[0] || wanted[1],
                zip64: wanted.into_iter().any(|v| v),
            },
            end,
        ))
    }

    fn local(
        &mut self,
        entry: &Central,
        directory: u64,
    ) -> Result<(u64, u64, bool), OperationError> {
        let p = entry.local;
        let h = self.record::<30>(p)?;
        if entry.needed == 45 && !entry.zip64 && word(&h, 4) != 45 {
            return Err(unsupported(
                "ZIP version 4.5 without matching local ZIP64 fields",
            ));
        }
        if &h[..4] != b"PK\x03\x04"
            || word(&h, 4) != entry.needed
            || word(&h, 6) != entry.flags
            || word(&h, 8) != entry.method
            || h[10..14] != entry.timestamp
        {
            return Err(malformed("ZIP local header contradicts central directory"));
        }
        let (name, extra_len) = (word(&h, 26) as usize, word(&h, 28) as usize);
        let raw = self.variable(p + 30, name + extra_len)?;
        if raw[..name] != entry.name {
            return Err(malformed(
                "ZIP local filename differs from central directory",
            ));
        }
        let (crc, compressed, expanded) = (dword(&h, 14), dword(&h, 18), dword(&h, 22));
        let wide = expanded == u32::MAX || compressed == u32::MAX;
        let ext = extras(
            &raw[name..],
            [expanded == u32::MAX, compressed == u32::MAX, false, false],
        )?;
        if wide && entry.needed < 45 {
            return Err(malformed("ZIP64 local entry requires version 4.5"));
        }
        if entry.needed == 45 && !entry.zip64 && !wide {
            return Err(unsupported("ZIP version 4.5 without ZIP64 fields"));
        }
        let fields = [
            u64::from(crc),
            ext[1].unwrap_or(u64::from(compressed)),
            ext[0].unwrap_or(u64::from(expanded)),
        ];
        let expected = [u64::from(entry.crc), entry.compressed, entry.uncompressed];
        let start = p + 30 + (name + extra_len) as u64;
        let stop = start
            .checked_add(entry.compressed)
            .filter(|&n| n <= directory)
            .ok_or_else(|| {
                malformed("ZIP entry overlaps central directory or its extent overflows")
            })?;
        let end = if entry.flags & 8 != 0 {
            if fields
                .into_iter()
                .zip(expected)
                .any(|(a, b)| a != 0 && a != b)
            {
                return Err(malformed(
                    "ZIP local descriptor-mode sizes/CRC contradict directory",
                ));
            }
            let descriptor = if wide || entry.wide_sizes {
                self.record::<24>(stop)?.to_vec()
            } else {
                self.record::<16>(stop)?.to_vec()
            };
            if &descriptor[..4] != b"PK\x07\x08" {
                return Err(unsupported(
                    "only signed ZIP data descriptors are supported",
                ));
            }
            let sizes = if descriptor.len() == 24 {
                [qword(&descriptor, 8), qword(&descriptor, 16)]
            } else {
                [
                    u64::from(dword(&descriptor, 8)),
                    u64::from(dword(&descriptor, 12)),
                ]
            };
            if dword(&descriptor, 4) != entry.crc || sizes != expected[1..] {
                return Err(malformed("ZIP data descriptor contradicts directory"));
            }
            stop + descriptor.len() as u64
        } else {
            if fields != expected {
                return Err(malformed("ZIP local sizes/CRC contradict directory"));
            }
            stop
        };
        if end > directory {
            return Err(malformed("ZIP entry overlaps central directory"));
        }
        Ok((start, end, wide))
    }

    pub fn directory(
        &mut self,
        pos: u64,
        scanned: u64,
        tail: bool,
    ) -> Result<Archive, OperationError> {
        let end = self.end(pos, scanned, tail)?;
        let mut zip64 = end.zip64;
        let mut entries = Vec::with_capacity(end.count as usize);
        let mut extents = Vec::with_capacity(end.count as usize);
        let mut declared = 0;
        let mut p = end.directory;
        let stop = end.directory + end.directory_length;
        for _ in 0..end.count {
            let (record, next) = self.central(p, stop)?;
            p = next;
            declared = u64::checked_add(declared, record.uncompressed)
                .ok_or_else(|| limit("ZIP declared-size overflow"))?;
            if declared > MAX_DECLARED_BYTES {
                return Err(limit("ZIP declared output exceeds 64 MiB"));
            }
            let (data, local_end, local_wide) = self.local(&record, end.directory)?;
            zip64 |= record.zip64 || local_wide;
            extents.push((record.local, local_end));
            entries.push(Entry {
                local_offset: self.base + record.local,
                data_offset: self.base + data,
                compressed_size: record.compressed,
                uncompressed_size: record.uncompressed,
                crc32: record.crc,
                method: record.method,
                flags: record.flags,
                name_hex: crate::encoding::hex_encode(&record.name),
            });
        }
        if p != stop {
            return Err(malformed("ZIP directory size/count mismatch"));
        }
        extents.sort_unstable();
        let mut previous = 0;
        for (start, finish) in extents {
            if start != previous {
                return Err(unsupported(
                    "ZIP local records overlap, have gaps or use another offset convention",
                ));
            }
            previous = finish;
        }
        if previous != end.directory {
            return Err(unsupported(
                "ZIP local records do not reach central directory contiguously",
            ));
        }
        let evidence = if zip64 {
            "Single-disk ZIP64 central/local/end relationships and bounded extents; compressed payloads, CRC integrity and authenticity are not verified"
        } else {
            "Classic single-disk ZIP central/local/end relationships and bounded extents; compressed payloads, CRC integrity and authenticity are not verified"
        };
        Ok(Archive {
            format: "zip",
            offset: self.base,
            length: end.logical_end,
            logical_end: self.base + end.logical_end,
            trailing_bytes: self.length - end.logical_end,
            entries,
            payload_verified: false,
            scanned_bytes: scanned,
            candidate_scan_complete: scanned == self.length,
            evidence,
        })
    }
}

fn extras(mut data: &[u8], wanted: [bool; 4]) -> Result<[Option<u64>; 4], OperationError> {
    let mut seen = false;
    let expected = wanted
        .into_iter()
        .zip([8, 8, 8, 4])
        .filter_map(|(needed, width)| needed.then_some(width))
        .sum::<usize>();
    let invalid = || malformed("missing, duplicate, conflicting or truncated ZIP64 extra");
    let mut output = [None; 4];
    while !data.is_empty() {
        let head = data
            .get(..4)
            .ok_or_else(|| malformed("truncated ZIP extra header"))?;
        let length = word(head, 2) as usize;
        let (field, next) = data[4..]
            .split_at_checked(length)
            .ok_or_else(|| malformed("truncated ZIP extra"))?;
        if word(head, 0) == 1 {
            if seen || expected == 0 || length != expected {
                return Err(invalid());
            }
            seen = true;
            let mut p = 0;
            for (i, needed) in wanted.into_iter().enumerate() {
                if needed {
                    output[i] = Some(if i == 3 {
                        u64::from(dword(field, p))
                    } else {
                        qword(field, p)
                    });
                    p += if i == 3 { 4 } else { 8 };
                }
            }
        }
        data = next;
    }
    if expected != 0 && !seen {
        return Err(invalid());
    }
    Ok(output)
}
