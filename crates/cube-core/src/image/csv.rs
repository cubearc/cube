//! Version 1 exact-sample CSV, deliberately limited to small complete 8-bit grids.
use super::{ColorModel, Image, limit, malformed, sample_count, unsupported};
use crate::{errors::OperationError, operations::Descriptor};
use std::fmt::Write;

/// Both encoded CSV input and output are capped at 2 MiB.
pub const MAX_BYTES: usize = 2 * 1024 * 1024;
/// Matches the existing explicit pixel-record budget.
pub const MAX_RECORDS: usize = 10000;
pub const MAX_ROW_BYTES: usize = 128;
pub const MAX_FIELD_BYTES: usize = 16;

fn failure(row: usize, field: &str, message: &str) -> OperationError {
    malformed(&format!("CSV row {row}, field {field}: {message}")).parameter("csv")
}

fn layout(model: ColorModel) -> Result<(&'static str, &'static str), OperationError> {
    Ok(match model {
        ColorModel::Gray => ("gray", "x,y,gray"),
        ColorModel::GrayAlpha => ("gray_alpha", "x,y,gray,alpha"),
        ColorModel::Rgb => ("rgb", "x,y,red,green,blue"),
        ColorModel::Rgba => ("rgba", "x,y,red,green,blue,alpha"),
        ColorModel::Indexed => {
            return Err(unsupported("CSV v1 does not preserve indexed palettes"));
        }
    })
}

// csv-core intentionally accepts some broken CSV. Validate this schema's small
// ASCII field grammar first, then use the maintained fixed-buffer tokenizer.
fn record(line: &str, row: usize) -> Result<Vec<String>, OperationError> {
    if line.len() > MAX_ROW_BYTES {
        return Err(limit(&format!("CSV row {row} exceeds 128 bytes")).parameter("csv"));
    }
    if line.is_empty() || !line.is_ascii() || line.contains(['\r', '\n']) {
        return Err(failure(row, "record", "expected one nonempty ASCII record"));
    }
    let fields: Vec<_> = line.split(',').collect();
    if fields.len() > 6 {
        return Err(failure(row, "record", "at most six fields are supported"));
    }
    for (i, field) in fields.iter().enumerate() {
        let inner = if field.starts_with('"') && field.ends_with('"') && field.len() >= 2 {
            &field[1..field.len() - 1]
        } else {
            field
        };
        if inner.len() > MAX_FIELD_BYTES {
            return Err(
                limit(&format!("CSV row {row}, field {} exceeds 16 bytes", i + 1)).parameter("csv"),
            );
        }
        if inner.contains('"') {
            return Err(failure(
                row,
                &(i + 1).to_string(),
                "malformed whole-field quotes",
            ));
        }
    }
    let mut input = [0u8; MAX_ROW_BYTES + 1];
    input[..line.len()].copy_from_slice(line.as_bytes());
    input[line.len()] = b'\n';
    let mut output = [0u8; MAX_ROW_BYTES];
    let mut ends = [0usize; 8];
    let (status, consumed, _, count) =
        csv_core::Reader::new().read_record(&input[..line.len() + 1], &mut output, &mut ends);
    if status != csv_core::ReadRecordResult::Record
        || consumed != line.len() + 1
        || count != fields.len()
    {
        return Err(failure(row, "record", "unsupported CSV syntax"));
    }
    let mut start = 0;
    Ok(ends[..count]
        .iter()
        .map(|&end| {
            let value = String::from_utf8(output[start..end].to_vec()).expect("validated ASCII");
            start = end;
            value
        })
        .collect())
}

fn number(value: &str, row: usize, field: &str) -> Result<u32, OperationError> {
    if value.is_empty() || value.len() > 10 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(failure(row, field, "expected an unsigned decimal integer"));
    }
    value
        .parse()
        .map_err(|_| failure(row, field, "integer exceeds u32"))
}

impl Image {
    /// Export CSV v1 in top-left row-major order, using native physical channels.
    /// Supports gray/gray_alpha/rgb/rgba only, without palette or tRNS metadata.
    /// Limits: 10000 pixels, 2 MiB CSV, 128 bytes/row, 16 bytes/field. No writes.
    /// CSV preserves shape/model/samples, not encoded bytes or ancillary metadata.
    pub fn to_csv(&self) -> Result<String, OperationError> {
        let result = (|| {
            let (model, columns) = layout(self.model())?;
            if !self.palette().is_empty() || !self.transparency().is_empty() {
                return Err(unsupported(
                    "CSV v1 cannot preserve palette or tRNS; use an explicit sample conversion",
                ));
            }
            let count = u64::from(self.width()) * u64::from(self.height());
            if count > MAX_RECORDS as u64 {
                return Err(limit("CSV exceeds 10000 pixel records"));
            }
            // Every possible supported row fits 128 bytes, so reserve and check
            // a conservative bound before obtaining any coordinate records.
            let bound = (count as usize + 2)
                .checked_mul(MAX_ROW_BYTES + 1)
                .ok_or_else(|| limit("CSV output size overflow"))?;
            if bound > MAX_BYTES {
                return Err(limit("CSV output exceeds 2 MiB"));
            }
            let mut text = String::with_capacity(bound);
            writeln!(
                text,
                "cube_pixels,1,{},{},{model},8",
                self.width(),
                self.height()
            )
            .expect("String write");
            writeln!(text, "{columns}").expect("String write");
            for pixel in self.pixels(None, None)?.records()? {
                write!(text, "{},{}", pixel.x, pixel.y).expect("String write");
                for value in pixel.values {
                    write!(text, ",{value}").expect("String write");
                }
                text.push('\n');
            }
            Ok(text)
        })();
        result.map_err(|e: OperationError| e.operation("image.csv.export"))
    }

