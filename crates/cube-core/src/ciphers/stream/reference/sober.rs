//! SOBER-128's Rose2003 reference initialization and little-endian stream.
//! Adapted from the Unlicense LibTomCrypt state flow; no C runtime dependency.
#[path = "sober-tables.rs"]
mod tables;
use zeroize::Zeroizing;
pub(super) const MAX_KONST_STEPS: usize = 65536;
#[derive(Debug)]
pub(super) struct SetupLimit;
struct State {
    r: Zeroizing<[u32; 17]>,
    konst: Zeroizing<u32>,
}
impl State {
    fn cycle(&mut self) {
        let next =
            self.r[15] ^ self.r[4] ^ (self.r[0] << 8) ^ tables::MULTAB[(self.r[0] >> 24) as usize];
        self.r.copy_within(1..17, 0);
        self.r[16] = next;
    }
    fn tap(&self) -> u32 {
        let mut t = self.r[0].wrapping_add(self.r[16]);
        t ^= tables::SBOX[(t >> 24) as usize];
        t = t.rotate_right(8);
        t = (t.wrapping_add(self.r[1]) ^ *self.konst).wrapping_add(self.r[6]);
        t ^= tables::SBOX[(t >> 24) as usize];
        t.wrapping_add(self.r[13])
    }
    fn diffuse(&mut self) {
        for _ in 0..17 {
            self.cycle();
            self.r[4] ^= self.tap();
        }
    }
    fn absorb(&mut self, material: &[u8]) {
        for bytes in material.as_chunks::<4>().0 {
            self.r[15] = self.r[15].wrapping_add(u32::from_le_bytes(*bytes));
            self.cycle();
            self.r[4] ^= self.tap();
        }
        self.r[15] = self.r[15].wrapping_add(material.len() as u32);
        self.diffuse();
    }
    fn generate_constant(&mut self, limit: usize) -> Result<(), SetupLimit> {
        for _ in 0..limit {
            self.cycle();
            let next = self.tap();
            if next & 0xff000000 != 0 {
                *self.konst = next;
                return Ok(());
            }
        }
        Err(SetupLimit)
    }
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) -> Result<(), SetupLimit> {
    let mut state = State {
        r: Zeroizing::new([0; 17]),
        konst: Zeroizing::new(0x6996c53a),
    };
    state.r[0] = 1;
    state.r[1] = 1;
    for i in 2..17 {
        state.r[i] = state.r[i - 1].wrapping_add(state.r[i - 2]);
    }
    state.absorb(key);
    state.generate_constant(MAX_KONST_STEPS)?;
    state.absorb(iv);
    for chunk in output.chunks_mut(4) {
        state.cycle();
        let z = state.tap();
        for (b, k) in chunk.iter_mut().zip(z.to_le_bytes()) {
            *b ^= k;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn setup_work_budget_is_finite_and_fails_explicitly() {
        let mut s = State {
            r: Zeroizing::new([1; 17]),
            konst: Zeroizing::new(0x6996c53a),
        };
        assert!(s.generate_constant(0).is_err());
        assert!(s.generate_constant(MAX_KONST_STEPS).is_ok());
        assert_ne!(*s.konst & 0xff000000, 0);
    }
}
