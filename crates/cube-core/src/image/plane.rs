//! Checked borrowed physical-channel views shared by image measurements.
use super::{Channel, Image, Region, limit};
use crate::errors::OperationError;

pub(super) const MIB: usize = 1 << 20;

#[derive(Clone, Copy)]
pub(super) struct Plane<'a> {
    pub width: usize,
    pub height: usize,
    samples: &'a [u8],
    offset: usize,
    stride: usize,
    channels: usize,
}

impl<'a> Plane<'a> {
    pub fn new(
        image: &'a Image,
        region: Region,
        channel: Channel,
        max_pixels: usize,
    ) -> Result<Self, OperationError> {
        let region = region.validate(image)?;
        let (width, height) = (region.width as usize, region.height as usize);
        if width * height > max_pixels {
            return Err(limit("selected physical channel exceeds its pixel budget"));
        }
        let (_, indices) = image.selection(Some(&[channel]))?;
        let channels = image.model().channels();
        let stride = image.width() as usize * channels;
        let offset = region.y as usize * stride + region.x as usize * channels + indices[0];
        Ok(Self {
            width,
            height,
            samples: image.samples(),
            offset,
            stride,
            channels,
        })
    }

    pub fn sample(self, x: usize, y: usize) -> u8 {
        self.samples[self.offset + y * self.stride + x * self.channels]
    }

    pub fn len(self) -> usize {
        self.width * self.height
    }
}

pub(super) fn border_index(index: isize, size: usize, replicate: bool) -> Option<usize> {
    if replicate {
        Some(index.clamp(0, size as isize - 1) as usize)
    } else {
        usize::try_from(index).ok().filter(|&i| i < size)
    }
}
