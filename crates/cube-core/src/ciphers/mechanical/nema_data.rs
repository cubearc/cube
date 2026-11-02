//! Measured Model45 contact facts; Sullivan/Weierud1999 and Crypto Museum.
//! B,J->O is independently measured; the paper's duplicate17 is a typo.
//! The entry map follows the paper's clockwise QWERTZU/I contact reference.
pub(super) const ROTORS: [&[u8; 26]; 6] = [
    b"NSKITCOYMVWAUJDRLZXHFQEGPB",
    b"KJYNTMEHLOZQBWPSXIRFAGUDVC",
    b"PNFUTEDIZYAHVRWOJSGBQMKCXL",
    b"WJBEYFUCMDTAZKXPIQHSVLGONR",
    b"HRQTYVXMNACFUJESWLZIGDPOKB",
    b"ZVGEQMUTWLNSHPOAFYIXKBDRJC",
];
pub(super) const REFLECTOR: &[u8; 26] = b"KWPNGHEFVTAZUDYCXSRJMIBQOL";
pub(super) const ENTRY: &[u8; 26] = b"YLNWGVUTBSRQJKAZIFXECMHOPD";
pub(super) const NOTCHES: [(u8, &[u8; 26]); 14] = [
    (1, b"01000000010000000010000011"),
    (2, b"01011001000000001000000000"),
    (12, b"01111111111100011110111111"),
    (13, b"11011110011011011101111110"),
    (14, b"00101111011111111010010101"),
    (15, b"10011010000010111111010111"),
    (16, b"11111101111111101111111110"),
    (17, b"01000001111000001010110110"),
    (18, b"11111111111110101111111011"),
    (19, b"11101111000111111111110111"),
    (20, b"11111101110101010101101111"),
    (21, b"10111011110111101110100100"),
    (22, b"11001011001011011110011100"),
    (23, b"10111111111101111111111110"),
];
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn measured_contacts_are_permutations_and_notches_have_source_counts() {
        for source in ROTORS.into_iter().chain([REFLECTOR, ENTRY]) {
            let mut values = *source;
            values.sort_unstable();
            assert_eq!(&values, b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        for (i, value) in REFLECTOR.iter().enumerate() {
            assert_ne!(usize::from(*value - b'A'), i);
            assert_eq!(REFLECTOR[usize::from(*value - b'A')], b'A' + i as u8);
        }
        for ((_, pattern), count) in NOTCHES
            .iter()
            .zip([5, 5, 21, 19, 17, 15, 23, 11, 23, 21, 19, 17, 15, 23])
        {
            assert_eq!(pattern.iter().filter(|b| **b == b'1').count(), count);
        }
        assert_eq!(ROTORS[1][9], b'O');
    }
}
