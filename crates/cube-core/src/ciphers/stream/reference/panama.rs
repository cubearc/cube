//! PANAMA's 1998 reset/push/pull stream construction, explicit word byte order.
use zeroize::Zeroizing;
fn step(s: &mut [u32; 17], belt: &mut [[u32; 8]; 32], head: &mut usize, block: Option<&[u32; 8]>) {
    let mut gamma = Zeroizing::new([0u32; 17]);
    let mut pi = Zeroizing::new([0u32; 17]);
    let mut theta = Zeroizing::new([0u32; 17]);
    for i in 0..17 {
        gamma[i] = s[i] ^ (s[(i + 1) % 17] | !s[(i + 2) % 17]);
    }
    for i in 0..17 {
        pi[i] = gamma[(7 * i) % 17].rotate_left(((i * (i + 1) / 2) % 32) as u32);
    }
    for i in 0..17 {
        theta[i] = pi[i] ^ pi[(i + 1) % 17] ^ pi[(i + 4) % 17];
    }
    let l = Zeroizing::new(block.copied().unwrap_or(belt[(*head + 4) & 31]));
    let b = Zeroizing::new(belt[(*head + 16) & 31]);
    let feedback = Zeroizing::new(
        block
            .copied()
            .unwrap_or_else(|| s[1..9].try_into().unwrap()),
    );
    *head = (*head + 31) & 31;
    for i in 0..8 {
        belt[(*head + 25) & 31][i] ^= belt[*head][(i + 2) & 7];
    }
    for i in 0..8 {
        belt[*head][i] ^= feedback[i];
    }
    s[0] = theta[0] ^ 1;
    for i in 0..8 {
        s[1 + i] = theta[1 + i] ^ l[i];
        s[9 + i] = theta[9 + i] ^ b[i];
    }
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8], little: bool) {
    let words = |bytes: &[u8]| {
        std::array::from_fn(|i| {
            let b = bytes[4 * i..4 * i + 4].try_into().unwrap();
            if little {
                u32::from_le_bytes(b)
            } else {
                u32::from_be_bytes(b)
            }
        })
    };
    let k = Zeroizing::new(words(key));
    let mut s = Zeroizing::new([0u32; 17]);
    let mut belt = Zeroizing::new([[0u32; 8]; 32]);
    let mut head = 0;
    step(&mut s, &mut belt, &mut head, Some(&k));
    if !iv.is_empty() {
        let iv = Zeroizing::new(words(iv));
        step(&mut s, &mut belt, &mut head, Some(&iv));
    }
    for _ in 0..32 {
        step(&mut s, &mut belt, &mut head, None);
    }
    for chunk in output.chunks_mut(32) {
        for (word, z) in chunk.chunks_mut(4).zip(&s[9..17]) {
            let stream = if little {
                z.to_le_bytes()
            } else {
                z.to_be_bytes()
            };
            for (b, k) in word.iter_mut().zip(stream) {
                *b ^= k;
            }
        }
        step(&mut s, &mut belt, &mut head, None);
    }
}
