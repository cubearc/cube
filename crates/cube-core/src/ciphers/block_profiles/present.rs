//! PRESENT-80/-128, 31 rounds plus final whitening, big-endian bytes.
//! Original CHES specification; unauthenticated reference with S-box lookups.
use zeroize::Zeroizing;
const SBOX: [u8; 16] = [12, 5, 6, 11, 9, 0, 10, 13, 3, 14, 15, 8, 4, 7, 1, 2];
const fn inverse() -> [u8; 16] {
    let mut table = [0; 16];
    let mut i = 0;
    while i < 16 {
        table[SBOX[i] as usize] = i as u8;
        i += 1;
    }
    table
}
const INVERSE: [u8; 16] = inverse();
fn schedule(key: &[u8]) -> Zeroizing<[u64; 32]> {
    assert!([10, 16].contains(&key.len()));
    let bits = key.len() * 8;
    let mut packed = Zeroizing::new([0u8; 16]);
    packed[16 - key.len()..].copy_from_slice(key);
    let mut key = Zeroizing::new(u128::from_be_bytes(*packed));
    let mut keys = Zeroizing::new([0; 32]);
    for (index, row) in keys.iter_mut().enumerate() {
        *row = (*key >> (bits - 64)) as u64;
        if index == 31 {
            break;
        }
        if bits == 80 {
            *key = ((*key << 61) | (*key >> 19)) & ((1u128 << 80) - 1);
            *key = (*key & !(15u128 << 76)) | ((SBOX[(*key >> 76) as usize] as u128) << 76);
            *key ^= ((index + 1) as u128) << 15;
        } else {
            *key = key.rotate_left(61);
            let high = SBOX[(*key >> 124) as usize] as u128;
            let low = SBOX[(((*key) >> 120) & 15) as usize] as u128;
            *key = (*key & !(255u128 << 120)) | (high << 124) | (low << 120);
            *key ^= ((index + 1) as u128) << 62;
        }
    }
    keys
}
fn substitute(input: u64, inverse: bool) -> u64 {
    let table = if inverse { &INVERSE } else { &SBOX };
    let mut output = 0;
    for nibble in 0..16 {
        output |= (table[((input >> (4 * nibble)) & 15) as usize] as u64) << (4 * nibble);
    }
    output
}
fn permute(input: u64, inverse: bool) -> u64 {
    let mut output = 0;
    let factor = if inverse { 4 } else { 16 };
    for bit in 0..64 {
        let dest = if bit == 63 { 63 } else { (bit * factor) % 63 };
        output |= ((input >> bit) & 1) << dest;
    }
    output
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(8));
    let keys = schedule(key);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut state = Zeroizing::new(u64::from_be_bytes(*block));
        if decrypting {
            *state ^= keys[31];
            for key in keys[..31].iter().rev() {
                *state = substitute(permute(*state, true), true) ^ *key;
            }
        } else {
            for key in keys[..31].iter() {
                *state = permute(substitute(*state ^ *key, false), false);
            }
            *state ^= keys[31];
        }
        block.copy_from_slice(&state.to_be_bytes());
    }
    output
}
