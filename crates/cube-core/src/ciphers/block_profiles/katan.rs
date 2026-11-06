//! Original KATAN/KTANTAN full 254-round profiles, numeric BE bytes.
//! Own register arithmetic; original-author C is an independent oracle only.
use zeroize::Zeroizing;
const fn controls() -> [u8; 254] {
    let mut out = [0; 254];
    let mut state = 255u8;
    let mut r = 0;
    while r < 254 {
        let next = ((state >> 7) ^ (state >> 6) ^ (state >> 4) ^ (state >> 2)) & 1;
        state = (state << 1) | next;
        out[r] = state;
        r += 1;
    }
    out
}
const CONTROL: [u8; 254] = controls();
fn keys(key: &[u8], fixed: bool) -> Zeroizing<[[u8; 2]; 254]> {
    let mut bits = Zeroizing::new([0u8; 508]);
    for i in 0..80 {
        bits[i] = (key[9 - i / 8] >> (i % 8)) & 1;
    }
    let mut result = Zeroizing::new([[0; 2]; 254]);
    if fixed {
        for (r, out) in result.iter_mut().enumerate() {
            let t = CONTROL[r];
            let offset = usize::from(t >> 4);
            let low = usize::from(t & 3);
            let a = if t & 12 == 0 { 0 } else { 1 + low };
            let b = if t & 12 == 4 { 4 } else { 3 - low };
            *out = [bits[16 * a + offset], bits[16 * b + offset]];
        }
    } else {
        for i in 80..508 {
            bits[i] = bits[i - 80] ^ bits[i - 61] ^ bits[i - 50] ^ bits[i - 13];
        }
        for (r, out) in result.iter_mut().enumerate() {
            *out = [bits[2 * r], bits[2 * r + 1]];
        }
    }
    result
}
fn feedback_a(a: u64, x: [u32; 5], ir: u64, key: u8) -> u64 {
    ((a >> x[1]) ^ ((a >> x[2]) & (a >> x[3])) ^ ((a >> x[4]) & ir) ^ u64::from(key)) & 1
}
fn feedback_b(b: u64, y: [u32; 6], key: u8) -> u64 {
    ((b >> y[1]) ^ ((b >> y[2]) & (b >> y[3])) ^ ((b >> y[4]) & (b >> y[5])) ^ u64::from(key)) & 1
}
pub(super) fn transform(
    data: &[u8],
    key: &[u8],
    width: usize,
    fixed: bool,
    inverse: bool,
) -> Vec<u8> {
    let (x, y, steps) = match width {
        4 => ([12, 7, 8, 5, 3], [18, 7, 12, 10, 8, 3], 1),
        6 => ([18, 12, 15, 7, 6], [28, 19, 21, 13, 15, 6], 2),
        8 => ([24, 15, 20, 11, 9], [38, 25, 33, 21, 14, 9], 3),
        _ => unreachable!("validated width"),
    };
    let keys = keys(key, fixed);
    let len_b = y[0] + 1;
    let ma = (1u64 << (x[0] + 1)) - 1;
    let mb = (1u64 << len_b) - 1;
    let mut result = data.to_vec();
    for block in result.chunks_exact_mut(width) {
        let word = block.iter().fold(0u64, |n, b| (n << 8) | u64::from(*b));
        let mut state = Zeroizing::new([word >> len_b, word & mb]);
        for i in 0..254 {
            let r = if inverse { 253 - i } else { i };
            let ir = u64::from(CONTROL[r] >> 7);
            for _ in 0..steps {
                let (a, b) = (state[0], state[1]);
                if inverse {
                    let mut old_a = a >> 1;
                    let mut old_b = b >> 1;
                    old_a |= ((b & 1) ^ feedback_a(old_a, x, ir, keys[r][0])) << x[0];
                    old_b |= ((a & 1) ^ feedback_b(old_b, y, keys[r][1])) << y[0];
                    *state = [old_a, old_b];
                } else {
                    let fa = ((a >> x[0]) & 1) ^ feedback_a(a, x, ir, keys[r][0]);
                    let fb = ((b >> y[0]) & 1) ^ feedback_b(b, y, keys[r][1]);
                    *state = [((a << 1) | fb) & ma, ((b << 1) | fa) & mb];
                }
            }
        }
        let word = (state[0] << len_b) | state[1];
        for (i, b) in block.iter_mut().enumerate() {
            *b = (word >> (8 * (width - 1 - i))) as u8;
        }
    }
    result
}
