//! Fixed-size contact permutations, shared by the two complete US rotor machines.
//! This handles finite wiring math only; each machine owns its motion and state.
#[derive(Clone, Copy, Debug)]
pub(super) struct Wiring<const N: usize> {
    forward: [u8; N],
    backward: [u8; N],
}
impl<const N: usize> Wiring<N> {
    pub(super) fn new(source: &[u8; N], reversed: bool) -> Self {
        let mut inverse = [0; N];
        for (x, y) in source.iter().enumerate() {
            inverse[usize::from(*y)] = x as u8;
        }
        let forward = std::array::from_fn(|x| {
            if reversed {
                ((N - usize::from(inverse[(N - x) % N])) % N) as u8
            } else {
                source[x]
            }
        });
        let mut backward = [0; N];
        for (x, y) in forward.iter().enumerate() {
            backward[usize::from(*y)] = x as u8;
        }
        Self { forward, backward }
    }
    pub(super) fn forward(&self, value: u8, position: u8) -> u8 {
        self.at(&self.forward, value, position)
    }
    pub(super) fn backward(&self, value: u8, position: u8) -> u8 {
        self.at(&self.backward, value, position)
    }
    fn at(&self, wiring: &[u8; N], value: u8, position: u8) -> u8 {
        ((usize::from(wiring[(usize::from(value) + usize::from(position)) % N]) + N
            - usize::from(position))
            % N) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::super::{kl7_data as kl7, sigaba_data as sigaba};
    #[test]
    fn full_reference_contact_and_notch_transcriptions_match_pinned_source_facts() {
        let sigaba = serde_json::to_vec(&(sigaba::ROTORS, sigaba::INDEX)).unwrap();
        assert_eq!(
            crate::hashing::sha256(&sigaba),
            "c08b1b6d484dfe98429e63e97e6ab3fcec8b9c5cb284efa2581d51903874114a"
        );
        let kl7 = serde_json::to_vec(&(
            kl7::ROTORS.map(|r| r.to_vec()),
            kl7::NOTCHES.map(|r| r.to_vec()),
            kl7::BASE.to_vec(),
        ))
        .unwrap();
        assert_eq!(
            crate::hashing::sha256(&kl7),
            "b48d92348510f63d6d597174409a2d81a94eab8dbc43a1b99f04c4abbf027be9"
        );
    }
}
