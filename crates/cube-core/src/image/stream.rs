//! Ordered byte export over the same coordinates used by bit extraction.
use super::{Channel, Corner, Image, PixelData, Region, Traversal};
use crate::errors::OperationError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PixelStream {
    #[serde(flatten)]
    pub pixels: PixelData,
    pub traversal: Traversal,
    pub start_corner: Corner,
    pub reverse: bool,
}
impl PixelStream {
    /// Bounded exact source coordinate records in stream order, not row order.
    pub fn records(&self) -> Result<Vec<super::analyze::PixelRecord>, OperationError> {
        let mut records = self.pixels.records()?;
        let coordinates =
            self.traversal
                .coordinates(self.pixels.region, self.start_corner, self.reverse)?;
        for (record, (x, y)) in records.iter_mut().zip(coordinates) {
            record.x = x;
            record.y = y;
        }
        Ok(records)
    }
}
impl Image {
    /// Export raw bytes in coordinate then selected channel order. Palette
    /// expansion is explicit; no UTF-8 interpretation or bit packing occurs.
    /// Existing image limits bound the output to 16 MiB.
    pub fn pixels_with_traversal(
        &self,
        region: Option<Region>,
        channels: Option<&[Channel]>,
        traversal: Traversal,
        start_corner: Corner,
        reverse: bool,
    ) -> Result<PixelStream, OperationError> {
        let result = (|| {
            let region = region
                .unwrap_or_else(|| Region::full(self))
                .validate(self)?;
            let (channels, indices) = self.selection(channels)?;
            let mut data =
                Vec::with_capacity(region.width as usize * region.height as usize * channels.len());
            for (x, y) in traversal.coordinates(region, start_corner, reverse)? {
                let position = self.position(x, y)?;
                for index in &indices {
                    data.push(self.samples[position + index]);
                }
            }
            Ok(PixelStream {
                pixels: PixelData {
                    region,
                    channels,
                    bit_depth: 8,
                    data,
                },
                traversal,
                start_corner,
                reverse,
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.pixels.export"))
    }
}
