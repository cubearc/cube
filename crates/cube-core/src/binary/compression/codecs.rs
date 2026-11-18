//! Backend adapters with exact end-of-stream and preflight checks.
use super::*;
use flate2::{Compression, Decompress, FlushDecompress, Status};

pub(super) fn encode(
    mut data: &[u8],
    profile: Profile,
    level: u8,
    output: &mut Output,
) -> io::Result<()> {
    let level = Compression::new(u32::from(level));
    match profile {
        Profile::Deflate => {
            let mut encoder = flate2::write::DeflateEncoder::new(output, level);
            encoder.write_all(data)?;
            encoder.finish()?;
        }
        Profile::Zlib => {
            let mut encoder = flate2::write::ZlibEncoder::new(output, level);
            encoder.write_all(data)?;
            encoder.finish()?;
        }
        Profile::Gzip => {
            let mut encoder = flate2::GzBuilder::new()
                .mtime(0)
                .operating_system(255)
                .write(output, level);
            encoder.write_all(data)?;
            encoder.finish()?;
        }
        Profile::Brotli => {
            let settings = brotli::enc::BrotliEncoderParams {
                quality: 4,
                lgwin: 20,
                lgblock: 16,
                ..Default::default()
            };
            brotli::BrotliCompress(&mut data, output, &settings)?;
        }
        Profile::Zstd => {
            ruzstd::encoding::compress(data, output, ruzstd::encoding::CompressionLevel::Fastest)
        }
        Profile::LzmaAlone => {
            let mut settings = lzma_rust2::LzmaOptions::with_preset(0);
            settings.dict_size = 1 << 20;
            let mut encoder = lzma_rust2::LzmaWriter::new_use_header(output, &settings, None)?;
            encoder.write_all(data)?;
            encoder.finish()?;
        }
    }
    Ok(())
}

pub(super) fn decode(
    data: &[u8],
    profile: Profile,
    output: &mut Output,
) -> Result<(), OperationError> {
    match profile {
        Profile::Deflate | Profile::Zlib => deflate(data, matches!(profile, Profile::Zlib), output),
        Profile::Gzip => {
            let mut decoder = flate2::bufread::GzDecoder::new(data);
            read_output(&mut decoder, output)?;
            if !decoder.into_inner().is_empty() {
                return Err(invalid());
            }
            Ok(())
        }
        Profile::Brotli => brotli(data, output),
        Profile::Zstd => zstd(data, output),
        Profile::LzmaAlone => lzma(data, output),
    }
}

