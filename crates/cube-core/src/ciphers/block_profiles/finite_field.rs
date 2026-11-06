//! Private byte-field arithmetic with an explicit reduction polynomial.
pub(super) const fn gf(mut a: u8, mut b: u8, polynomial: u8) -> u8 {
    let mut out = 0;
    let mut i = 0;
    while i < 8 {
        if b & 1 != 0 {
            out ^= a;
        }
        a = (a << 1) ^ ((a >> 7).wrapping_neg() & polynomial);
        b >>= 1;
        i += 1;
    }
    out
}
