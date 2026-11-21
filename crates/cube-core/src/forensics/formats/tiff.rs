//! Classic TIFF IFD and SubIFD graph validation with explicit field ranges.
use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Little,
    Big,
}

impl Order {
    fn short(self, data: &[u8], offset: usize) -> u16 {
        let bytes = data[offset..offset + 2]
            .try_into()
            .expect("checked TIFF field");
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }
    fn word(self, data: &[u8], offset: usize) -> u32 {
        let bytes = data[offset..offset + 4]
            .try_into()
            .expect("checked TIFF field");
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Field {
    pub tag: u16,
    pub field_type: u16,
    pub value_count: u32,
    pub entry_range: Extent,
    pub value_range: Option<Extent>,
    pub inline_hex: Option<String>,
    pub offset_value_hex: Option<String>,
    pub unsupported_type: bool,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Directory {
    pub range: Extent,
    pub entries: Vec<Field>,
    pub next: Option<u32>,
    pub sub_ifds: Vec<u32>,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub byte_order: Order,
    pub first: Option<u32>,
    pub directories: Vec<Directory>,
    pub payload_verified: bool,
}

struct Graph<'a> {
    data: &'a [u8],
    order: Order,
    visited: BTreeMap<u32, Option<usize>>,
    directories: Vec<Directory>,
    entries: usize,
    retained: usize,
}

impl Graph<'_> {
    fn entry(
        &mut self,
        raw: &[u8],
        offset: usize,
        links: &mut Vec<u32>,
    ) -> Result<Field, OperationError> {
        let (tag, field_type, count) = (
            self.order.short(raw, 0),
            self.order.short(raw, 2),
            self.order.word(raw, 4),
        );
        let size = match field_type {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 | 13 => 4,
            5 | 10 | 12 => 8,
            _ => 0,
        };
        let mut field = Field {
            tag,
            field_type,
            value_count: count,
            entry_range: Extent::new(offset, 12),
            value_range: None,
            inline_hex: None,
            offset_value_hex: None,
            unsupported_type: size == 0,
        };
        if size == 0 {
            self.retain(4)?;
            field.inline_hex = Some(crate::encoding::hex_encode(&raw[8..]));
            return Ok(field);
        }
        let bytes = usize::try_from(u64::from(count) * size)
            .map_err(|_| malformed("TIFF field extent overflows"))?;
        let position = if bytes <= 4 {
            offset + 8
        } else {
            self.order.word(raw, 8) as usize
        };
        if bytes > 4 && (position < 8 || !position.is_multiple_of(2)) {
            return Err(malformed("invalid TIFF value offset").offset(position as u64));
        }
        let data = slice(self.data, position, bytes)?;
        self.retain(bytes)?;
        field.value_range = Some(Extent::new(position, bytes));
        if bytes <= 4 {
            field.inline_hex = Some(crate::encoding::hex_encode(data));
        } else {
            field.offset_value_hex = Some(crate::encoding::hex_encode(data));
        }
        if tag == 330 {
            if !matches!(field_type, 4 | 13) {
                return Err(malformed("TIFF SubIFD requires LONG or IFD offsets"));
            }
            if links.len() + count as usize > 1024 {
                return Err(limit("TIFF SubIFD link limit exceeded"));
            }
            for i in 0..count as usize {
                let link = self.order.word(data, i * 4);
                if link == 0 {
                    return Err(malformed("zero TIFF SubIFD offset"));
                }
                links.push(link);
            }
        }
        Ok(field)
    }

    fn retain(&mut self, count: usize) -> Result<(), OperationError> {
        self.retained += count;
        if self.retained > MIB {
            return Err(limit("TIFF retained payload exceeds 1 MiB"));
        }
        Ok(())
    }

    fn directory(&mut self, offset: u32) -> Result<Directory, OperationError> {
        let p = offset as usize;
        if p < 8 || !p.is_multiple_of(2) {
            return Err(malformed("invalid TIFF directory offset").offset(u64::from(offset)));
        }
        let count = self.order.short(slice(self.data, p, 2)?, 0) as usize;
        self.entries += count;
        if self.entries > 10_000 {
            return Err(limit("TIFF exceeds 10000 directory entries"));
        }
        let table = slice(self.data, p + 2, count * 12 + 4)?;
        let mut sub_ifds = Vec::new();
        let entries = table[..count * 12]
            .as_chunks::<12>()
            .0
            .iter()
            .enumerate()
            .map(|(i, raw)| self.entry(raw, p + 2 + 12 * i, &mut sub_ifds))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Directory {
            range: Extent::new(p, 2 + table.len()),
            entries,
            next: nonzero(self.order.word(table, count * 12)),
            sub_ifds,
        })
    }

    fn visit(&mut self, first: u32, depth: usize) -> Result<usize, OperationError> {
        if depth > 8 {
            return Err(limit("TIFF directory depth exceeds eight"));
        }
        let mut next = Some(first);
        let mut path = Vec::new();
        let mut height = 0;
        while let Some(offset) = next {
            if let Some(previous) = self.visited.get(&offset) {
                height = previous.ok_or_else(|| malformed("cyclic TIFF directory graph"))?;
                if depth + height - 1 > 8 {
                    return Err(limit("TIFF directory depth exceeds eight"));
                }
                break;
            }
            if self.visited.len() == 1024 {
                return Err(limit("TIFF exceeds 1024 directories"));
            }
            self.visited.insert(offset, None);
            let mut directory = self.directory(offset)?;
            next = directory.next;
            let sub_ifds = std::mem::take(&mut directory.sub_ifds);
            let position = self.directories.len();
            self.directories.push(directory);
            let mut nodes = 1;
            for &child in &sub_ifds {
                nodes = nodes.max(self.visit(child, depth + 1)? + 1);
            }
            self.directories[position].sub_ifds = sub_ifds;
            path.push((offset, nodes));
        }
        for (offset, nodes) in path.into_iter().rev() {
            height = height.max(nodes);
            self.visited.insert(offset, Some(height));
        }
        Ok(height)
    }
}

fn nonzero(value: u32) -> Option<u32> {
    (value != 0).then_some(value)
}

/// Inspect a classic TIFF header and reachable directories in file order.
pub fn inspect(data: &[u8]) -> Result<Report, OperationError> {
    check_source(data)?;
    let header = slice(data, 0, 8)?;
    let order = match &header[..2] {
        b"II" => Order::Little,
        b"MM" => Order::Big,
        _ => return Err(unsupported("expected classic TIFF byte order")),
    };
    if order.short(header, 2) != 42 {
        return Err(unsupported("only classic TIFF magic 42 is supported"));
    }
    let first = nonzero(order.word(header, 4));
    let mut graph = Graph {
        data,
        order,
        visited: BTreeMap::new(),
        directories: Vec::new(),
        entries: 0,
        retained: 0,
    };
    if let Some(root) = first {
        graph.visit(root, 1)?;
    }
    bounded_report(
        Report {
            byte_order: order,
            first,
            directories: graph.directories,
            payload_verified: false,
        },
        MAX_REPORT,
    )
}
