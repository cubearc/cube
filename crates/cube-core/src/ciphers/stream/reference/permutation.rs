//! Direct implementations of the published VMPC and Spritz state machines.
//! All byte arithmetic is modulo 256. Spritz encryption is addition, not XOR.
use zeroize::{Zeroize, Zeroizing};

fn identity() -> [u8; 256] {
    std::array::from_fn(|i| i as u8)
}

pub(super) fn vmpc(output: &mut [u8], key: &[u8], iv: &[u8], ksa3: bool) {
    let mut p = Zeroizing::new(identity());
    let mut s = Zeroizing::new(0u8);
    let mix = |p: &mut [u8; 256], s: &mut u8, material: &[u8]| {
        for m in 0..768 {
            let n = m & 255;
            *s = p[s
                .wrapping_add(p[n])
                .wrapping_add(material[m % material.len()]) as usize];
            p.swap(n, *s as usize);
        }
    };
    mix(&mut p, &mut s, key);
    mix(&mut p, &mut s, iv);
    if ksa3 {
        mix(&mut p, &mut s, key);
    }
    for (m, byte) in output.iter_mut().enumerate() {
        let n = m & 255;
        *s = p[s.wrapping_add(p[n]) as usize];
        *byte ^= p[p[p[*s as usize] as usize].wrapping_add(1) as usize];
        p.swap(n, *s as usize);
    }
}

struct Spritz {
    p: [u8; 256],
    i: u8,
    j: u8,
    k: u8,
    z: u8,
    a: u8,
    w: u8,
}
impl Drop for Spritz {
    fn drop(&mut self) {
        self.p.zeroize();
        self.i.zeroize();
        self.j.zeroize();
        self.k.zeroize();
        self.z.zeroize();
        self.a.zeroize();
        self.w.zeroize();
    }
}
impl Spritz {
    fn update(&mut self) {
        self.i = self.i.wrapping_add(self.w);
        self.j = self
            .k
            .wrapping_add(self.p[self.j.wrapping_add(self.p[self.i as usize]) as usize]);
        self.k = self
            .i
            .wrapping_add(self.k)
            .wrapping_add(self.p[self.j as usize]);
        self.p.swap(self.i as usize, self.j as usize);
    }
    fn whip(&mut self) {
        for _ in 0..512 {
            self.update();
        }
        self.w = self.w.wrapping_add(2);
    }
    fn crush(&mut self) {
        for v in 0..128 {
            if self.p[v] > self.p[255 - v] {
                self.p.swap(v, 255 - v);
            }
        }
    }
    fn shuffle(&mut self) {
        self.whip();
        self.crush();
        self.whip();
        self.crush();
        self.whip();
        self.a = 0;
    }
    fn absorb(&mut self, input: &[u8]) {
        for byte in input {
            for nibble in [byte & 15, byte >> 4] {
                if self.a == 128 {
                    self.shuffle();
                }
                self.p.swap(self.a as usize, 128 + nibble as usize);
                self.a += 1;
            }
        }
    }
    fn stop(&mut self) {
        if self.a == 128 {
            self.shuffle();
        }
        self.a += 1;
    }
    fn drip(&mut self) -> u8 {
        if self.a > 0 {
            self.shuffle();
        }
        self.update();
        self.z = self.p[self.j.wrapping_add(
            self.p[self
                .i
                .wrapping_add(self.p[self.z.wrapping_add(self.k) as usize])
                as usize],
        ) as usize];
        self.z
    }
}
pub(super) fn spritz(output: &mut [u8], key: &[u8], iv: Option<&[u8]>, decrypting: bool) {
    let mut s = Spritz {
        p: identity(),
        i: 0,
        j: 0,
        k: 0,
        z: 0,
        a: 0,
        w: 1,
    };
    s.absorb(key);
    if let Some(iv) = iv {
        s.stop();
        s.absorb(iv);
    }
    for byte in output {
        let z = s.drip();
        *byte = if decrypting {
            byte.wrapping_sub(z)
        } else {
            byte.wrapping_add(z)
        };
    }
}

/// FSE2004 section4.2 leaves its K2 PRG choice open. This exact construction
/// selects RC4 with no discard and exactly key.len() output bytes as K2.
pub(super) fn rc4a(output: &mut [u8], key: &[u8]) {
    use chacha20::cipher::{KeyInit, StreamCipher};
    let mut k2 = Zeroizing::new(vec![0; key.len()]);
    rc4::Rc4::new_from_slice(key)
        .expect("validated RC4 key")
        .apply_keystream(&mut k2);
    let ksa = |key: &[u8]| {
        let mut p = Zeroizing::new(identity());
        let mut j = Zeroizing::new(0u8);
        for i in 0..256 {
            *j = j.wrapping_add(p[i]).wrapping_add(key[i % key.len()]);
            p.swap(i, *j as usize);
        }
        p
    };
    let mut p = ksa(key);
    let mut q = ksa(&k2);
    let mut j1 = Zeroizing::new(0u8);
    let mut j2 = Zeroizing::new(0u8);
    for (round, pair) in output.chunks_mut(2).enumerate() {
        let i = (round + 1) & 255;
        *j1 = j1.wrapping_add(p[i]);
        p.swap(i, *j1 as usize);
        pair[0] ^= q[p[i].wrapping_add(p[*j1 as usize]) as usize];
        *j2 = j2.wrapping_add(q[i]);
        q.swap(i, *j2 as usize);
        if pair.len() == 2 {
            pair[1] ^= p[q[i].wrapping_add(q[*j2 as usize]) as usize];
        }
    }
}
