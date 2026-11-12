//! BluetoothCore5.4 Vol2H4.4–4.6 session128 raw E0, full two-level setup.
//! Key, address and CL octets follow specification array order; dataLSB-first.
use zeroize::Zeroizing;
const LENGTHS: [u32; 4] = [25, 31, 33, 39];
const TAPS: [u64; 4] = [
    (1 << 24) | (1 << 19) | (1 << 11) | (1 << 7),
    (1 << 30) | (1 << 23) | (1 << 15) | (1 << 11),
    (1 << 32) | (1 << 27) | (1 << 23) | (1 << 3),
    (1 << 38) | (1 << 35) | (1 << 27) | (1 << 3),
];
const OUTPUT: [u32; 4] = [23, 23, 31, 31];
fn symbols(r: &[u64; 4]) -> [u8; 4] {
    std::array::from_fn(|i| ((r[i] >> OUTPUT[i]) & 1) as u8)
}
fn value(r: &[u64; 4], c: &[u8; 2]) -> u8 {
    symbols(r).iter().fold(c[0] & 1, |v, b| v ^ b)
}
fn blend(r: &[u64; 4], c: &mut [u8; 2]) {
    let sum: u8 = symbols(r).iter().sum();
    let next = ((sum + c[0]) >> 1) ^ c[0] ^ [0, 3, 1, 2][c[1] as usize];
    c[1] = c[0];
    c[0] = next;
}
fn shift(r: &mut [u64; 4], inputs: &mut [u64; 4], t: u32) {
    for i in 0..4 {
        let feedback = if t > LENGTHS[i] {
            u64::from((r[i] & TAPS[i]).count_ones() & 1)
        } else {
            0
        };
        r[i] = ((r[i] << 1) & ((1u64 << LENGTHS[i]) - 1)) ^ feedback ^ (inputs[i] & 1);
        inputs[i] >>= 1;
    }
}
fn initialize(key: &[u8], nonce: &[u8]) -> (Zeroizing<[u64; 4]>, Zeroizing<[u8; 2]>) {
    let a = &nonce[..6];
    let cl = &nonce[6..];
    let mut inputs = Zeroizing::new([
        u64::from(cl[3] & 1)
            | (u64::from(key[0]) << 1)
            | (u64::from(key[4]) << 9)
            | (u64::from(key[8]) << 17)
            | (u64::from(key[12]) << 25)
            | (u64::from(cl[1]) << 33)
            | (u64::from(a[2]) << 41),
        1 | (u64::from(cl[0] & 15) << 3)
            | (u64::from(key[1]) << 7)
            | (u64::from(key[5]) << 15)
            | (u64::from(key[9]) << 23)
            | (u64::from(key[13]) << 31)
            | (u64::from(a[0]) << 39)
            | (u64::from(a[3]) << 47),
        u64::from((cl[3] >> 1) & 1)
            | (u64::from(key[2]) << 1)
            | (u64::from(key[6]) << 9)
            | (u64::from(key[10]) << 17)
            | (u64::from(key[14]) << 25)
            | (u64::from(cl[2]) << 33)
            | (u64::from(a[4]) << 41),
        7 | (u64::from(cl[0] >> 4) << 3)
            | (u64::from(key[3]) << 7)
            | (u64::from(key[7]) << 15)
            | (u64::from(key[11]) << 23)
            | (u64::from(key[15]) << 31)
            | (u64::from(a[1]) << 39)
            | (u64::from(a[5]) << 47),
    ]);
    let mut r = Zeroizing::new([0u64; 4]);
    let mut c = Zeroizing::new([0u8; 2]);
    let mut z = Zeroizing::new([0u8; 16]);
    for t in 1..240 {
        if t > 39 {
            blend(&r, &mut c);
        }
        shift(&mut r, &mut inputs, t);
        if t >= 112 {
            let n = (t - 112) as usize;
            z[n / 8] |= value(&r, &c) << (n % 8);
        }
    }
    r[0] = u64::from(z[0])
        | (u64::from(z[4]) << 8)
        | (u64::from(z[8]) << 16)
        | (u64::from(z[12] & 1) << 24);
    r[1] = u64::from(z[1])
        | (u64::from(z[5]) << 8)
        | (u64::from(z[9]) << 16)
        | (u64::from(z[12] >> 1) << 24);
    r[2] = u64::from(z[2])
        | (u64::from(z[6]) << 8)
        | (u64::from(z[10]) << 16)
        | (u64::from(z[13]) << 24)
        | (u64::from(z[15] & 1) << 32);
    r[3] = u64::from(z[3])
        | (u64::from(z[7]) << 8)
        | (u64::from(z[11]) << 16)
        | (u64::from(z[14]) << 24)
        | (u64::from(z[15] >> 1) << 32);
    (r, c)
}
pub(super) fn apply(data: &mut [u8], key: &[u8], nonce: &[u8], data_bits: usize) {
    let (mut r, mut c) = initialize(key, nonce);
    let mut no_input = Zeroizing::new([0u64; 4]);
    for i in 0..data_bits {
        data[i / 8] ^= value(&r, &c) << (i % 8);
        blend(&r, &mut c);
        shift(&mut r, &mut no_input, 240);
    }
}
