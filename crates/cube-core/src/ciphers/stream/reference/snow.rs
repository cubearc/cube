//! Full SNOW2.0 and SNOW3G raw generators with exact external word ordering.
use super::word_field as field;
use crate::ciphers::block_profiles::rijndael as aes;
#[path = "snow-sq.rs"]
mod sq;
use zeroize::Zeroizing;
fn nonlinear(w: u32, table: &[u8; 256], polynomial: u8) -> u32 {
    let b = w.to_be_bytes().map(|b| table[b as usize]);
    let x = b.map(|b| (b << 1) ^ if b & 128 != 0 { polynomial } else { 0 });
    u32::from_be_bytes([
        x[0] ^ b[1] ^ b[2] ^ x[3] ^ b[3],
        x[0] ^ b[0] ^ x[1] ^ b[2] ^ b[3],
        b[0] ^ x[1] ^ b[1] ^ x[2] ^ b[3],
        b[0] ^ b[1] ^ x[2] ^ b[2] ^ x[3],
    ])
}
fn fsm(s: &[u32; 16], r: &mut [u32; 3], third_generation: bool) -> u32 {
    let f = s[15].wrapping_add(r[0]) ^ r[1];
    let next = r[1].wrapping_add(s[5] ^ if third_generation { r[2] } else { 0 });
    if third_generation {
        r[2] = nonlinear(r[1], &sq::SQ, 0x69);
    }
    r[1] = nonlinear(r[0], &aes::SBOX, 0x1b);
    r[0] = next;
    f
}
fn lfsr(s: &mut [u32; 16], feedback: u32) {
    let next = field::mul(s[0]) ^ s[2] ^ field::div(s[11]) ^ feedback;
    s.copy_within(1..16, 0);
    s[15] = next;
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8], third_generation: bool) {
    let mut k = Zeroizing::new([0u32; 8]);
    let mut v = Zeroizing::new([0u32; 4]);
    for (i, b) in key.as_chunks::<4>().0.iter().enumerate() {
        k[i] = u32::from_be_bytes(*b);
    }
    for (i, b) in iv.as_chunks::<4>().0.iter().enumerate() {
        v[i] = u32::from_be_bytes(*b);
    }
    let mut s = Zeroizing::new([0u32; 16]);
    let mut r = Zeroizing::new([0u32; 3]);
    if third_generation {
        for i in 0..4 {
            s[i] = !k[i];
            s[i + 4] = k[i];
            s[i + 8] = !k[i];
            s[i + 12] = k[i];
        }
    } else if key.len() == 16 {
        for i in 0..4 {
            s[15 - i] = k[i];
            s[11 - i] = !k[i];
            s[7 - i] = k[i];
            s[3 - i] = !k[i];
        }
    } else {
        for i in 0..8 {
            s[15 - i] = k[i];
            s[7 - i] = !k[i];
        }
    }
    if !third_generation {
        v.reverse();
    }
    s[15] ^= v[0];
    s[12] ^= v[1];
    s[10] ^= v[2];
    s[9] ^= v[3];
    for _ in 0..32 {
        let f = fsm(&s, &mut r, third_generation);
        lfsr(&mut s, f);
    }
    fsm(&s, &mut r, third_generation);
    lfsr(&mut s, 0);
    for chunk in output.chunks_mut(4) {
        let z = fsm(&s, &mut r, third_generation) ^ s[0];
        for (b, k) in chunk.iter_mut().zip(z.to_be_bytes()) {
            *b ^= k;
        }
        lfsr(&mut s, 0);
    }
}
