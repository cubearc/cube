//! Shared finite serialized-byte accounting for concrete mapping reports.
use std::io::{self, Write};
pub(crate) struct ByteBudget {
    remaining: usize,
    initial: usize,
}
impl ByteBudget {
    pub(crate) fn written(&self) -> usize {
        self.initial - self.remaining
    }
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            remaining: limit,
            initial: limit,
        }
    }
}
impl Write for ByteBudget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(io::Error::other("serialized byte limit exceeded"));
        }
        self.remaining -= bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
