//! Allocation-free canonical framing before the maintained parser or unwrap.
use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
pub(super) fn check(input: &[u8]) -> Result<(), OperationError> {
    if !input.starts_with(b"age-encryption.org/v1\n") {
        return Err(unsupported());
    }
    let mut cursor = b"age-encryption.org/v1\n".len();
    let mut stanzas = 0;
    let mut x25519 = false;
    loop {
        let header_line = line(input, &mut cursor)?;
        if let Some(mac) = header_line.strip_prefix(b"--- ") {
            if stanzas == 0 {
                return Err(malformed());
            }
            if canonical(mac)? != 32 {
                return Err(malformed());
            }
            if !x25519 {
                return Err(unsupported());
            }
            if input.len() - cursor < 32 {
                return Err(malformed());
            }
            return Ok(());
        }
        if stanzas == MAX_STANZAS {
            return Err(limit());
        }
        let arguments = header_line.strip_prefix(b"-> ").ok_or_else(malformed)?;
        if arguments
            .split(|&b| b == b' ')
            .any(|a| a.is_empty() || a.iter().any(|b| !b.is_ascii_graphic()))
        {
            return Err(malformed());
        }
        let tag = arguments
            .split(|&b| b == b' ')
            .next()
            .ok_or_else(malformed)?;
        if tag == b"scrypt" {
            return Err(unsupported());
        }
        if tag == b"X25519" {
            let arg = arguments.strip_prefix(b"X25519 ").ok_or_else(malformed)?;
            if canonical(arg)? != 32 {
                return Err(malformed());
            }
            let body = line(input, &mut cursor)?;
            if canonical(body)? != 32 {
                return Err(malformed());
            }
            x25519 = true;
        } else {
            // Opaque recipient extensions are authenticated as part of the header but
            // never dispatched to any other identity, KDF, plugin or external process.
            loop {
                let body = line(input, &mut cursor)?;
                canonical(body)?;
                if body.len() < 64 {
                    break;
                }
            }
        }
        stanzas += 1;
    }
}
fn line<'a>(input: &'a [u8], cursor: &mut usize) -> Result<&'a [u8], OperationError> {
    let remaining = &input[*cursor..input.len().min(MAX_HEADER)];
    let end = remaining.iter().position(|&b| b == b'\n').ok_or_else(|| {
        if input.len() >= MAX_HEADER {
            limit()
        } else {
            malformed()
        }
    })?;
    *cursor += end + 1;
    let line = &remaining[..end];
    if !line.is_ascii() || line.contains(&b'\r') {
        return Err(malformed());
    }
    Ok(line)
}
fn canonical(token: &[u8]) -> Result<usize, OperationError> {
    if token.len() > 64 {
        return Err(malformed());
    }
    let mut decoded = [0; 48];
    let length = STANDARD_NO_PAD
        .decode_slice(token, &mut decoded)
        .map_err(|_| malformed())?;
    let mut encoded = [0; 64];
    let size = STANDARD_NO_PAD
        .encode_slice(&decoded[..length], &mut encoded)
        .map_err(|_| malformed())?;
    if &encoded[..size] != token {
        return Err(malformed());
    }
    Ok(length)
}
