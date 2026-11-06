//! Full-round unaudited educational SPN reference, standard external bytes.
//! Specification arithmetic reviewed against go-cryptobin e938a803 (Apache-2.0).
//! Expected bytes are independently published; no constant-time claim.
use super::anubis::ORIGINAL as SBOX;
use super::finite_field::gf;
use zeroize::Zeroizing;
const COEFFICIENTS: [u8; 8] = [1, 3, 4, 5, 6, 8, 11, 7];
const fn tables() -> [[u64; 256]; 8] {
    let mut out = [[0; 256]; 8];
    let mut col = 0;
    while col < 8 {
        let mut x = 0;
        while x < 256 {
            let mut row = 0;
            let mut word = 0;
            while row < 8 {
                word = (word << 8) | (gf(SBOX[x], COEFFICIENTS[row ^ col], 0x1d) as u64);
                row += 1;
            }
            out[col][x] = word;
            x += 1;
        }
        col += 1;
    }
    out
}
const TABLES: [[u64; 256]; 8] = tables();
fn substitute(word: u64) -> u64 {
    u64::from_be_bytes(word.to_be_bytes().map(|b| SBOX[b as usize]))
}
fn layer(word: u64) -> u64 {
    word.to_be_bytes()
        .into_iter()
        .zip(TABLES.iter())
        .fold(0, |sum, (b, table)| sum ^ table[b as usize])
}
fn schedule(key: &[u8], decrypting: bool) -> Zeroizing<[u64; 9]> {
    assert_eq!(key.len(), 16);
    let mut pair = Zeroizing::new([
        u64::from_be_bytes(key[..8].try_into().unwrap()),
        u64::from_be_bytes(key[8..].try_into().unwrap()),
    ]);
    let mut keys = Zeroizing::new([0u64; 9]);
    for (i, row) in keys.iter_mut().enumerate() {
        *row = layer(pair[1])
            ^ u64::from_be_bytes(SBOX[8 * i..8 * i + 8].try_into().unwrap())
            ^ pair[0];
        pair[0] = pair[1];
        pair[1] = *row;
    }
    if decrypting {
        keys.reverse();
        for word in &mut keys[1..8] {
            *word = layer(substitute(*word));
        }
    }
    keys
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(8));
    let keys = schedule(key, decrypting);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut state = Zeroizing::new(u64::from_be_bytes(*block) ^ keys[0]);
        for key in &keys[1..8] {
            *state = layer(*state) ^ key;
        }
        *state = substitute(*state) ^ keys[8];
        *block = state.to_be_bytes();
    }
    output
}
