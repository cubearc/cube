//! Rectified standard PDF417. Geometry bounds the matrix before RXing's
//! codeword lookup, error correction and compaction parser can run.
use rxing::{
    common::BitMatrix,
    pdf417::{
        PDF417RXingResultMetadata,
        decoder::{decoded_bit_stream_parser, ec},
        pdf_417_common::{CODEWORD_TABLE, NUMBER_OF_CODEWORDS, SYMBOL_TABLE},
    },
};

use super::{Crop, Status, blank, limit};
use crate::errors::OperationError;

const START: u32 = 0x1fea8;
const STOP: u32 = 0x3fa29;

fn codeword(symbol: u32, cluster: usize) -> Option<u32> {
    let index = SYMBOL_TABLE.binary_search(&symbol).ok()?;
    let mapped = CODEWORD_TABLE[index] - 1;
    (mapped as usize / NUMBER_OF_CODEWORDS as usize == cluster)
        .then_some(mapped % NUMBER_OF_CODEWORDS)
}

fn row(image: &BitMatrix, x: u32, y: u32, scale: u32, columns: usize) -> Option<Vec<u32>> {
    let mut sample_x = x + scale / 2;
    let mut read = |len| {
        (0..len).fold(0, |word, _| {
            let next = (word << 1) | u32::from(image.get(sample_x, y));
            sample_x += scale;
            next
        })
    };
    if read(17) != START {
        return None;
    }
    let codes: Vec<_> = (0..columns + 2).map(|_| read(17)).collect();
    (read(18) == STOP).then_some(codes)
}

fn indicators(index: usize, rows: usize, columns: usize, level: u32) -> [u32; 2] {
    let base = 30 * (index as u32 / 3);
    let (upper, lower) = ((rows as u32 - 1) / 3, (rows as u32 - 1) % 3);
    let fields = [upper, level * 3 + lower, columns as u32 - 1];
    [base + fields[index % 3], base + fields[(index + 2) % 3]]
}

pub(super) fn decode(
    image: &BitMatrix,
    [x, y, width, height]: [u32; 4],
    remaining: &mut usize,
) -> Result<Crop, OperationError> {
    let first_run = (x..x + width).take_while(|&x| image.get(x, y)).count() as u32;
    if first_run == 0 || !first_run.is_multiple_of(8) {
        return Ok(blank(Status::NoDetection, vec![]));
    }
    let scale = first_run / 8;
    let modules = width / scale;
    if !width.is_multiple_of(scale) || modules < 86 || !(modules - 18).is_multiple_of(17) {
        return Ok(blank(Status::NoDetection, vec![]));
    }
    let columns = ((modules - 18) / 17 - 3) as usize;
    if !(1..=30).contains(&columns) {
        return Ok(blank(Status::NoDetection, vec![]));
    }
    let mut rows = Vec::new();
    for line in y..y + height {
        let Some(bits) = row(image, x, line, scale, columns) else {
            let status = if rows.is_empty() {
                Status::NoDetection
            } else {
                Status::DetectedUndecodable
            };
            return Ok(blank(status, vec![]));
        };
        if rows.last() == Some(&bits) {
            continue;
        }
        if rows.len() == 90 {
            return Ok(blank(Status::DetectedUndecodable, vec![]));
        }
        rows.push(bits);
    }
    let mut result = blank(
        Status::DetectedUndecodable,
        super::rectangle([x, y, width, height]),
    );
    let count = rows.len() * columns;
    if rows.len() < 3 || count > 928 {
        return Ok(result);
    }
    let Some(level_code) = codeword(rows[1][0], 1) else {
        return Ok(result);
    };
    let level = (level_code % 30) / 3;
    let parity = 1usize << (level + 1);
    if parity > 512 || count < parity + 2 {
        return Ok(result);
    }
    let mut codes = Vec::with_capacity(count);
    let mut erasures = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let cluster = i % 3;
        let [left, right] = indicators(i, rows.len(), columns, level);
        if codeword(row[0], cluster) != Some(left)
            || codeword(row[columns + 1], cluster) != Some(right)
        {
            return Ok(result);
        }
        for &bits in &row[1..=columns] {
            if let Some(value) = codeword(bits, cluster) {
                codes.push(value);
            } else {
                erasures.push(codes.len() as u32);
                codes.push(0);
            }
        }
    }
    if erasures.len() > parity / 2 {
        return Ok(result);
    }
    // One correction attempt. Charge syndrome evaluation, Euclidean
    // polynomial work and the fixed prime-field location scan before entering
    // the backend. Caller-provided row metadata cannot increase these bounds.
    let work = 64 * (count * parity + parity * parity) + 1024 * (count + parity);
    *remaining = remaining
        .checked_sub(work)
        .ok_or_else(|| limit("PDF417 correction exceeds the optical work budget"))?;
    let Ok(corrected) = ec::error_correction::decode(&mut codes, parity as u32, &mut erasures)
    else {
        return Ok(result);
    };
    if codes[0] as usize != count - parity {
        return Ok(result);
    }
    let correction_level = level.to_string();
    let Ok(decoded) = decoded_bit_stream_parser::decode(&codes, &correction_level) else {
        return Ok(result);
    };
    if decoded
        .getOther()
        .and_then(|metadata| metadata.downcast::<PDF417RXingResultMetadata>().ok())
        .is_some_and(|metadata| !metadata.getFileId().is_empty())
    {
        return Ok(result);
    }
    result.status = Status::Decoded;
    result.text = Some(decoded.getText().to_owned());
    result.correction_level = Some(correction_level);
    result.symbology_identifier = Some(format!("]L{}", decoded.getSymbologyModifier()));
    result.corrected_errors = Some(corrected);
    result.codewords = Some(codes);
    Ok(result)
}
