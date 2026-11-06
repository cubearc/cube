//! Original twenty-round PRIDE, published external byte representation.
//! Independently implemented from designer Kavun's thesis, Chapter10:
//! involutive bitsliced S-box, four binary matrices, byte-local round keys.
//! No FELICS source or dependency is included; its literals verify the result.
use zeroize::Zeroizing;
const M: [[u16; 16]; 4] = [
    [
        4368, 8736, 17472, 34944, 4353, 8706, 17412, 34824, 4113, 8226, 16452, 32904, 273, 546,
        1092, 2184,
    ],
    [
        8321, 16387, 32774, 268, 536, 1072, 2144, 4288, 12289, 24578, 49156, 33032, 784, 1568,
        3136, 6272,
    ],
    [
        536, 1072, 2144, 4288, 8321, 16387, 32774, 268, 784, 1568, 3136, 6272, 12289, 24578, 49156,
        33032,
    ],
    [
        4113, 8226, 16452, 32904, 273, 546, 1092, 2184, 4368, 8736, 17472, 34944, 4353, 8706,
        17412, 34824,
    ],
];
const MI: [[u16; 16]; 4] = [
    [
        4368, 8736, 17472, 34944, 4353, 8706, 17412, 34824, 4113, 8226, 16452, 32904, 273, 546,
        1092, 2184,
    ],
    [
        1030, 2060, 4120, 8240, 16480, 32960, 385, 515, 12320, 24640, 49280, 33025, 770, 1540,
        3080, 6160,
    ],
    [
        16480, 32960, 385, 515, 1030, 2060, 4120, 8240, 770, 1540, 3080, 6160, 12320, 24640, 49280,
        33025,
    ],
    [
        4113, 8226, 16452, 32904, 273, 546, 1092, 2184, 4368, 8736, 17472, 34944, 4353, 8706,
        17412, 34824,
    ],
];
fn substitution(x: &mut [u16; 4]) {
    let [a, b, c, d] = *x;
    let aa = c ^ (a & b);
    let bb = d ^ (b & c);
    let cc = a ^ (aa & bb);
    let dd = b ^ (bb & cc);
    *x = [aa, bb, cc, dd];
}
fn linear(x: &mut [u16; 4], inverse: bool) {
    let matrices = if inverse { &MI } else { &M };
    for (i, word) in x.iter_mut().enumerate() {
        let before = *word;
        *word = matrices[i].iter().enumerate().fold(0, |v, (bit, row)| {
            v | (((before & row).count_ones() as u16 & 1) << bit)
        });
    }
}
fn round_key(key: &[u8], round: u8) -> Zeroizing<[u16; 4]> {
    let coefficients = [193u8, 165, 81, 197];
    Zeroizing::new(std::array::from_fn(|i| {
        u16::from_le_bytes([
            key[2 * i],
            key[2 * i + 1].wrapping_add(coefficients[i].wrapping_mul(round)),
        ])
    }))
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let white: Zeroizing<[u16; 4]> = Zeroizing::new(std::array::from_fn(|i| {
        u16::from_le_bytes(key[2 * i..2 * i + 2].try_into().unwrap())
    }));
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x: Zeroizing<[u16; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u16::from_le_bytes(block[2 * i..2 * i + 2].try_into().unwrap()) ^ white[i]
        }));
        for n in 0..20 {
            let r = if inverse { 19 - n } else { n };
            let rk = round_key(&key[8..], (r + 1) as u8);
            if inverse {
                if r < 19 {
                    linear(&mut x, true);
                }
                substitution(&mut x);
                for i in 0..4 {
                    x[i] ^= rk[i];
                }
            } else {
                for i in 0..4 {
                    x[i] ^= rk[i];
                }
                substitution(&mut x);
                if r < 19 {
                    linear(&mut x, false);
                }
            }
        }
        for (i, b) in block.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            *b = (x[i] ^ white[i]).to_le_bytes();
        }
    }
    out
}
