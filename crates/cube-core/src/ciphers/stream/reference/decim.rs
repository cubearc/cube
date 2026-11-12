//! DECIMv2 from designers' BBC08b specification, sections3.1–3.4.
//! Independent safe Rust math; restricted reference C is validation-only.
use zeroize::Zeroizing;
const F: [usize; 13] = [13, 28, 45, 54, 65, 104, 111, 144, 162, 172, 178, 186, 191];
const L: [usize; 14] = [0, 3, 4, 23, 36, 37, 60, 61, 98, 115, 146, 175, 176, 187];
pub(super) const MAX_PREFILL: usize = 65536;
#[derive(Debug)]
pub(super) struct WorkLimit;
enum Abs {
    First,
    Second(Zeroizing<u8>),
    Seek(Zeroizing<u8>),
}
impl Abs {
    fn feed(&mut self, bit: u8) -> Option<u8> {
        match self {
            Self::First => {
                *self = Self::Second(Zeroizing::new(bit));
                None
            }
            Self::Second(first) if **first == bit => {
                *self = Self::First;
                Some(bit)
            }
            Self::Second(first) => {
                let value = **first;
                *self = Self::Seek(Zeroizing::new(value));
                None
            }
            Self::Seek(first) if **first == bit => {
                *self = Self::First;
                Some(bit ^ 1)
            }
            Self::Seek(_) => None,
        }
    }
}
struct State {
    s: Zeroizing<[u64; 3]>,
    abs: Abs,
    queue: Zeroizing<u32>,
    len: usize,
}
impl State {
    fn bit(&self, n: usize) -> u8 {
        ((self.s[n / 64] >> (n % 64)) & 1) as u8
    }
    fn clock(&mut self, initializing: bool) -> u8 {
        let weight: u8 = F.iter().map(|&n| self.bit(n)).sum();
        let nonlinear = ((weight >> 1) ^ weight) & 1;
        let output = nonlinear ^ self.bit(1);
        let linear = L.iter().fold(0, |v, &n| v ^ self.bit(n));
        let next = linear ^ if initializing { nonlinear } else { 0 };
        self.s[0] = (self.s[0] >> 1) | (self.s[1] << 63);
        self.s[1] = (self.s[1] >> 1) | (self.s[2] << 63);
        self.s[2] = (self.s[2] >> 1) | (u64::from(next) << 63);
        output
    }
    fn step(&mut self) {
        let bit = self.clock(false);
        if let Some(bit) = self.abs.feed(bit)
            && self.len < 32
        {
            *self.queue |= u32::from(bit) << self.len;
            self.len += 1;
        }
    }
    fn prefill(&mut self, limit: usize) -> Result<(), WorkLimit> {
        for _ in 0..limit / 4 {
            for _ in 0..4 {
                self.step();
            }
            if self.len == 32 {
                return Ok(());
            }
        }
        Err(WorkLimit)
    }
    fn pop(&mut self) -> Result<u8, WorkLimit> {
        if self.len == 0 {
            return Err(WorkLimit);
        }
        let bit = (*self.queue & 1) as u8;
        *self.queue >>= 1;
        self.len -= 1;
        Ok(bit)
    }
}
fn bit(bytes: &[u8], n: usize) -> u64 {
    u64::from((bytes[n / 8] >> (n % 8)) & 1)
}
fn initialize(key: &[u8], iv: &[u8]) -> State {
    let mut state = State {
        s: Zeroizing::new([0; 3]),
        abs: Abs::First,
        queue: Zeroizing::new(0),
        len: 0,
    };
    for i in 0..192 {
        let value = match i {
            0..80 => bit(key, i),
            80..144 => bit(key, i - 80) ^ bit(iv, i - 80),
            144..160 => {
                bit(key, i - 80)
                    ^ bit(iv, i - 144)
                    ^ bit(iv, i - 128)
                    ^ bit(iv, i - 112)
                    ^ bit(iv, i - 96)
            }
            _ => bit(iv, i - 160) ^ bit(iv, i - 128) ^ 1,
        };
        state.s[i / 64] |= value << (i % 64);
    }
    for _ in 0..768 {
        state.clock(true);
    }
    state
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), WorkLimit> {
    let mut state = initialize(key, iv);
    state.prefill(MAX_PREFILL)?;
    for byte in output {
        for n in 0..8 {
            for _ in 0..4 {
                state.step();
            }
            *byte ^= state.pop()? << n;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initialization_and_empty_buffer_fail_with_a_finite_budget() {
        let mut state = initialize(&[0; 10], &[0; 8]);
        assert!(state.prefill(0).is_err());
        assert!(state.pop().is_err());
        assert!(state.prefill(MAX_PREFILL).is_ok());
        assert_eq!(state.len, 32);
    }
}
