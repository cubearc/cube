//! Trivium with the eSTREAM 80-bit key/IV byte convention and full1152 clocks.
use zeroize::Zeroizing;
fn clock(s: &mut [u128; 3]) -> u8 {
    let [a, b, c] = *s;
    let t1 = (a >> 65) ^ (a >> 92);
    let t2 = (b >> 68) ^ (b >> 83);
    let t3 = (c >> 65) ^ (c >> 110);
    let z = ((t1 ^ t2 ^ t3) & 1) as u8;
    let t1 = t1 ^ ((a >> 90) & (a >> 91)) ^ (b >> 77);
    let t2 = t2 ^ ((b >> 81) & (b >> 82)) ^ (c >> 86);
    let t3 = t3 ^ ((c >> 108) & (c >> 109)) ^ (a >> 68);
    s[0] = ((a << 1) | (t3 & 1)) & ((1u128 << 93) - 1);
    s[1] = ((b << 1) | (t1 & 1)) & ((1u128 << 84) - 1);
    s[2] = ((c << 1) | (t2 & 1)) & ((1u128 << 111) - 1);
    z
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let load = |bytes: &[u8]| {
        bytes
            .iter()
            .flat_map(|b| (0..8).map(move |j| (b >> j) & 1))
            .fold(0u128, |v, b| (v << 1) | u128::from(b))
    };
    let mut s = Zeroizing::new([load(key), load(iv), 7u128 << 108]);
    for _ in 0..1152 {
        clock(&mut s);
    }
    for byte in output {
        for bit in 0..8 {
            *byte ^= clock(&mut s) << bit;
        }
    }
}
