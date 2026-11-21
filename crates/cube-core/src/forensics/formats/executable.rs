//! Inert ELF and PE table facts through the narrow object reader.
use super::*;
use object::{
    Endianness, LittleEndian, elf, pe,
    read::{
        elf::{FileHeader, ProgramHeader, SectionHeader},
        pe::{ImageNtHeaders, ImageOptionalHeader},
    },
};

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Region {
    pub kind: u32,
    pub file: Option<Extent>,
    pub virtual_address: String,
    pub virtual_length: String,
}

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub format: &'static str,
    pub byte_order: &'static str,
    pub architecture: &'static str,
    pub machine: u16,
    /// Exact virtual address (ELF) or relative virtual address (PE).
    pub entry_point: String,
    pub image_base: Option<String>,
    pub main_header: Extent,
    pub stub: Option<Extent>,
    pub program_table: Option<Extent>,
    pub section_table: Option<Extent>,
    pub program_headers: Vec<Region>,
    pub sections: Vec<Region>,
    pub data_directories: Vec<Region>,
    pub unsupported_features: Vec<&'static str>,
}

fn range(data: &[u8], offset: u64, length: u64) -> Result<Extent, OperationError> {
    if offset
        .checked_add(length)
        .is_none_or(|end| end > data.len() as u64)
    {
        return Err(malformed("executable extent exceeds source").offset(offset));
    }
    Ok(Extent { offset, length })
}

fn table(
    data: &[u8],
    offset: u64,
    count: u32,
    stride: usize,
) -> Result<Option<Extent>, OperationError> {
    if count == 0 {
        return Ok(None);
    }
    if offset == 0 {
        return Err(malformed("missing executable table offset"));
    }
    range(data, offset, u64::from(count) * stride as u64).map(Some)
}

fn headers(total: u64, read: u64) -> Result<(), OperationError> {
    if total > 4096 || read > MIB as u64 {
        return Err(limit("executable header or metadata budget exceeded"));
    }
    Ok(())
}

fn address(value: u64) -> String {
    format!("0x{value:x}")
}
fn format_region(
    kind: u32,
    file: Option<Extent>,
    virtual_address: u64,
    virtual_length: u64,
) -> Result<Region, OperationError> {
    virtual_address
        .checked_add(virtual_length)
        .ok_or_else(|| malformed("executable virtual range overflows"))?;
    Ok(Region {
        kind,
        file,
        virtual_address: address(virtual_address),
        virtual_length: address(virtual_length),
    })
}

fn elf_arch(machine: elf::Machine) -> &'static str {
    match machine {
        elf::EM_386 => "x86",
        elf::EM_X86_64 => "x86_64",
        elf::EM_ARM => "arm",
        elf::EM_AARCH64 => "aarch64",
        elf::EM_RISCV => "riscv",
        elf::EM_MIPS => "mips",
        elf::EM_PPC => "powerpc",
        elf::EM_PPC64 => "powerpc64",
        elf::EM_S390 => "s390",
        _ => "unknown",
    }
}

