//! Exact-length entropy streams with checked, zero-padded final bytes.
use crate::binary::packing::{BitOrder, bit_shift, store};
use crate::errors::OperationError;

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    end: usize,
    order: BitOrder,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8], end: usize, order: BitOrder) -> Self {
        Self {
            bytes,
            position: 0,
            end,
            order,
        }
    }

    pub fn read(&mut self, width: u8) -> Option<u32> {
        if self.position + width as usize > self.end {
            return None;
        }
        let mut result = 0;
        for i in 0..width {
            let bit = (self.bytes[self.position / 8] >> bit_shift(self.position, self.order)) & 1;
            self.position += 1;
            let shift = match self.order {
                BitOrder::Msb => width - 1 - i,
                BitOrder::Lsb => i,
            };
            result |= u32::from(bit) << shift;
        }
        Some(result)
    }

    pub fn remaining(&self) -> usize {
        self.end - self.position
    }

    pub fn zero_padding(&self) -> bool {
        let extent = self.bytes.len() * 8;
        extent - self.position < 8
            && (self.position..extent)
                .all(|p| self.bytes[p / 8] & (1 << bit_shift(p, self.order)) == 0)
    }
}

pub(super) struct Writer {
    bytes: Vec<u8>,
    position: usize,
    order: BitOrder,
    limit: usize,
}

impl Writer {
    pub fn new(order: BitOrder, limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            position: 0,
            order,
            limit,
        }
    }

    pub fn write(&mut self, value: u32, width: u8) -> Result<(), OperationError> {
        let stop = self.position + width as usize;
        super::check_limit(stop, super::WORK_LIMIT, "entropy bit work limit exceeded")?;
        let size = stop.div_ceil(8);
        super::check_limit(size, self.limit, "entropy stream exceeds its output limit")?;
        self.bytes.resize(size, 0);
        for i in 0..width {
            let shift = match self.order {
                BitOrder::Msb => width - 1 - i,
                BitOrder::Lsb => i,
            };
            store(
                &mut self.bytes,
                self.position,
                ((value >> shift) & 1) as u8,
                self.order,
            );
            self.position += 1;
        }
        Ok(())
    }

    pub fn finish(self) -> (Vec<u8>, usize) {
        (self.bytes, self.position)
    }
}
