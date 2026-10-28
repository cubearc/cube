use super::*;

// Measured right-to-left wiring at ring A/01; turnover letters are those
// visible BEFORE the keypress. Keep physical notch locations out of this table.
const WIRINGS: [&[u8; 26]; 8] = [
    b"EKMFLGDQVZNTOWYHXUSPAIBRCJ",
    b"AJDKSIRUXBLHWTMCQGZNPYFVOE",
    b"BDFHJLCPRTXVZNYEIWGAKMUSQO",
    b"ESOVPZJAYQUIRHXLNFTGKDCMWB",
    b"VZBRGITYUPSDNHLXAWMJQOFECK",
    b"JPGVOUMFYQBENHZRDKASXLICTW",
    b"NZJHGRCXMYSWBOUFAIVLPEKQDT",
    b"FKQHTLXOCBJSPDZRAMEWNIUYGV",
];
const NOTCHES: [&[u8]; 8] = [b"Q", b"E", b"V", b"J", b"Z", b"MZ", b"MZ", b"MZ"];

struct Wheel {
    forward: [u8; 26],
    inverse: [u8; 26],
    notches: &'static [u8],
    ring: u8,
    position: u8,
}
impl Wheel {
    fn new(rotor: Rotor, ring: u8, position: u8) -> Self {
        let wiring = WIRINGS[rotor as usize];
        let forward = wiring.map(|b| b - b'A');
        let mut inverse = [0; 26];
        for (input, output) in forward.iter().enumerate() {
            inverse[*output as usize] = input as u8;
        }
        Self {
            forward,
            inverse,
            notches: NOTCHES[rotor as usize],
            ring: ring - 1,
            position,
        }
    }
    fn at_notch(&self) -> bool {
        self.notches.contains(&(self.position + b'A'))
    }
    fn step(&mut self) {
        self.position = (self.position + 1) % 26;
    }
    fn pass(&self, input: u8, backwards: bool) -> u8 {
        let offset = (self.position + 26 - self.ring) % 26;
        let index = (input + offset) % 26;
        let wiring = if backwards {
            &self.inverse
        } else {
            &self.forward
        };
        (wiring[index as usize] + 26 - offset) % 26
    }
}

pub(super) struct Machine {
    wheels: [Wheel; 3],
    plugs: [u8; 26],
    reflector: &'static [u8; 26],
}
impl Machine {
    pub(super) fn new(p: &Parameters) -> Result<Self, OperationError> {
        validate(p)?;
        let positions = p.windows.as_bytes();
        Ok(Self {
            wheels: std::array::from_fn(|i| {
                Wheel::new(p.rotors[i], p.rings[i], positions[i] - b'A')
            }),
            plugs: plugboard(&p.plugboard)?,
            reflector: match p.reflector {
                Reflector::B => b"YRUHQSLDPXNGOKMIEBFZCWVJAT",
                Reflector::C => b"FVPJIAOYEDRZXWGCTKUQSBNMHL",
            },
        })
    }
    pub(super) fn windows(&self) -> String {
        self.wheels
            .iter()
            .map(|w| char::from(b'A' + w.position))
            .collect()
    }
    pub(super) fn key(&mut self, input: u8) -> u8 {
        // Decide all movements from the old state, before electrical traversal.
        let middle = self.wheels[1].at_notch();
        let right = self.wheels[2].at_notch();
        if middle {
            self.wheels[0].step();
        }
        if middle || right {
            self.wheels[1].step();
        }
        self.wheels[2].step();
        let mut value = self.plugs[input as usize];
        for w in self.wheels.iter().rev() {
            value = w.pass(value, false);
        }
        value = self.reflector[value as usize] - b'A';
        for w in &self.wheels {
            value = w.pass(value, true);
        }
        self.plugs[value as usize]
    }
}

fn validate(p: &Parameters) -> Result<(), OperationError> {
    if p.rotors[0] == p.rotors[1]
        || p.rotors[1] == p.rotors[2]
        || p.rotors[0] == p.rotors[2]
        || (p.profile == Profile::EnigmaI && p.rotors.iter().any(|r| *r as usize >= 5))
    {
        return Err(error(
            ErrorCode::InvalidParameter,
            "rotors",
            "use three distinct rotors available to the selected profile",
        ));
    }
    if p.rings.iter().any(|r| !(1..=26).contains(r)) {
        return Err(error(
            ErrorCode::InvalidParameter,
            "rings",
            "rings must be 1..26 in left-to-right order",
        ));
    }
    if p.windows.len() != 3 || !p.windows.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(error(
            ErrorCode::InvalidParameter,
            "windows",
            "windows require exactly three ASCII A-Z letters",
        ));
    }
    Ok(())
}

fn plugboard(pairs: &[String]) -> Result<[u8; 26], OperationError> {
    let invalid = || {
        error(
            ErrorCode::InvalidParameter,
            "plugboard",
            "use at most ten disjoint two-letter ASCII A-Z pairs",
        )
    };
    if pairs.len() > 10 {
        return Err(invalid());
    }
    let mut plugs = std::array::from_fn(|i| i as u8);
    let mut used = [false; 26];
    for pair in pairs {
        if pair.len() != 2 || !pair.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(invalid());
        }
        let a = (pair.as_bytes()[0] - b'A') as usize;
        let b = (pair.as_bytes()[1] - b'A') as usize;
        if a == b || used[a] || used[b] {
            return Err(invalid());
        }
        used[a] = true;
        used[b] = true;
        plugs[a] = b as u8;
        plugs[b] = a as u8;
    }
    Ok(plugs)
}
