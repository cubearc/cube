//! ZUC128 v1.6 raw stream, all32 setup rounds and the discarded first step.
#[path = "zuc-tables.rs"]
mod tables;
use zeroize::Zeroizing;
fn reorganize(s: &[u32; 16]) -> [u32; 4] {
    [
        ((s[15] & 0x7fff8000) << 1) | (s[14] & 0xffff),
        ((s[11] & 0xffff) << 16) | (s[9] >> 15),
        ((s[7] & 0xffff) << 16) | (s[5] >> 15),
        ((s[2] & 0xffff) << 16) | (s[0] >> 15),
    ]
}
fn substitution(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([
        tables::S0[b[0] as usize],
        tables::S1[b[1] as usize],
        tables::S0[b[2] as usize],
        tables::S1[b[3] as usize],
    ])
}
fn f(x: &[u32; 4], r: &mut [u32; 2]) -> u32 {
    let w = (x[0] ^ r[0]).wrapping_add(r[1]);
    let w1 = r[0].wrapping_add(x[1]);
    let w2 = r[1] ^ x[2];
    let u = (w1 << 16) | (w2 >> 16);
    let v = (w2 << 16) | (w1 >> 16);
    r[0] = substitution(
        u ^ u.rotate_left(2) ^ u.rotate_left(10) ^ u.rotate_left(18) ^ u.rotate_left(24),
    );
    r[1] = substitution(
        v ^ v.rotate_left(8) ^ v.rotate_left(14) ^ v.rotate_left(22) ^ v.rotate_left(30),
    );
    w
}
fn lfsr(s: &mut [u32; 16], u: u32) {
    const P: u32 = 0x7fffffff;
    let rot = |v: u32, n: u32| ((v << n) | (v >> (31 - n))) & P;
    let total = u64::from(s[0])
        + u64::from(rot(s[0], 8))
        + u64::from(rot(s[4], 20))
        + u64::from(rot(s[10], 21))
        + u64::from(rot(s[13], 17))
        + u64::from(rot(s[15], 15))
        + u64::from(u);
    let next = (total % u64::from(P)) as u32;
    s.copy_within(1..16, 0);
    s[15] = if next == 0 { P } else { next };
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut s = Zeroizing::new([0u32; 16]);
    let mut r = Zeroizing::new([0u32; 2]);
    for i in 0..16 {
        s[i] = (u32::from(key[i]) << 23) | (u32::from(tables::D[i]) << 8) | u32::from(iv[i]);
    }
    for _ in 0..32 {
        let x = reorganize(&s);
        let w = f(&x, &mut r);
        lfsr(&mut s, w >> 1);
    }
    f(&reorganize(&s), &mut r);
    lfsr(&mut s, 0);
    for chunk in output.chunks_mut(4) {
        let x = reorganize(&s);
        let z = f(&x, &mut r) ^ x[3];
        lfsr(&mut s, 0);
        for (b, k) in chunk.iter_mut().zip(z.to_be_bytes()) {
            *b ^= k;
        }
    }
}