    /// Import a complete version-1 CSV grid. Coordinate rows may be reordered;
    /// missing, duplicate and invalid cells fail before an Image is exposed.
    /// LF/CRLF and optional whole-field quotes are accepted; embedded quotes,
    /// delimiters/newlines, blank records, BOM and numeric whitespace are not.
    /// Dimensions obey Image limits, with at most 10000 pixels and 2 MiB input.
    pub fn from_csv(text: &str) -> Result<Self, OperationError> {
        let result = (|| {
            if text.len() > MAX_BYTES {
                return Err(limit("CSV input exceeds 2 MiB").parameter("csv"));
            }
            let mut lines = text.lines();
            let header = record(lines.next().unwrap_or_default(), 1)?;
            if header.len() != 6 || header[0] != "cube_pixels" || header[1] != "1" {
                return Err(failure(
                    1,
                    "schema/version",
                    "expected cube_pixels,1,width,height,model,8",
                ));
            }
            if header[5] != "8" {
                return Err(failure(1, "sample_depth", "only depth 8 is supported"));
            }
            let width = number(&header[2], 1, "width")?;
            let height = number(&header[3], 1, "height")?;
            let model = ColorModel::parse(&header[4])
                .map_err(|_| failure(1, "model", "expected gray, gray_alpha, rgb or rgba"))?;
            let (_, columns) = layout(model)
                .map_err(|_| failure(1, "model", "indexed palettes are unsupported"))?;
            let size = sample_count(width, height, model.channels()).map_err(|e| {
                OperationError::new(
                    e.code,
                    format!("CSV row 1, field width/height: {}", e.message),
                )
                .parameter("csv")
            })?;
            let count = size / model.channels();
            if count > MAX_RECORDS {
                return Err(limit("CSV exceeds 10000 pixel records").parameter("csv"));
            }
            let column_fields = record(lines.next().unwrap_or_default(), 2)?;
            if column_fields.join(",") != columns {
                return Err(failure(2, "columns", &format!("expected {columns}")));
            }
            let mut samples = vec![0u8; size];
            let mut seen = vec![false; count];
            let mut records = 0;
            for (i, line) in lines.enumerate() {
                let row = i + 3;
                if records == count {
                    return Err(failure(row, "record", "more records than declared pixels"));
                }
                let fields = record(line, row)?;
                if fields.len() != model.channels() + 2 {
                    return Err(failure(row, "channels", "wrong field count"));
                }
                let x = number(&fields[0], row, "x")?;
                let y = number(&fields[1], row, "y")?;
                if x >= width || y >= height {
                    return Err(failure(
                        row,
                        "x/y",
                        "coordinate outside declared dimensions",
                    ));
                }
                let index = y as usize * width as usize + x as usize;
                if seen[index] {
                    return Err(failure(row, "x/y", "duplicate coordinate"));
                }
                for (channel, value) in fields[2..].iter().enumerate() {
                    let name = &column_fields[channel + 2];
                    samples[index * model.channels() + channel] =
                        u8::try_from(number(value, row, name)?)
                            .map_err(|_| failure(row, name, "sample must be 0..=255"))?;
                }
                seen[index] = true;
                records += 1;
            }
            if records != count {
                return Err(failure(records + 3, "record", "missing pixel coordinates"));
            }
            Self::from_samples(width, height, model, samples, vec![], vec![])
        })();
        result.map_err(|e: OperationError| e.operation("image.csv.import"))
    }
}

pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "image.csv.export",
        category: "image",
        description: "Export CSV v1 exact 8-bit gray/gray_alpha/rgb/rgba samples without palette/tRNS; 10000 pixels, 2 MiB, 128-byte rows/16-byte fields. Shape and native channel order retained; encoded/ancillary metadata excluded.",
        input_type: "image.samples",
        output_type: "text",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: "image csv export",
        example: "cube image csv export sample.png --output pixels.csv",
    },
    Descriptor {
        id: "image.csv.import",
        category: "image",
        description: "Import a complete CSV v1 grid, validating explicit shape/model/depth, zero-based coordinates and 0..255 samples. LF/CRLF and whole-field quotes; no embedded delimiters/newlines/quotes. 10000 pixels, 2 MiB, 128-byte rows/16-byte fields; constructed samples only.",
        input_type: "text",
        output_type: "image.samples",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: "image csv import",
        example: "cube image csv import --input pixels.csv --output restored.png",
    },
];
