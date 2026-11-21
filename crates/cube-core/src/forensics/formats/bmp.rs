//! Windows BITMAPINFOHEADER framing and uncompressed raster extents.
use super::*;
use crate::encoding::hex_encode;

#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct Report {
    pub width: u32,
    pub height: u32,
    pub top_down: bool,
    pub bits_per_pixel: u16,
    pub compression: &'static str,
    pub declared_size: u32,
    pub row_stride_bytes: u64,
    pub raster: Extent,
    pub color_table: Option<Extent>,
    pub channel_masks: Option<[String; 3]>,
    pub logical_end: u64,
    pub trailing_bytes: u64,
    pub unsupported_features: Vec<&'static str>,
    pub payload_verified: bool,
}

/// Inspect only the Windows 40-byte DIB BI_RGB and BI_BITFIELDS subsets.
pub fn inspect(bytes: &[u8]) -> Result<Report, OperationError> {
    check_source(bytes)?;
    let prefix = slice(bytes, 0, 18)?;
    if &prefix[..2] != b"BM" {
        return Err(unsupported("expected a Windows BMP file"));
    }
    if le32(prefix, 14) != 40 {
        return Err(unsupported(
            "only the 40-byte Windows BITMAPINFOHEADER is supported",
        ));
    }
    let head = slice(bytes, 0, 54)?;
    let declared = le32(head, 2);
    let logical = declared as usize;
    let raster = le32(head, 10) as usize;
    let width = le32(head, 18) as i32;
    let height = le32(head, 22) as i32;
    let bits = le16(head, 28);
    let profile = le32(head, 30);
    let invalid = || malformed("invalid BMP dimensions, header or raster range");
    if le32(head, 6) != 0
        || le16(head, 26) != 1
        || width <= 0
        || height == 0
        || !(54..=bytes.len()).contains(&logical)
        || raster > logical
    {
        return Err(invalid());
    }
    let (compression, end, masks) = match (profile, bits) {
        (0, 1 | 4 | 8 | 16 | 24 | 32) => ("bi_rgb", 54, None),
        (3, 16 | 32) => {
            let data = slice(bytes, 54, 12)?;
            let (r, g, b) = (le32(data, 0), le32(data, 4), le32(data, 8));
            let fits = |mask: u32| {
                let field = mask >> mask.trailing_zeros().min(31);
                mask != 0 && field & field.wrapping_add(1) == 0 && u64::from(mask) < (1u64 << bits)
            };
            if ![r, g, b].into_iter().all(fits) || r & g != 0 || r & b != 0 || g & b != 0 {
                return Err(invalid());
            }
            (
                "bi_bitfields",
                66,
                Some([r, g, b].map(|mask| hex_encode(&mask.to_be_bytes()))),
            )
        }
        (0 | 3, _) => return Err(invalid()),
        _ => return Err(unsupported("unsupported BMP compression")),
    };
    let used = le32(head, 46);
    let colors = if bits <= 8 {
        let max = 1 << bits;
        if used > max {
            return Err(invalid());
        }
        if used == 0 { max } else { used }
    } else {
        used
    };
    let table_size = u64::from(colors) * 4;
    if end as u64 + table_size > raster as u64 {
        return Err(invalid());
    }
    let stride = (width as u64 * u64::from(bits)).div_ceil(32) * 4;
    let raster_size = stride
        .checked_mul(u64::from(height.unsigned_abs()))
        .ok_or_else(invalid)?;
    let image_size = u64::from(le32(head, 34));
    if raster_size > (logical - raster) as u64 || (image_size != 0 && image_size != raster_size) {
        return Err(invalid());
    }
    Ok(Report {
        width: width as u32,
        height: height.unsigned_abs(),
        top_down: height < 0,
        bits_per_pixel: bits,
        compression,
        declared_size: declared,
        row_stride_bytes: stride,
        raster: Extent {
            offset: raster as u64,
            length: raster_size,
        },
        color_table: (colors != 0).then_some(Extent {
            offset: end as u64,
            length: table_size,
        }),
        channel_masks: masks,
        logical_end: logical as u64,
        trailing_bytes: (bytes.len() - logical) as u64,
        unsupported_features: vec![],
        payload_verified: false,
    })
}
