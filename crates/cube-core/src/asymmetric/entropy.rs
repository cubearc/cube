//! Fallible OS/browser CSPRNG adapter; no caller-supplied production coins.
use rand_core::{TryCryptoRng, TryRng};
#[derive(Debug)]
pub(super) struct EntropyError;
impl std::fmt::Display for EntropyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("system entropy unavailable")
    }
}
impl std::error::Error for EntropyError {}
pub(super) struct SystemRng {
    remaining: usize,
}
impl Default for SystemRng {
    fn default() -> Self {
        Self { remaining: 65536 }
    }
}
impl TryRng for SystemRng {
    type Error = EntropyError;
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.remaining = self
            .remaining
            .checked_sub(dst.len().max(1))
            .ok_or(EntropyError)?;
        getrandom02::getrandom(dst).map_err(|_| EntropyError)
    }
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut b = [0; 4];
        self.try_fill_bytes(&mut b)?;
        Ok(u32::from_le_bytes(b))
    }
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut b = [0; 8];
        self.try_fill_bytes(&mut b)?;
        Ok(u64::from_le_bytes(b))
    }
}
impl TryCryptoRng for SystemRng {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_entropy_budget_fails_before_requesting_more_bytes() {
        let mut rng = SystemRng { remaining: 0 };
        assert!(rng.try_fill_bytes(&mut [0; 1]).is_err());
        assert!(rng.try_fill_bytes(&mut []).is_err());
    }
}
