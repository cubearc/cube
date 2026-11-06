//! LED full-round 64-/128-bit-key profiles, standard high-first nibbles.
//! Original matrix/constants checked against BSD ImpeccableCircuits 33619f7.
use zeroize::Zeroizing;
const S: [u8; 16] = [12, 5, 6, 11, 9, 0, 10, 13, 3, 14, 15, 8, 4, 7, 1, 2];
const M: [[u8; 4]; 4] = [[4, 1, 2, 2], [8, 6, 5, 6], [11, 14, 10, 9], [2, 2, 15, 11]];
const MI: [[u8; 4]; 4] = [[12, 12, 13, 4], [3, 8, 4, 5], [7, 6, 2, 14], [13, 9, 9, 13]];
fn gf(mut a: u8, mut b: u8) -> u8 {
    let mut out = 0;
    for _ in 0..4 {
        if b & 1 != 0 {
            out ^= a;
        }
        a = ((a << 1) ^ ((a >> 3).wrapping_neg() & 0x13)) & 15;
        b >>= 1;
    }
    out
}
fn mix(x: &mut [u8; 16], inverse: bool) {
    let before = Zeroizing::new(*x);
    let m = if inverse { &MI } else { &M };
    for row in 0..4 {
        for col in 0..4 {
            x[4 * row + col] = (0..4).fold(0, |out, i| out ^ gf(m[row][i], before[4 * i + col]));
        }
    }
}
fn shift(x: &mut [u8; 16], inverse: bool) {
    let old = Zeroizing::new(*x);
    for row in 0..4 {
        for col in 0..4 {
            let from = if inverse {
                (col + 4 - row) % 4
            } else {
                (col + row) % 4
            };
            x[4 * row + col] = old[4 * row + from];
        }
    }
}
fn constant(x: &mut [u8; 16], bits: usize, round: u8) {
    let high = (bits >> 4) as u8;
    let low = (bits & 15) as u8;
    x[0] ^= high;
    x[4] ^= 1 ^ high;
    x[8] ^= 2 ^ low;
    x[12] ^= 3 ^ low;
    x[1] ^= round >> 3;
    x[9] ^= round >> 3;
    x[5] ^= round & 7;
    x[13] ^= round & 7;
}
fn add_key(x: &mut [u8; 16], key: &[u8], step: usize) {
    let start = 16 * (step % (key.len() / 16));
    for (i, out) in x.iter_mut().enumerate() {
        *out ^= key[start + i];
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let rounds = if key.len() == 8 { 32 } else { 48 };
    let mut key_cells = Zeroizing::new(vec![0u8; 2 * key.len()]);
    for (i, b) in key.iter().enumerate() {
        key_cells[2 * i] = b >> 4;
        key_cells[2 * i + 1] = b & 15;
    }
    let mut constants = [0u8; 48];
    let mut c = 1;
    for out in &mut constants {
        *out = c;
        c = ((c << 1) & 63) | (((c >> 5) ^ (c >> 4) ^ 1) & 1);
    }
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x: Zeroizing<[u8; 16]> = Zeroizing::new(std::array::from_fn(|i| {
            if i % 2 == 0 {
                block[i / 2] >> 4
            } else {
                block[i / 2] & 15
            }
        }));
        if inverse {
            add_key(&mut x, &key_cells, rounds / 4);
            for r in (0..rounds).rev() {
                mix(&mut x, true);
                shift(&mut x, true);
                for b in x.iter_mut() {
                    *b = S.iter().position(|v| v == b).unwrap() as u8;
                }
                constant(&mut x, key.len() * 8, constants[r]);
                if r % 4 == 0 {
                    add_key(&mut x, &key_cells, r / 4);
                }
            }
        } else {
            add_key(&mut x, &key_cells, 0);
            for (r, rc) in constants.iter().enumerate().take(rounds) {
                constant(&mut x, key.len() * 8, *rc);
                for b in x.iter_mut() {
                    *b = S[*b as usize];
                }
                shift(&mut x, false);
                mix(&mut x, false);
                if r % 4 == 3 {
                    add_key(&mut x, &key_cells, r / 4 + 1);
                }
            }
        }
        for (i, b) in block.iter_mut().enumerate() {
            *b = (x[2 * i] << 4) | x[2 * i + 1];
        }
    }
    out
}
