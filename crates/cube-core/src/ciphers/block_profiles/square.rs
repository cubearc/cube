//! Full-round unaudited educational SPN reference, standard external bytes.
//! Specification arithmetic reviewed against go-cryptobin e938a803 (Apache-2.0).
//! Expected bytes are independently published; no constant-time claim.
use zeroize::Zeroizing;
pub(super) const SBOX: [u8; 256] = [
    177, 206, 195, 149, 90, 173, 231, 2, 77, 68, 251, 145, 12, 135, 161, 80, 203, 103, 84, 221, 70,
    143, 225, 78, 240, 253, 252, 235, 249, 196, 26, 110, 94, 245, 204, 141, 28, 86, 67, 254, 7, 97,
    248, 117, 89, 255, 3, 34, 138, 209, 19, 238, 136, 0, 14, 52, 21, 128, 148, 227, 237, 181, 83,
    35, 75, 71, 23, 167, 144, 53, 171, 216, 184, 223, 79, 87, 154, 146, 219, 27, 60, 200, 153, 4,
    142, 224, 215, 125, 133, 187, 64, 44, 58, 69, 241, 66, 101, 32, 65, 24, 114, 37, 147, 112, 54,
    5, 242, 11, 163, 121, 236, 8, 39, 49, 50, 182, 124, 176, 10, 115, 91, 123, 183, 129, 210, 13,
    106, 38, 158, 88, 156, 131, 116, 179, 172, 48, 122, 105, 119, 15, 174, 33, 222, 208, 46, 151,
    16, 164, 152, 168, 212, 104, 45, 98, 41, 109, 22, 73, 118, 199, 232, 193, 150, 55, 229, 202,
    244, 233, 99, 18, 194, 166, 20, 188, 211, 40, 175, 47, 230, 36, 82, 198, 160, 9, 189, 140, 207,
    93, 17, 95, 1, 197, 159, 61, 162, 155, 201, 59, 190, 81, 25, 31, 63, 92, 178, 239, 74, 205,
    191, 186, 111, 100, 217, 243, 62, 180, 170, 220, 213, 6, 192, 126, 246, 102, 108, 132, 113, 56,
    185, 29, 127, 157, 72, 139, 42, 218, 165, 51, 130, 57, 214, 120, 134, 250, 228, 43, 169, 30,
    137, 96, 107, 234, 85, 76, 247, 226,
];
use super::finite_field::gf;
const fn inverse() -> [u8; 256] {
    let mut out = [0; 256];
    let mut i = 0;
    while i < 256 {
        out[SBOX[i] as usize] = i as u8;
        i += 1;
    }
    out
}
const INVERSE: [u8; 256] = inverse();
const fn table(sbox: &[u8; 256], coefficients: [u8; 4]) -> [u32; 256] {
    let mut out = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut word = 0;
        let mut row = 0;
        while row < 4 {
            word = (word << 8) | (gf(sbox[i], coefficients[row], 0xf5) as u32);
            row += 1;
        }
        out[i] = word;
        i += 1;
    }
    out
}
const FORWARD: [u32; 256] = table(&SBOX, [2, 1, 1, 3]);
const REVERSE: [u32; 256] = table(&INVERSE, [14, 9, 13, 11]);
fn theta(word: u32) -> u32 {
    word.to_be_bytes()
        .into_iter()
        .enumerate()
        .fold(0, |sum, (i, b)| {
            let value = u32::from_be_bytes([gf(b, 2, 0xf5), b, b, gf(b, 3, 0xf5)]);
            sum ^ value.rotate_right((8 * i) as u32)
        })
}
fn schedule(key: &[u8], decrypting: bool) -> Zeroizing<[[u32; 4]; 9]> {
    assert_eq!(key.len(), 16);
    let mut raw = Zeroizing::new([[0u32; 4]; 9]);
    raw[0] = std::array::from_fn(|i| u32::from_be_bytes(key[4 * i..4 * i + 4].try_into().unwrap()));
    for r in 1..=8 {
        raw[r][0] = raw[r - 1][0] ^ raw[r - 1][3].rotate_left(8) ^ (1u32 << (23 + r));
        for i in 1..4 {
            raw[r][i] = raw[r - 1][i] ^ raw[r][i - 1];
        }
    }
    if decrypting {
        raw.reverse();
        for word in &mut raw[8] {
            *word = theta(*word);
        }
    } else {
        for row in &mut raw[..8] {
            for word in row {
                *word = theta(*word);
            }
        }
    }
    raw
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(16));
    let keys = schedule(key, decrypting);
    let (sbox, table) = if decrypting {
        (&INVERSE, &REVERSE)
    } else {
        (&SBOX, &FORWARD)
    };
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<16>().0 {
        let mut state: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_be_bytes(block[4 * i..4 * i + 4].try_into().unwrap()) ^ keys[0][i]
        }));
        for (r, key) in keys.iter().enumerate().skip(1) {
            let next = Zeroizing::new(std::array::from_fn(|j| {
                let mut word = key[j];
                for (i, previous) in state.iter().enumerate() {
                    let b = ((previous >> (24 - 8 * j)) & 255) as usize;
                    word ^= if r == 8 {
                        (sbox[b] as u32) << (24 - 8 * i)
                    } else {
                        table[b].rotate_right((8 * i) as u32)
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
