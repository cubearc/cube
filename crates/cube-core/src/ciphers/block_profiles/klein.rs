//! KLEIN-64/80/96, full original 12/16/20-round profiles.
//! Original paper Appendix A and MIT huljar/klein-vhdl 59cb3a0 qualify math.
use super::finite_field::gf;
use zeroize::Zeroizing;
const S: [u8; 16] = [7, 4, 10, 9, 1, 15, 11, 0, 12, 3, 2, 6, 8, 14, 13, 5];
fn sub(x: u8) -> u8 {
    (S[(x >> 4) as usize] << 4) | S[(x & 15) as usize]
}
fn schedule(key: &[u8], rounds: usize) -> Zeroizing<Vec<[u8; 8]>> {
    let half = key.len() / 2;
    let mut state = Zeroizing::new(key.to_vec());
    let mut keys = Zeroizing::new(Vec::with_capacity(rounds + 1));
    for r in 0..=rounds {
        keys.push(state[..8].try_into().unwrap());
        if r == rounds {
            break;
        }
        state[..half].rotate_left(1);
        state[half..].rotate_left(1);
        let old = Zeroizing::new(state.to_vec());
        for i in 0..half {
            state[i] = old[half + i];
            state[half + i] = old[i] ^ old[half + i];
        }
        state[2] ^= (r + 1) as u8;
        state[half + 1] = sub(state[half + 1]);
        state[half + 2] = sub(state[half + 2]);
    }
    keys
}
fn mix(x: &mut [u8; 8], inverse: bool) {
    let first = if inverse {
        [14, 11, 13, 9]
    } else {
        [2, 3, 1, 1]
    };
    for column in x.as_chunks_mut::<4>().0 {
        let old = Zeroizing::new(*column);
        for row in 0..4 {
            column[row] = (0..4).fold(0, |v, col| {
                v ^ gf(old[col], first[(col + 4 - row) % 4], 0x1b)
            });
        }
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let rounds = match key.len() {
        8 => 12,
        10 => 16,
        12 => 20,
        _ => unreachable!("validated key"),
    };
    let keys = schedule(key, rounds);
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = Zeroizing::new(*block);
        if inverse {
            for i in 0..8 {
                x[i] ^= keys[rounds][i];
            }
            for r in (0..rounds).rev() {
                mix(&mut x, true);
                x.rotate_right(2);
                for i in 0..8 {
                    x[i] = sub(x[i]) ^ keys[r][i];
                }
            }
        } else {
            for r in 0..rounds {
                for i in 0..8 {
                    x[i] = sub(x[i] ^ keys[r][i]);
                }
                x.rotate_left(2);
                mix(&mut x, false);
            }
            for i in 0..8 {
                x[i] ^= keys[rounds][i];
            }
        }
        *block = *x;
    }
    out
}
