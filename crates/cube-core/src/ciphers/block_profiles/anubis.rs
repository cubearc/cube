//! Full-round unaudited educational SPN reference, standard external bytes.
//! Specification arithmetic reviewed against go-cryptobin e938a803 (Apache-2.0).
//! Expected bytes are independently published; no constant-time claim.
use zeroize::Zeroizing;
pub(super) const ORIGINAL: [u8; 256] = [
    186, 84, 47, 116, 83, 211, 210, 77, 80, 172, 141, 191, 112, 82, 154, 76, 234, 213, 151, 209,
    51, 81, 91, 166, 222, 72, 168, 153, 219, 50, 183, 252, 227, 158, 145, 155, 226, 187, 65, 110,
    165, 203, 107, 149, 161, 243, 177, 2, 204, 196, 29, 20, 195, 99, 218, 93, 95, 220, 125, 205,
    127, 90, 108, 92, 247, 38, 255, 237, 232, 157, 111, 142, 25, 160, 240, 137, 15, 7, 175, 251, 8,
    21, 13, 4, 1, 100, 223, 118, 121, 221, 61, 22, 63, 55, 109, 56, 185, 115, 233, 53, 85, 113,
    123, 140, 114, 136, 246, 42, 62, 94, 39, 70, 12, 101, 104, 97, 3, 193, 87, 214, 217, 88, 216,
    102, 215, 58, 200, 60, 250, 150, 167, 152, 236, 184, 199, 174, 105, 75, 171, 169, 103, 10, 71,
    242, 181, 34, 229, 238, 190, 43, 129, 18, 131, 27, 14, 35, 245, 69, 33, 206, 73, 44, 249, 230,
    182, 40, 23, 130, 26, 139, 254, 138, 9, 201, 135, 78, 225, 46, 228, 224, 235, 144, 164, 30,
    133, 96, 0, 37, 244, 241, 148, 11, 231, 117, 239, 52, 49, 212, 208, 134, 126, 173, 253, 41, 48,
    59, 159, 248, 198, 19, 6, 5, 197, 17, 119, 124, 122, 120, 54, 28, 57, 89, 24, 86, 179, 176, 36,
    32, 178, 146, 163, 192, 68, 98, 16, 180, 132, 67, 147, 194, 74, 189, 143, 45, 188, 156, 106,
    64, 207, 162, 128, 79, 31, 202, 170, 66,
];
pub(super) const TWEAKED: [u8; 256] = [
    167, 211, 230, 113, 208, 172, 77, 121, 58, 201, 145, 252, 30, 71, 84, 189, 140, 165, 122, 251,
    99, 184, 221, 212, 229, 179, 197, 190, 169, 136, 12, 162, 57, 223, 41, 218, 43, 168, 203, 76,
    75, 34, 170, 36, 65, 112, 166, 249, 90, 226, 176, 54, 125, 228, 51, 255, 96, 32, 8, 139, 94,
    171, 127, 120, 124, 44, 87, 210, 220, 109, 126, 13, 83, 148, 195, 40, 39, 6, 95, 173, 103, 92,
    85, 72, 14, 82, 234, 66, 91, 93, 48, 88, 81, 89, 60, 78, 56, 138, 114, 20, 231, 198, 222, 80,
    142, 146, 209, 119, 147, 69, 154, 206, 45, 3, 98, 182, 185, 191, 150, 107, 63, 7, 18, 174, 64,
    52, 70, 62, 219, 207, 236, 204, 193, 161, 192, 214, 29, 244, 97, 59, 16, 216, 104, 160, 177,
    10, 105, 108, 73, 250, 118, 196, 158, 155, 110, 153, 194, 183, 152, 188, 143, 133, 31, 180,
    248, 17, 46, 0, 37, 28, 42, 61, 5, 79, 123, 178, 50, 144, 175, 25, 163, 247, 115, 157, 21, 116,
    238, 202, 159, 15, 27, 117, 134, 132, 156, 74, 151, 26, 101, 246, 237, 9, 187, 38, 131, 235,
    111, 129, 4, 106, 67, 1, 23, 225, 135, 245, 141, 227, 35, 128, 68, 22, 102, 33, 254, 213, 49,
    217, 53, 24, 2, 100, 242, 241, 86, 205, 130, 200, 186, 240, 239, 233, 232, 253, 137, 215, 199,
    181, 164, 47, 149, 19, 11, 243, 224, 55,
];
use super::finite_field::gf;
const MATRIX: [[u8; 4]; 4] = [[1, 2, 4, 6], [2, 1, 6, 4], [4, 6, 1, 2], [6, 4, 2, 1]];
const fn tables(sbox: &[u8; 256]) -> [[u32; 256]; 4] {
    let mut out = [[0; 256]; 4];
    let mut col = 0;
    while col < 4 {
        let mut x = 0;
        while x < 256 {
            let mut row = 0;
            let mut word = 0;
            while row < 4 {
                word = (word << 8) | (gf(sbox[x], MATRIX[row][col], 0x1d) as u32);
                row += 1;
            }
            out[col][x] = word;
            x += 1;
        }
        col += 1;
    }
    out
}
const ORIGINAL_T: [[u32; 256]; 4] = tables(&ORIGINAL);
const TWEAKED_T: [[u32; 256]; 4] = tables(&TWEAKED);
fn byte(word: u32, index: usize) -> usize {
    ((word >> (24 - 8 * index)) & 255) as usize
}
fn mix(word: u32, sbox: &[u8; 256], t: &[[u32; 256]; 4]) -> u32 {
    let mut out = 0;
    for (i, row) in t.iter().enumerate() {
        out ^= row[sbox[byte(word, i)] as usize];
    }
    out
}
fn schedule(
    key: &[u8],
    sbox: &[u8; 256],
    t: &[[u32; 256]; 4],
    decrypting: bool,
) -> Zeroizing<Vec<[u32; 4]>> {
    assert!((16..=40).contains(&key.len()) && key.len().is_multiple_of(4));
    let n = key.len() / 4;
    let rounds = 8 + n;
    let mut kappa = Zeroizing::new(
        key.as_chunks::<4>()
            .0
            .iter()
            .map(|b| u32::from_be_bytes(*b))
            .collect::<Vec<_>>(),
    );
    let mut keys = Zeroizing::new(vec![[0u32; 4]; rounds + 1]);
    for (r, row) in keys.iter_mut().enumerate() {
        let mut powers = [1u8; 4];
        for word in kappa.iter() {
            for (j, out) in row.iter_mut().enumerate() {
                let s = sbox[byte(*word, j)];
                for (index, power) in powers.iter().enumerate() {
                    *out ^= (gf(s, *power, 0x1d) as u32) << (24 - 8 * index);
                }
            }
            for (power, base) in powers.iter_mut().zip([1, 2, 6, 8]) {
                *power = gf(*power, base, 0x1d);
            }
        }
        if r == rounds {
            break;
        }
        let mut next = Zeroizing::new(vec![0u32; n]);
        for (i, word) in next.iter_mut().enumerate() {
            for (j, table) in t.iter().enumerate() {
                *word ^= table[byte(kappa[(n + i - j) % n], j)];
            }
        }
        next[0] ^= u32::from_be_bytes(sbox[4 * r..4 * r + 4].try_into().unwrap());
        kappa.copy_from_slice(&next);
    }
    if decrypting {
        keys.reverse();
        for row in &mut keys[1..rounds] {
            for word in row {
                *word = mix(*word, sbox, t);
            }
        }
    }
    keys
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool, tweaked: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(16));
    let (sbox, t) = if tweaked {
        (&TWEAKED, &TWEAKED_T)
    } else {
        (&ORIGINAL, &ORIGINAL_T)
    };
    let keys = schedule(key, sbox, t, decrypting);
    let rounds = keys.len() - 1;
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<16>().0 {
        let mut state: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_be_bytes(block[4 * i..4 * i + 4].try_into().unwrap()) ^ keys[0][i]
        }));
        for (r, key) in keys.iter().enumerate().skip(1) {
            let next = Zeroizing::new(std::array::from_fn(|j| {
                let mut word = key[j];
                for (i, table) in t.iter().enumerate() {
                    let value = table[byte(state[i], j)];
                    word ^= if r == rounds {
                        value & (0xff000000u32 >> (8 * i))
                    } else {
                        value
                    };
                }
                word
            }));
            *state = *next;
        }
        for (word, bytes) in state.iter().zip(block.as_chunks_mut::<4>().0) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
    }
    output
}