fn elf_report<F: FileHeader>(data: &[u8]) -> Result<Report, OperationError> {
    let invalid = |_| malformed("invalid ELF header or table");
    let h = F::parse(data).map_err(invalid)?;
    let e = h.endian().map_err(invalid)?;
    let main = std::mem::size_of::<F>();
    let pc = h.phnum(e, data).map_err(invalid)?;
    let sc = h.shnum(e, data).map_err(invalid)?;
    let ps = std::mem::size_of::<F::ProgramHeader>();
    let ss = std::mem::size_of::<F::SectionHeader>();
    let ph = h.e_phoff(e).into();
    let sh = h.e_shoff(e).into();
    if h.e_ehsize(e) as usize != main
        || h.e_version(e) != 1
        || (pc == 0 && ph != 0)
        || (sc == 0 && sh != 0)
        || (pc > 0 && (ph < main as u64 || h.e_phentsize(e) as usize != ps))
        || (sc > 0 && (sh < main as u64 || h.e_shentsize(e) as usize != ss))
    {
        return Err(malformed("inconsistent ELF table layout"));
    }
    headers(
        u64::from(pc) + u64::from(sc),
        main as u64 + u64::from(pc) * ps as u64 + u64::from(sc) * ss as u64,
    )?;
    let program_table = table(data, ph, pc, ps)?;
    let section_table = table(data, sh, sc, ss)?;
    if let (Some(p), Some(s)) = (program_table, section_table)
        && p.offset < s.offset + s.length
        && s.offset < p.offset + p.length
    {
        return Err(malformed("overlapping ELF tables"));
    }
    let program_headers = h
        .program_headers(e, data)
        .map_err(invalid)?
        .iter()
        .map(|p| {
            let (offset, size) = p.file_range(e);
            let mem = p.p_memsz(e).into();
            let virtual_address = p.p_vaddr(e).into();
            let kind = p.p_type(e);
            let align: u64 = p.p_align(e).into();
            if (align > 1 && !align.is_power_of_two())
                || (kind == elf::PT_LOAD
                    && (size > mem || (align > 1 && offset % align != virtual_address % align)))
            {
                return Err(malformed("inconsistent ELF segment extents"));
            }
            let file = (size != 0).then(|| range(data, offset, size)).transpose()?;
            format_region(kind.0, file, virtual_address, mem)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sections = h
        .section_headers(e, data)
        .map_err(invalid)?
        .iter()
        .map(|s| {
            let kind = s.sh_type(e);
            let file = if kind == elf::SHT_NULL {
                None
            } else {
                s.file_range(e)
                    .map(|(offset, size)| range(data, offset, size))
                    .transpose()?
            };
            let length = if kind == elf::SHT_NULL {
                0
            } else {
                s.sh_size(e).into()
            };
            format_region(kind.0, file, s.sh_addr(e).into(), length)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let machine = h.e_machine(e);
    let mut unsupported_features = vec!["interpreter_and_dynamic_metadata", "section_payloads"];
    let architecture = elf_arch(machine);
    if architecture == "unknown" {
        unsupported_features.push("machine_name");
    }
    Ok(Report {
        format: if h.is_type_64() { "elf64" } else { "elf32" },
        byte_order: if h.is_little_endian() {
            "little"
        } else {
            "big"
        },
        architecture,
        machine: machine.0,
        entry_point: address(h.e_entry(e).into()),
        image_base: None,
        main_header: Extent::new(0, main),
        stub: None,
        program_table,
        section_table,
        program_headers,
        sections,
        data_directories: vec![],
        unsupported_features,
    })
}

fn pe_arch(machine: u16) -> &'static str {
    match machine {
        0x14c => "x86",
        0x8664 => "x86_64",
        0x1c0 | 0x1c4 => "arm",
        0xaa64 => "aarch64",
        0x200 => "ia64",
        0x5032 | 0x5064 | 0x5128 => "riscv",
        _ => "unknown",
    }
}

fn pe_report<N: ImageNtHeaders>(
    data: &[u8],
    offset: usize,
    section_count: u32,
    optional_size: usize,
    dir_count: u32,
) -> Result<Report, OperationError> {
    let section_start = offset + 24 + optional_size;
    let extent = table(data, section_start as u64, section_count, 40)?;
    let mut start = offset as u64;
    let (nt, dirs) =
        N::parse(data, &mut start).map_err(|_| malformed("invalid PE/COFF headers"))?;
    let h = nt.optional_header();
    let image_size = u64::from(h.size_of_image());
    let header_size = u64::from(h.size_of_headers());
    let end_table = section_start as u64 + u64::from(section_count) * 40;
    if start as usize != section_start
        || end_table > header_size
        || header_size > data.len() as u64
        || header_size > image_size
        || u64::from(h.address_of_entry_point()) >= image_size
        || dirs.len() != dir_count as usize
    {
        return Err(malformed("inconsistent PE header or image boundaries"));
    }
    let mut raw_ranges = Vec::new();
    let mut virtual_ranges = Vec::new();
    let mut sections = Vec::with_capacity(section_count as usize);
    for i in 0..section_count as usize {
        let section = slice(data, section_start + 40 * i, 40)?;
        let raw_offset = u64::from(le32(section, 20));
        let raw_size = u64::from(le32(section, 16));
        let va = u64::from(le32(section, 12));
        let vsize = u64::from(le32(section, 8));
        let file = if raw_size > 0 {
            if raw_offset < header_size {
                return Err(malformed("PE raw section overlaps its headers"));
            }
            let file = range(data, raw_offset, raw_size)?;
            raw_ranges.push((raw_offset, raw_offset + raw_size));
            Some(file)
        } else {
            None
        };
        let mapped = vsize.max(raw_size);
        if mapped > 0 {
            if va < header_size || va + mapped > image_size {
                return Err(malformed("PE section extends beyond virtual image"));
            }
            virtual_ranges.push((va, va + mapped, file));
        }
        sections.push(format_region(le32(section, 36), file, va, vsize)?);
    }
    raw_ranges.sort_unstable();
    virtual_ranges.sort_unstable_by_key(|r| r.0);
    if raw_ranges.windows(2).any(|r| r[0].1 > r[1].0)
        || virtual_ranges.windows(2).any(|r| r[0].1 > r[1].0)
    {
        return Err(malformed("overlapping PE section ranges"));
    }
    let mut data_directories = Vec::with_capacity(dir_count as usize);
    for (index, entry) in dirs.iter().enumerate() {
        let va = u64::from(entry.virtual_address.get(LittleEndian));
        let size = u64::from(entry.size.get(LittleEndian));
        let file = if va == 0 && size == 0 {
            None
        } else if va == 0 || size == 0 {
            return Err(malformed("inconsistent PE data directory range"));
        } else if index == 4 || va + size <= header_size {
            Some(range(data, va, size)?)
        } else {
            let section = virtual_ranges.partition_point(|section| section.0 <= va);
            let Some(&(base, end, file)) =
                section.checked_sub(1).and_then(|i| virtual_ranges.get(i))
            else {
                return Err(malformed("PE data directory has no mapped section"));
            };
            if va + size > end {
                return Err(malformed("PE data directory exceeds its mapped section"));
            }
            file.filter(|r| va - base + size <= r.length)
                .map(|r| range(data, r.offset + va - base, size))
                .transpose()?
        };
        data_directories.push(format_region(index as u32, file, va, size)?);
    }
    let machine = nt.file_header().machine.get(LittleEndian).0;
    let architecture = pe_arch(machine);
    let mut unsupported_features = vec![
        "imports_and_relocations",
        "section_payloads",
        "certificate_validation",
    ];
    if architecture == "unknown" {
        unsupported_features.push("machine_name");
    }
    Ok(Report {
        format: if nt.is_type_64() { "pe32_plus" } else { "pe32" },
        byte_order: "little",
        architecture,
        machine,
        entry_point: address(u64::from(h.address_of_entry_point())),
        image_base: Some(address(h.image_base())),
        main_header: Extent::new(offset, 24 + optional_size),
        stub: Some(Extent::new(64, offset - 64)),
        program_table: None,
        section_table: extent,
        program_headers: vec![],
        sections,
        data_directories,
        unsupported_features,
    })
}

fn pe(data: &[u8]) -> Result<Report, OperationError> {
    let dos = slice(data, 0, 64)?;
    let p = le32(dos, 60) as usize;
    if p < 64 {
        return Err(malformed("PE header overlaps the DOS header"));
    }
    let coff = slice(data, p, 24)?;
    if &coff[..4] != b"PE\0\0" {
        return Err(malformed("invalid PE signature"));
    }
    let count = u32::from(le16(coff, 6));
    let size = usize::from(le16(coff, 20));
    let optional = slice(data, p + 24, size)?;
    let magic = le16(slice(optional, 0, 2)?, 0);
    let base = match magic {
        0x10b => 96,
        0x20b => 112,
        _ => return Err(unsupported("only PE32 or PE32+ images are supported")),
    };
    let dirs = le32(slice(optional, base - 4, 4)?, 0);
    headers(
        u64::from(count) + u64::from(dirs),
        88 + size as u64 + u64::from(count) * 40,
    )?;
    if base + u64::from(dirs) as usize * 8 > size {
        return Err(malformed("PE data directories exceed optional header"));
    }
    if magic == 0x10b {
        pe_report::<pe::ImageNtHeaders32>(data, p, count, size, dirs)
    } else {
        pe_report::<pe::ImageNtHeaders64>(data, p, count, size, dirs)
    }
}

/// Read ELF32/64 or PE32/PE32+ static headers. Keep addresses exact as hex.
pub fn inspect(data: &[u8]) -> Result<Report, OperationError> {
    check_source(data)?;
    let report = if data.starts_with(b"\x7fELF") {
        match data.get(4) {
            Some(1) => elf_report::<elf::FileHeader32<Endianness>>(data),
            Some(2) => elf_report::<elf::FileHeader64<Endianness>>(data),
            _ => Err(unsupported("unsupported ELF class")),
        }
    } else if data.starts_with(b"MZ") {
        pe(data)
    } else {
        Err(unsupported("expected ELF or PE executable headers"))
    }?;
    bounded_report(report, MAX_REPORT)
}