fn deflate(data: &[u8], zlib: bool, output: &mut Output) -> Result<(), OperationError> {
    let mut decoder = Decompress::new(zlib);
    let mut chunk = [0; 8192];
    loop {
        let start = decoder.total_in() as usize;
        let before = decoder.total_out() as usize;
        let capacity = chunk.len().min(output.max - output.data.len() + 1);
        let status = decoder
            .decompress(
                &data[start..],
                &mut chunk[..capacity],
                FlushDecompress::None,
            )
            .map_err(|_| invalid())?;
        let size = decoder.total_out() as usize - before;
        if !output.append(&chunk[..size]) {
            return Err(limit(DECODE));
        }
        if status == Status::StreamEnd {
            return if decoder.total_in() as usize == data.len() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if start == decoder.total_in() as usize && size == 0 {
            return Err(invalid());
        }
    }
}

fn brotli(data: &[u8], output: &mut Output) -> Result<(), OperationError> {
    use brotli::{BrotliDecompressStream, BrotliResult, BrotliState, HeapAlloc};
    // RFC 7932 WBITS occupies one, four or seven bits, least-significant bit first.
    // Standard streams use up to 24 window bits. 0b0010001 begins the nonstandard
    // large-window extension, which is also disabled in the decoder state.
    if data.first().is_none_or(|first| first & 0x7f == 0x11) {
        return Err(invalid());
    }
    let mut state = BrotliState::new_strict(
        HeapAlloc::default(),
        HeapAlloc::default(),
        HeapAlloc::default(),
    );
    let mut available = data.len();
    let mut offset = 0;
    let mut written = 0;
    let mut chunk = [0; 8192];
    loop {
        let mut capacity = chunk.len().min(output.max - output.data.len() + 1);
        let mut size = 0;
        let before = offset;
        let result = BrotliDecompressStream(
            &mut available,
            &mut offset,
            data,
            &mut capacity,
            &mut size,
            &mut chunk,
            &mut written,
            &mut state,
        );
        if !output.append(&chunk[..size]) {
            return Err(limit(DECODE));
        }
        match result {
            BrotliResult::ResultSuccess if offset == data.len() => return Ok(()),
            BrotliResult::NeedsMoreOutput if size > 0 || before != offset => (),
            _ => return Err(invalid()),
        }
    }
}

fn zstd_header(data: &[u8], bound: usize) -> Result<Option<u64>, OperationError> {
    let [0x28, 0xb5, 0x2f, 0xfd, desc, tail @ ..] = data else {
        return Err(invalid());
    };
    if desc & 0x0b != 0 {
        return Err(invalid());
    }
    let single = desc & 0x20 != 0;
    let tail = if single {
        tail
    } else {
        let (&window, tail) = tail.split_first().ok_or_else(invalid)?;
        let base = 1u64 << (10 + (window >> 3));
        if base + (base / 8) * u64::from(window & 7) > MAX_BYTES as u64 {
            return Err(limit(DECODE));
        }
        tail
    };
    let size = match desc >> 6 {
        0 => usize::from(single),
        flag => 1 << flag,
    };
    if size == 0 {
        return Ok(None);
    }
    let field = tail.get(..size).ok_or_else(invalid)?;
    let mut fcs = [0; 8];
    fcs[..size].copy_from_slice(field);
    let bytes = u64::from_le_bytes(fcs) + if size == 2 { 256 } else { 0 };
    if bytes > bound as u64 {
        return Err(limit(DECODE));
    }
    Ok(Some(bytes))
}

fn zstd(data: &[u8], output: &mut Output) -> Result<(), OperationError> {
    let declared = zstd_header(data, output.max)?;
    let mut decoder =
        ruzstd::decoding::StreamingDecoder::new_with_max_window_size(data, MAX_BYTES as u64)
            .map_err(|_| invalid())?;
    read_output(&mut decoder, output)?;
    let (trailing, engine) = decoder.into_parts();
    if !trailing.is_empty()
        || !engine.is_finished()
        || declared.is_some_and(|length| length != output.data.len() as u64)
        || engine
            .get_checksum_from_data()
            .is_some_and(|sum| engine.get_calculated_checksum() != Some(sum))
    {
        return Err(invalid());
    }
    Ok(())
}

fn lzma(data: &[u8], output: &mut Output) -> Result<(), OperationError> {
    let [_properties, dictionary @ ..] = data else {
        return Err(invalid());
    };
    let dictionary = u32::from_le_bytes(*dictionary.first_chunk::<4>().ok_or_else(invalid)?);
    let size_bytes = data.get(5..13).ok_or_else(invalid)?;
    let declared = u64::from_le_bytes(size_bytes.try_into().expect("eight header bytes"));
    if dictionary > MAX_BYTES as u32 || (declared != u64::MAX && declared > output.max as u64) {
        return Err(limit(DECODE));
    }
    let mut decoder =
        lzma_rust2::LzmaReader::new_mem_limit(data, 20 * 1024, None).map_err(|_| invalid())?;
    decoder.set_relaxed_end_condition(false);
    read_output(&mut decoder, output)?;
    let (remaining, buffered) = decoder.into_parts();
    if !remaining.is_empty()
        || !buffered.is_empty()
        || (declared != u64::MAX && declared != output.data.len() as u64)
    {
        return Err(invalid());
    }
    Ok(())
}
