//! RFC 3533 Ogg framing, per-stream packet fragments and page CRCs.
use super::*;
use std::collections::BTreeMap;

const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::Algorithm {
    width: 32,
    poly: 0x04c1_1db7,
    init: 0,
    refin: false,
    refout: false,
    xorout: 0,
    check: 0x89a1_897f,
    residue: 0,
});

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Fragment {
    pub packet: u32,
    pub range: Extent,
    pub complete: bool,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Page {
    pub range: Extent,
    pub serial: u32,
    pub sequence: u32,
    pub flags: u8,
    /// Canonical decimal so the entire unsigned field survives JS transport.
    pub granule_position: String,
    pub checksum: u32,
    pub lacing: Vec<u8>,
    pub fragments: Vec<Fragment>,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Stream {
    pub serial: u32,
    pub pages: u32,
    pub packets: u32,
    pub ended: bool,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub pages: Vec<Page>,
    pub streams: Vec<Stream>,
    pub checksum_verified: bool,
}

#[derive(Default)]
struct State {
    next_page: u32,
    packets: u32,
    continued: bool,
    ended: bool,
}

fn fragments(start: usize, lacing: &[u8], state: &mut State) -> Vec<Fragment> {
    let mut output = Vec::new();
    let mut offset = start;
    let mut length = 0;
    for &lace in lacing {
        length += lace as usize;
        if lace != 255 {
            output.push(Fragment {
                packet: state.packets,
                range: Extent::new(offset, length),
                complete: true,
            });
            state.packets += 1;
            offset += length;
            length = 0;
        }
    }
    if length > 0 {
        output.push(Fragment {
            packet: state.packets,
            range: Extent::new(offset, length),
            complete: false,
        });
    }
    if !lacing.is_empty() {
        state.continued = length > 0;
    }
    output
}

/// Parse complete Ogg pages and verify every page CRC before returning a report.
pub fn inspect(data: &[u8]) -> Result<Report, OperationError> {
    check_source(data)?;
    if data.len() > 16 * MIB {
        return Err(limit("Ogg CRC work exceeds 16 Mi units"));
    }
    let mut start = 0;
    let mut pages = Vec::new();
    let mut streams: BTreeMap<u32, State> = BTreeMap::new();
    let mut retained = 0;
    while start < data.len() {
        if pages.len() == 10_000 {
            return Err(limit("Ogg exceeds 10000 pages"));
        }
        let head = slice(data, start, 27)?;
        if &head[..4] != b"OggS" || head[4] != 0 {
            return Err(unsupported("expected Ogg version 0 page"));
        }
        let flags = head[5];
        if flags & !7 != 0 {
            return Err(malformed("invalid Ogg page flags"));
        }
        let serial = le32(head, 14);
        let sequence = le32(head, 18);
        let checksum = le32(head, 22);
        let lace = slice(data, start + 27, head[26] as usize)?;
        let payload = lace.iter().map(|&v| v as usize).sum();
        let payload_offset = start + 27 + lace.len();
        let body = slice(data, payload_offset, payload)?;
        let mut crc = CRC.digest();
        crc.update(&head[..22]);
        crc.update(&[0; 4]);
        crc.update(&head[26..]);
        crc.update(lace);
        crc.update(body);
        if crc.finalize() != checksum {
            return Err(malformed("Ogg page CRC mismatch").offset(start as u64));
        }
        if !streams.contains_key(&serial) {
            if streams.len() == 128 {
                return Err(limit("Ogg exceeds 128 streams"));
            }
            if flags & 2 == 0 {
                return Err(malformed("Ogg stream starts without BOS"));
            }
            streams.insert(serial, State::default());
        }
        let state = streams.get_mut(&serial).expect("registered stream");
        if state.ended
            || sequence != state.next_page
            || (flags & 2 != 0 && sequence != 0)
            || (flags & 1 != 0) != state.continued
        {
            return Err(
                malformed("Ogg stream sequence or continuation mismatch").offset(start as u64)
            );
        }
        let page_fragments = fragments(payload_offset, lace, state);
        let granule = le64(head, 6);
        state.next_page += 1;
        state.ended = flags & 4 != 0;
        if (state.ended && state.continued)
            || (!lace.iter().any(|&n| n < 255) && granule != u64::MAX)
        {
            return Err(malformed(
                "inconsistent Ogg packet boundary or granule position",
            ));
        }
        retained += 64 + lace.len() + 32 * page_fragments.len();
        if retained > MIB {
            return Err(limit("Ogg retained metadata exceeds 1 MiB"));
        }
        let length = payload_offset - start + payload;
        pages.push(Page {
            range: Extent::new(start, length),
            serial,
            sequence,
            flags,
            granule_position: granule.to_string(),
            checksum,
            lacing: lace.to_vec(),
            fragments: page_fragments,
        });
        start += length;
    }
    if pages.is_empty() || streams.values().any(|state| state.continued) {
        return Err(malformed("empty Ogg source or unterminated Ogg packet"));
    }
    let streams = streams
        .into_iter()
        .map(|(serial, state)| Stream {
            serial,
            pages: state.next_page,
            packets: state.packets,
            ended: state.ended,
        })
        .collect();
    bounded_report(
        Report {
            pages,
            streams,
            checksum_verified: true,
        },
        MAX_REPORT,
    )
}
