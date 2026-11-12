//! F-FCSR-H2005, fixed80-bit IV,160 setup clocks. The allzero key/IV
//! leaves a zero stream. This historical revision is distinct from H-v2/16.
use zeroize::Zeroizing;
const D: [u32; 5] = [0x3dd4254e, 0xaf46d590, 0x8623dc8a, 0x26619fc5, 0xae985dff];
fn clock(s: &mut [u32; 5], carry: &mut [u32; 5]) {
    let feedback = (s[0] & 1).wrapping_neg();
    for i in 0..5 {
        let shifted = (s[i] >> 1) | if i < 4 { s[i + 1] << 31 } else { 0 };
        let buffer = shifted ^ carry[i];
        carry[i] = (carry[i] & shifted) ^ (buffer & feedback & D[i]);
        s[i] = buffer ^ (feedback & D[i]);
    }
}
fn filter(s: &[u32; 5]) -> u8 {
    let mut word = 0;
    for i in 0..5 {
        word ^= s[i] & D[i];
    }
    word ^= word >> 16;
    word ^= word >> 8;
    word as u8
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut s = Zeroizing::new([0u32; 5]);
    let mut carry = Zeroizing::new([0u32; 5]);
    for i in 0..10 {
        s[i / 4] |= u32::from(key[9 - i]) << (8 * (i % 4));
        s[(i + 10) / 4] |= u32::from(iv[9 - i]) << (8 * ((i + 10) % 4));
    }
    for _ in 0..160 {
        clock(&mut s, &mut carry);
    }
    for b in output {
        clock(&mut s, &mut carry);
        *b ^= filter(&s);
    }
}
