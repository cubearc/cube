//! SEAL3, L=32768 bits per position. State flow adapted from public-domain
//! Wei Dai/Leonard Janke Crypto++ reference; SHA1 compression reuses RustCrypto.
use zeroize::Zeroizing;
struct Tables {
    t: Zeroizing<[u32; 512]>,
    s: Zeroizing<[u32; 256]>,
    r: Zeroizing<[u32; 16]>,
}
fn tables(key: &[u8]) -> Tables {
    let mut h = Zeroizing::new([0u32; 5]);
    for (w, b) in h.iter_mut().zip(key.as_chunks::<4>().0) {
        *w = u32::from_be_bytes(*b);
    }
    let mut last = None;
    let mut z = Zeroizing::new([0u32; 5]);
    let mut block = Zeroizing::new([[0u8; 64]; 1]);
    let mut gamma = |i: usize| {
        let index = (i / 5) as u32;
        if last != Some(index) {
            z.copy_from_slice(&*h);
            block[0][..4].copy_from_slice(&index.to_be_bytes());
            sha1::block_api::compress(&mut z, &*block);
            last = Some(index);
        }
        z[i % 5]
    };
    let mut tables = Tables {
        t: Zeroizing::new([0; 512]),
        s: Zeroizing::new([0; 256]),
        r: Zeroizing::new([0; 16]),
    };
    for (i, w) in tables.t.iter_mut().enumerate() {
        *w = gamma(i);
    }
    for (i, w) in tables.s.iter_mut().enumerate() {
        *w = gamma(0x1000 + i);
    }
    for (i, w) in tables.r.iter_mut().enumerate() {
        *w = gamma(0x2000 + i);
    }
    tables
}
fn lookup(t: &[u32; 512], p: u32) -> u32 {
    t[(p >> 2) as usize]
}
fn initialize(t: &[u32; 512], x: &mut [u32; 4]) {
    for i in 0..4 {
        let p = x[i] & 0x7fc;
        let next = (i + 1) % 4;
        x[next] = x[next].wrapping_add(lookup(t, p));
        x[i] = x[i].rotate_right(9);
    }
}
fn round(t: &[u32; 512], x: &mut [u32; 4]) {
    let mut p = x[0] & 0x7fc;
    x[0] = x[0].rotate_right(9);
    x[1] = x[1].wrapping_add(lookup(t, p)) ^ x[0];
    let mut q = x[1] & 0x7fc;
    x[1] = x[1].rotate_right(9);
    x[2] = (x[2] ^ lookup(t, q)).wrapping_add(x[1]);
    p = p.wrapping_add(x[2]) & 0x7fc;
    x[2] = x[2].rotate_right(9);
    x[3] = x[3].wrapping_add(lookup(t, p)) ^ x[2];
    q = q.wrapping_add(x[3]) & 0x7fc;
    x[3] = x[3].rotate_right(9);
    x[0] = (x[0] ^ lookup(t, q)).wrapping_add(x[3]);
    p = p.wrapping_add(x[0]) & 0x7fc;
    x[1] ^= lookup(t, p);
    x[0] = x[0].rotate_right(9);
    q = q.wrapping_add(x[1]) & 0x7fc;
    x[2] = x[2].wrapping_add(lookup(t, q));
    x[1] = x[1].rotate_right(9);
    p = p.wrapping_add(x[2]) & 0x7fc;
    x[3] ^= lookup(t, p);
    x[2] = x[2].rotate_right(9);
    q = q.wrapping_add(x[3]) & 0x7fc;
    x[3] = x[3].rotate_right(9);
    x[0] = x[0].wrapping_add(lookup(t, q));
}
fn block(output: &mut [u8], tables: &Tables, outside: u32, inside: usize, little: bool) {
    let mut x = Zeroizing::new([
        outside ^ tables.r[4 * inside],
        outside.rotate_right(8) ^ tables.r[4 * inside + 1],
        outside.rotate_right(16) ^ tables.r[4 * inside + 2],
        outside.rotate_right(24) ^ tables.r[4 * inside + 3],
    ]);
    initialize(&tables.t, &mut x);
    initialize(&tables.t, &mut x);
    let n = Zeroizing::new([x[3], x[1], x[0], x[2]]);
    initialize(&tables.t, &mut x);
    for (i, chunk) in output.chunks_mut(16).enumerate() {
        round(&tables.t, &mut x);
        let words = Zeroizing::new([
            x[1].wrapping_add(tables.s[4 * i]),
            x[2] ^ tables.s[4 * i + 1],
            x[3].wrapping_add(tables.s[4 * i + 2]),
            x[0] ^ tables.s[4 * i + 3],
        ]);
        for (word, bytes) in words.iter().zip(chunk.chunks_mut(4)) {
            let stream = if little {
                word.to_le_bytes()
            } else {
                word.to_be_bytes()
            };
            for (b, k) in bytes.iter_mut().zip(stream) {
                *b ^= k;
            }
        }
        let j = if i & 1 == 0 { 0 } else { 2 };
        x[0] = x[0].wrapping_add(n[j]);
        x[1] = x[1].wrapping_add(n[j + 1]);
        x[2] ^= n[j];
        x[3] ^= n[j + 1];
    }
}
#[derive(Debug)]
pub(super) struct CounterLimit;
pub(super) fn apply(
    output: &mut [u8],
    key: &[u8],
    iv: &[u8],
    little: bool,
) -> Result<(), CounterLimit> {
    let start = u32::from_be_bytes(iv.try_into().expect("validated IV"));
    if output.len() as u64 > (u64::from(u32::MAX) - u64::from(start) + 1) * 4096 {
        return Err(CounterLimit);
    }
    let tables = tables(key);
    for (i, chunk) in output.chunks_mut(1024).enumerate() {
        block(chunk, &tables, start + (i / 4) as u32, i % 4, little);
    }
    Ok(())
}
