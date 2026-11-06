//! Full 32-round CRAFT, required explicit tweak, paper row-major nibble order.
//! Math and KATs checked against designer BSD hardware, 33619f7.
use zeroize::Zeroizing;
const S: [u8; 16] = [12, 10, 13, 3, 14, 11, 15, 7, 8, 9, 1, 5, 0, 2, 4, 6];
const P: [usize; 16] = [15, 12, 13, 14, 10, 9, 8, 11, 6, 5, 4, 7, 1, 2, 3, 0];
const Q: [usize; 16] = [12, 10, 15, 5, 14, 8, 9, 2, 11, 3, 7, 4, 6, 0, 1, 13];
fn cells(bytes: &[u8]) -> [u8; 16] {
    std::array::from_fn(|i| (bytes[i / 2] >> (4 * (1 - i % 2))) & 15)
}
fn mix(x: &mut [u8; 16]) {
    for c in 0..4 {
        x[c] ^= x[c + 8] ^ x[c + 12];
        x[c + 4] ^= x[c + 12];
    }
}
fn substitute_permute(x: &mut [u8; 16]) {
    let old = *x;
    for (i, s) in x.iter_mut().enumerate() {
        *s = S[old[P[i]] as usize];
    }
}
fn constants() -> [[u8; 2]; 32] {
    let mut a = 1u8;
    let mut b = 1u8;
    std::array::from_fn(|_| {
        let out = [a, b];
        a = (a >> 1) | (((a ^ (a >> 1)) & 1) << 3);
        b = (b >> 1) | (((b ^ (b >> 1)) & 1) << 2);
        out
    })
}
pub(super) fn transform(data: &[u8], key: &[u8], tweak: &[u8], inverse: bool) -> Vec<u8> {
    let t = Zeroizing::new(cells(tweak));
    let mut tk = Zeroizing::new([[0u8; 16]; 4]);
    for j in 0..4 {
        let k = Zeroizing::new(cells(&key[8 * (j % 2)..8 * (j % 2) + 8]));
        for i in 0..16 {
            tk[j][i] = k[i] ^ t[if j < 2 { i } else { Q[i] }];
        }
    }
    let rc = constants();
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = Zeroizing::new(cells(block));
        if inverse {
            for r in (0..32).rev() {
                if r < 31 {
                    substitute_permute(&mut x);
                }
                for i in 0..16 {
                    x[i] ^= tk[r % 4][i];
                }
                x[4] ^= rc[r][0];
                x[5] ^= rc[r][1];
                mix(&mut x);
            }
        } else {
            for r in 0..32 {
                mix(&mut x);
                x[4] ^= rc[r][0];
                x[5] ^= rc[r][1];
                for i in 0..16 {
                    x[i] ^= tk[r % 4][i];
                }
                if r < 31 {
                    substitute_permute(&mut x);
                }
            }
        }
        for (i, b) in block.iter_mut().enumerate() {
            *b = (x[2 * i] << 4) | x[2 * i + 1];
        }
    }
    out
}
