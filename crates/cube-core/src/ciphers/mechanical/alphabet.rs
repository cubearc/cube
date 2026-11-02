//! Validated26-contact alphabets; this shares permutations, never machine stepping.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Alphabet {
    letters: [u8; 26],
    positions: [u8; 26],
}
impl Alphabet {
    pub(super) fn new(
        text: &str,
        op: &'static str,
        field: &'static str,
    ) -> Result<Self, OperationError> {
        let invalid = || {
            error(
                op,
                ErrorCode::InvalidParameter,
                field,
                "use each ASCII letter A-Z exactly once",
            )
        };
        let letters: [u8; 26] = text.as_bytes().try_into().map_err(|_| invalid())?;
        let mut seen = [false; 26];
        let mut positions = [0; 26];
        for (index, &letter) in letters.iter().enumerate() {
            if !letter.is_ascii_uppercase() {
                return Err(invalid());
            }
            let code = usize::from(letter - b'A');
            if std::mem::replace(&mut seen[code], true) {
                return Err(invalid());
            }
            positions[code] = index as u8;
        }
        Ok(Self { letters, positions })
    }
    pub(super) fn position(self, letter: u8) -> Option<u8> {
        letter
            .is_ascii_uppercase()
            .then(|| self.positions[usize::from(letter - b'A')])
    }
    pub(super) fn letter(self, position: u8) -> u8 {
        self.letters[usize::from(position)]
    }
}
