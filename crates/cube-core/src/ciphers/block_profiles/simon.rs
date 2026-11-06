//! Original SIMON, every full-round block/key tuple, paper BE word ordering.
//! Public specification and CC0 designer material; independently checked MIT
//! McCoy9eec981 known answers. Unaudited educational reference in safe Rust.
use zeroize::Zeroizing;
const Z: [u64; 5] = [
    0b01100111000011010100100010111110110011100001101010010001011111,
    0b01011010000110010011111011100010101101000011001001111101110001,
    0b11001101101001111110001000010100011001001011000000111011110101,
    0b11110000101100111001010001001000000111101001100011010111011011,
    0b11110111001001010011000011101000000100011011010110011110001011,
];
fn setup(width: usize, key: usize) -> (usize, usize) {
    match (width * 8, key * 8) {
        (32, 64) => (32, 0),
        (48, 72) => (36, 0),
        (48, 96) => (36, 1),
        (64, 96) => (42, 2),
        (64, 128) => (44, 3),
        (96, 96) => (52, 2),
        (96, 144) => (54, 3),
        (128, 128) => (68, 2),
        (128, 192) => (69, 3),
        (128, 256) => (72, 4),
        _ => unreachable!("validated tuple"),
    }
}
fn read(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |word, b| (word << 8) | u64::from(*b))
}
fn rotl(word: u64, amount: u32, bits: u32, mask: u64) -> u64 {
    ((word << amount) | (word >> (bits - amount))) & mask
}
fn rotr(word: u64, amount: u32, bits: u32, mask: u64) -> u64 {
    ((word >> amount) | (word << (bits - amount))) & mask
}
fn nonlinear(word: u64, bits: u32, mask: u64) -> u64 {
    (rotl(word, 1, bits, mask) & rotl(word, 8, bits, mask)) ^ rotl(word, 2, bits, mask)
}
pub(super) fn transform(data: &[u8], key: &[u8], width: usize, decrypting: bool) -> Vec<u8> {
    let (rounds, z) = setup(width, key.len());
    assert!(data.len().is_multiple_of(width));
    let word_bytes = width / 2;
    let bits = (word_bytes * 8) as u32;
    let mask = u64::MAX >> (64 - bits);
    let m = key.len() / word_bytes;
    let mut keys = Zeroizing::new(vec![0u64; rounds]);
    for (slot, bytes) in keys.iter_mut().zip(key.chunks_exact(word_bytes).rev()) {
        *slot = read(bytes);
    }
    for i in m..rounds {
        let mut temp = rotr(keys[i - 1], 3, bits, mask);
        if m == 4 {
            temp ^= keys[i - 3];
        }
        temp ^= rotr(temp, 1, bits, mask);
        keys[i] = (mask ^ 3) ^ keys[i - m] ^ temp ^ ((Z[z] >> ((i - m) % 62)) & 1);
    }
    let mut output = data.to_vec();
    for block in output.chunks_exact_mut(width) {
        let mut state = Zeroizing::new([read(&block[..word_bytes]), read(&block[word_bytes..])]);
        for i in 0..rounds {
            let [x, y] = *state;
            *state = if decrypting {
                [y, x ^ nonlinear(y, bits, mask) ^ keys[rounds - 1 - i]]
            } else {
                [y ^ nonlinear(x, bits, mask) ^ keys[i], x]
            };
        }
        block[..word_bytes].copy_from_slice(&state[0].to_be_bytes()[8 - word_bytes..]);
        block[word_bytes..].copy_from_slice(&state[1].to_be_bytes()[8 - word_bytes..]);
    }
    output
}
