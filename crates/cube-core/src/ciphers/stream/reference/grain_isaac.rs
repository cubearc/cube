//! Exact Grain v1 and 32-bit ISAAC reference initialization conventions.
use zeroize::Zeroizing;

fn grain_clock(state: &mut [u128; 2], initializing: bool) -> u8 {
    let b = |i: u32| ((state[0] >> i) & 1u128) as u8;
    let s = |i: u32| ((state[1] >> i) & 1u128) as u8;
    let z = s(25)
        ^ b(63)
        ^ (s(3) & s(64))
        ^ (s(46) & s(64))
        ^ (s(64) & b(63))
        ^ (s(3) & s(25) & s(46))
        ^ (s(3) & s(46) & s(64))
        ^ (s(3) & s(46) & b(63))
        ^ (s(25) & s(46) & b(63))
        ^ (s(46) & s(64) & b(63))
        ^ b(1)
        ^ b(2)
        ^ b(4)
        ^ b(10)
        ^ b(31)
        ^ b(43)
        ^ b(56);
    let g = b(62)
        ^ b(60)
        ^ b(52)
        ^ b(45)
        ^ b(37)
        ^ b(33)
        ^ b(28)
        ^ b(21)
        ^ b(14)
        ^ b(9)
        ^ b(0)
        ^ (b(63) & b(60))
        ^ (b(37) & b(33))
        ^ (b(15) & b(9))
        ^ (b(60) & b(52) & b(45))
        ^ (b(33) & b(28) & b(21))
        ^ (b(63) & b(45) & b(28) & b(9))
        ^ (b(60) & b(52) & b(37) & b(33))
        ^ (b(63) & b(60) & b(21) & b(15))
        ^ (b(63) & b(60) & b(52) & b(45) & b(37))
        ^ (b(33) & b(28) & b(21) & b(15) & b(9))
        ^ (b(52) & b(45) & b(37) & b(33) & b(28) & b(21))
        ^ s(0);
    let f = s(0) ^ s(13) ^ s(23) ^ s(38) ^ s(51) ^ s(62);
    let feedback = if initializing { z } else { 0 };
    state[0] = (state[0] >> 1) | (u128::from(g ^ feedback) << 79);
    state[1] = (state[1] >> 1) | (u128::from(f ^ feedback) << 79);
    z
}
pub(super) fn grain(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut key_le = Zeroizing::new([0u8; 16]);
    key_le[..10].copy_from_slice(key);
    let mut state = Zeroizing::new([
        u128::from_le_bytes(*key_le),
        u128::from(u64::from_le_bytes(iv.try_into().unwrap())) | (0xffffu128 << 64),
    ]);
    for _ in 0..160 {
        grain_clock(&mut state, true);
    }
    for b in output {
        for bit in 0..8 {
            *b ^= grain_clock(&mut state, false) << bit;
        }
    }
}

fn mix(x: &mut [u32; 8]) {
    const SHIFTS: [u32; 8] = [11, 2, 8, 16, 10, 4, 8, 9];
    for i in 0..8 {
        let next = x[(i + 1) & 7];
        x[i] ^= if i & 1 == 0 {
            next << SHIFTS[i]
        } else {
            next >> SHIFTS[i]
        };
        x[(i + 3) & 7] = x[(i + 3) & 7].wrapping_add(x[i]);
        x[(i + 1) & 7] = x[(i + 1) & 7].wrapping_add(x[(i + 2) & 7]);
    }
}
fn block(m: &mut [u32; 256], r: &mut [u32; 256], abc: &mut [u32; 3]) {
    abc[2] = abc[2].wrapping_add(1);
    abc[1] = abc[1].wrapping_add(abc[2]);
    for i in 0..256 {
        let x = m[i];
        abc[0] ^= match i & 3 {
            0 => abc[0] << 13,
            1 => abc[0] >> 6,
            2 => abc[0] << 2,
            _ => abc[0] >> 16,
        };
        abc[0] = abc[0].wrapping_add(m[(i + 128) & 255]);
        let y = m[((x >> 2) & 255) as usize]
            .wrapping_add(abc[0])
            .wrapping_add(abc[1]);
        m[i] = y;
        abc[1] = m[((y >> 10) & 255) as usize].wrapping_add(x);
        r[i] = abc[1];
    }
}
pub(super) fn isaac(output: &mut [u8], key: &[u8]) {
    let mut m = Zeroizing::new([0u32; 256]);
    let mut r = Zeroizing::new([0u32; 256]);
    let mut x = Zeroizing::new([0x9e3779b9u32; 8]);
    let mut abc = Zeroizing::new([0u32; 3]);
    for (i, b) in key.as_chunks::<4>().0.iter().enumerate() {
        r[i] = u32::from_le_bytes(*b);
    }
    for _ in 0..4 {
        mix(&mut x);
    }
    for pass in 0..2 {
        for j in (0..256).step_by(8) {
            for i in 0..8 {
                x[i] = x[i].wrapping_add(if pass == 0 { r[j + i] } else { m[j + i] });
            }
            mix(&mut x);
            m[j..j + 8].copy_from_slice(&*x);
        }
    }
    // BC-compatible transport discards randinit's first generated 256 words.
    block(&mut m, &mut r, &mut abc);
    for chunk in output.chunks_mut(1024) {
        block(&mut m, &mut r, &mut abc);
        for (bytes, word) in chunk.chunks_mut(4).zip(r.iter()) {
            for (b, k) in bytes.iter_mut().zip(word.to_be_bytes()) {
                *b ^= k;
            }
        }
    }
}
