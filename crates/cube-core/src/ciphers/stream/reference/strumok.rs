//! Strumok 256/512-bit keys, explicit little-endian words and full 32-step setup.
//! Safe state flow follows the DSTU8845 reference, with owned-state wiping.
#[path = "strumok-tables.rs"]
mod tables;
use zeroize::Zeroizing;
fn step(s: &mut [u64; 16], r: &mut [u64; 2], head: usize, initializing: bool) -> u64 {
    let feedback = if initializing {
        r[0].wrapping_add(s[(head + 15) & 15]) ^ r[1]
    } else {
        0
    };
    s[head] = tables::a_mul(s[head])
        ^ s[(head + 13) & 15]
        ^ tables::ainv_mul(s[(head + 11) & 15])
        ^ feedback;
    let next_r = r[1].wrapping_add(s[(head + 13) & 15]);
    r[1] = tables::t(r[0]);
    r[0] = next_r;
    r[0].wrapping_add(s[head]) ^ r[1] ^ s[(head + 1) & 15]
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut k = Zeroizing::new([0u64; 8]);
    let mut v = Zeroizing::new([0u64; 4]);
    for (i, b) in key.as_chunks::<8>().0.iter().enumerate() {
        k[i] = u64::from_le_bytes(*b);
    }
    for (i, b) in iv.as_chunks::<8>().0.iter().enumerate() {
        v[i] = u64::from_le_bytes(*b);
    }
    let mut s = Zeroizing::new(if key.len() == 32 {
        [
            k[3] ^ v[0],
            k[2],
            k[1] ^ v[1],
            k[0] ^ v[2],
            k[3],
            k[2] ^ v[3],
            !k[1],
            !k[0],
            k[3],
            k[2],
            !k[1],
            k[0],
            k[3],
            !k[2],
            k[1],
            !k[0],
        ]
    } else {
        [
            k[7] ^ v[0],
            k[6],
            k[5],
            k[4] ^ v[1],
            k[3],
            k[2] ^ v[2],
            k[1],
            !k[0],
            k[4] ^ v[3],
            !k[6],
            k[5],
            !k[7],
            k[3],
            k[2],
            !k[1],
            k[0],
        ]
    });
    let mut r = Zeroizing::new([0u64; 2]);
    for i in 0..32 {
        step(&mut s, &mut r, i & 15, true);
    }
    for (i, chunk) in output.chunks_mut(8).enumerate() {
        let z = step(&mut s, &mut r, i & 15, false);
        for (b, k) in chunk.iter_mut().zip(z.to_le_bytes()) {
            *b ^= k;
        }
    }
}
