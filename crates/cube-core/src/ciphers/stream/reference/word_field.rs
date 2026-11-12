//! Shared GF(2^32) arithmetic for SOSEMANUK and SNOW.
const fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        a = (a << 1) ^ if a & 128 != 0 { 0xa9 } else { 0 };
        b >>= 1;
    }
    r
}
const fn masks(coeff: [u8; 4]) -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = u32::from_be_bytes([
            gf_mul(i as u8, coeff[0]),
            gf_mul(i as u8, coeff[1]),
            gf_mul(i as u8, coeff[2]),
            gf_mul(i as u8, coeff[3]),
        ]);
        i += 1;
    }
    table
}
// beta^23, beta^245, beta^48, beta^239 over x^8+x^7+x^5+x^3+1.
const MUL: [u32; 256] = masks([0xe1, 0x9f, 0xcf, 0x13]);
// Inverse alpha derived in the same field: beta^16, beta^39, beta^6, beta^64.
const DIV: [u32; 256] = masks([0x18, 0x0f, 0x40, 0xcd]);

pub(super) fn mul(x: u32) -> u32 {
    (x << 8) ^ MUL[(x >> 24) as usize]
}
pub(super) fn div(x: u32) -> u32 {
    (x >> 8) ^ DIV[(x & 255) as usize]
}
