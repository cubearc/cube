//! Original Rijndael reference: all 128/160/192/224/256-bit block/key widths.
//! Unauthenticated, unaudited and not a constant-time backend.
//! Column-major bytes and Nb-dependent row shifts follow the original proposal.
use zeroize::Zeroizing;

const fn xtime(value: u8) -> u8 {
    (value << 1) ^ ((value >> 7).wrapping_neg() & 0x1b)
}
const fn multiply(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0;
    let mut i = 0;
    while i < 8 {
        if b & 1 != 0 {
            result ^= a;
        }
        a = xtime(a);
        b >>= 1;
        i += 1;
    }
    result
}
const fn substitution() -> [u8; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut value = i as u8;
        let mut exponent = 254;
        let mut inverse = 1;
        while exponent > 0 {
            if exponent & 1 != 0 {
                inverse = multiply(inverse, value);
            }
            value = multiply(value, value);
            exponent >>= 1;
        }
        table[i] = inverse
            ^ inverse.rotate_left(1)
            ^ inverse.rotate_left(2)
            ^ inverse.rotate_left(3)
            ^ inverse.rotate_left(4)
            ^ 0x63;
        i += 1;
    }
    table
}
pub(crate) const SBOX: [u8; 256] = substitution();
const fn inverse_substitution() -> [u8; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[SBOX[i] as usize] = i as u8;
        i += 1;
    }
    table
}
const INVERSE: [u8; 256] = inverse_substitution();
struct Schedule {
    words: Zeroizing<Vec<[u8; 4]>>,
    columns: usize,
    rounds: usize,
}
impl Schedule {
    fn new(key: &[u8], width: usize) -> Self {
        assert!([16, 20, 24, 28, 32].contains(&key.len()));
        assert!([16, 20, 24, 28, 32].contains(&width));
        let columns = width / 4;
        let key_words = key.len() / 4;
        let rounds = columns.max(key_words) + 6;
        let mut words = Zeroizing::new(vec![[0u8; 4]; columns * (rounds + 1)]);
        for (index, word) in key.as_chunks::<4>().0.iter().enumerate() {
            words[index] = *word;
        }
        let mut round_constant = 1;
        for index in key_words..words.len() {
            let mut word = words[index - 1];
            if index % key_words == 0 {
                word.rotate_left(1);
                for b in &mut word {
                    *b = SBOX[*b as usize];
                }
                word[0] ^= round_constant;
                round_constant = xtime(round_constant);
            } else if key_words > 6 && index % key_words == 4 {
                for b in &mut word {
                    *b = SBOX[*b as usize];
                }
            }
            for j in 0..4 {
                words[index][j] = words[index - key_words][j] ^ word[j];
            }
        }
        Self {
            words,
            columns,
            rounds,
        }
    }
    fn add_key(&self, state: &mut [u8], round: usize) {
        for (column, word) in state.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            for (byte, key_byte) in word
                .iter_mut()
                .zip(self.words[round * self.columns + column])
            {
                *byte ^= key_byte;
            }
        }
    }
    fn shifts(&self, state: &mut [u8], inverse: bool) {
        let offsets = match self.columns {
            7 => [0, 1, 2, 4],
            8 => [0, 1, 3, 4],
            _ => [0, 1, 2, 3],
        };
        let mut before = Zeroizing::new([0u8; 32]);
        before[..state.len()].copy_from_slice(state);
        for row in 1..4 {
            for column in 0..self.columns {
                let offset = if inverse {
                    self.columns - offsets[row]
                } else {
                    offsets[row]
                };
                state[4 * column + row] = before[4 * ((column + offset) % self.columns) + row];
            }
        }
    }
    fn apply(&self, state: &mut [u8], decrypting: bool) {
        if decrypting {
            self.add_key(state, self.rounds);
            for round in (0..self.rounds).rev() {
                self.shifts(state, true);
                for b in state.iter_mut() {
                    *b = INVERSE[*b as usize];
                }
                self.add_key(state, round);
                if round != 0 {
                    mix_columns(state, true);
                }
            }
        } else {
            self.add_key(state, 0);
            for round in 1..=self.rounds {
                for b in state.iter_mut() {
                    *b = SBOX[*b as usize];
                }
                self.shifts(state, false);
                if round != self.rounds {
                    mix_columns(state, false);
                }
                self.add_key(state, round);
            }
        }
    }
}
fn mix_columns(state: &mut [u8], inverse: bool) {
    for word in state.as_chunks_mut::<4>().0 {
        if inverse {
            let a = xtime(xtime(word[0] ^ word[2]));
            let b = xtime(xtime(word[1] ^ word[3]));
            word[0] ^= a;
            word[2] ^= a;
            word[1] ^= b;
            word[3] ^= b;
        }
        let [a, b, c, d] = *word;
        let sum = a ^ b ^ c ^ d;
        *word = [
            a ^ sum ^ xtime(a ^ b),
            b ^ sum ^ xtime(b ^ c),
            c ^ sum ^ xtime(c ^ d),
            d ^ sum ^ xtime(d ^ a),
        ];
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], width: usize, decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(width));
    let schedule = Schedule::new(key, width);
    let mut output = data.to_vec();
    for block in output.chunks_exact_mut(width) {
        schedule.apply(block, decrypting);
    }
    output
}

/// LEX v2's full AES128 initialization and ten leaky rounds per40 bytes.
/// The eSTREAM wrapper uses LE IV/output words and BE key words.
pub(crate) fn lex128(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let schedule = Schedule::new(key, 16);
    let mut state = Zeroizing::new([0u8; 16]);
    state.copy_from_slice(iv);
    for word in state.as_chunks_mut::<4>().0 {
        word.reverse();
    }
    schedule.apply(&mut state[..], false);
    for chunk in output.chunks_mut(40) {
        for (round, bytes) in (1..=10).zip(chunk.chunks_mut(4)) {
            for b in state.iter_mut() {
                *b = SBOX[*b as usize];
            }
            schedule.shifts(&mut state[..], false);
            mix_columns(&mut state[..], false);
            schedule.add_key(&mut state[..], round);
            let indices = if round & 1 != 0 {
                [10, 2, 8, 0]
            } else {
                [11, 3, 9, 1]
            };
            for (b, i) in bytes.iter_mut().zip(indices) {
                *b ^= state[i];
            }
        }
    }
}
