//! Explicit state, rebuilt on each call. No global deck, stream or random source.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Gromark {
    #[schemars(length(equal = 26))]
    pub alphabet: String,
    #[schemars(length(equal = 5), regex(pattern = r"^[0-9]{5}$"))]
    pub primer: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct PeriodicGromark {
    #[schemars(length(min = 1, max = 65536), regex(pattern = r"^[A-Za-z]+$"))]
    pub keyword: String,
    /// Must equal the number of distinct keyword letters (2..9).
    #[schemars(range(min = 2, max = 9))]
    pub period: u8,
}
fn positions(alphabet: &[u8]) -> [usize; 26] {
    let mut positions = [0; 26];
    for (i, &c) in alphabet.iter().enumerate() {
        positions[usize::from(c - b'A')] = i;
    }
    positions
}
/// Five decimal primer digits, adjacent-sum recurrence; no printed primer/check digit.
pub fn gromark(text: &str, p: &Gromark, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.gromark.decode"
    } else {
        "cipher.gromark.encode"
    };
    let a = permutation(&p.alphabet, AZ, op, "alphabet")?;
    if p.primer.len() != 5 || !p.primer.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(
            op,
            "primer",
            "primer must contain exactly five ASCII decimal digits",
        ));
    }
    let mut stream: Vec<u8> = p.primer.bytes().map(|b| b - b'0').collect();
    let x = letters(text, MAX_INPUT, op, false)?;
    let pos = positions(&a);
    let mut out = String::with_capacity(x.len());
    for (i, c) in x.into_iter().enumerate() {
        let j = i % 5;
        let k = usize::from(stream[j]);
        stream[j] = (stream[j] + stream[(j + 1) % 5]) % 10;
        out.push(char::from(if decode {
            b'A' + ((pos[usize::from(c - b'A')] + 26 - k) % 26) as u8
        } else {
            a[(usize::from(c - b'A') + k) % 26]
        }));
    }
    Ok(out)
}
/// ACA keyed columnar alphabet, distinct-key ranking primer, and periodic offsets.
pub fn periodic_gromark(
    text: &str,
    p: &PeriodicGromark,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.periodic_gromark.decode"
    } else {
        "cipher.periodic_gromark.encode"
    };
    let key = letter_key(&p.keyword, MAX_KEY, false, op)?;
    let mut distinct = Vec::new();
    for c in key {
        if !distinct.contains(&c) {
            distinct.push(c);
        }
    }
    if usize::from(p.period) != distinct.len() {
        return Err(invalid(
            op,
            "period",
            "period must equal the distinct keyword length",
        ));
    }
    if !(2..=9).contains(&distinct.len()) {
        return Err(invalid(
            op,
            "keyword",
            "qualified decimal-ranking profile requires 2..9 distinct letters",
        ));
    }
    let n = distinct.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| distinct[i]);
    let mut chain = vec![0; n];
    for (rank, &i) in order.iter().enumerate() {
        chain[i] = (rank + 1) as u8;
    }
    let mut keyed = distinct.clone();
    for c in 0..26 {
        if !distinct.contains(&c) {
            keyed.push(c);
        }
    }
    let mut a = Vec::with_capacity(26);
    for col in order {
        for i in (col..26).step_by(n) {
            a.push(keyed[i] + b'A');
        }
    }
    let pos = positions(&a);
    let x = letters(text, MAX_INPUT, op, false)?;
    let mut out = String::with_capacity(x.len());
    for (i, c) in x.into_iter().enumerate() {
        let j = i % n;
        let k = usize::from(chain[j]) + pos[usize::from(distinct[(i / n) % n])];
        chain[j] = (chain[j] + chain[(j + 1) % n]) % 10;
        out.push(char::from(if decode {
            b'A' + ((pos[usize::from(c - b'A')] + 52 - k) % 26) as u8
        } else {
            a[(usize::from(c - b'A') + k) % 26]
        }));
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Condi {
    #[schemars(length(equal = 26))]
    pub alphabet: String,
    #[schemars(range(min = 0, max = 25))]
    pub shift: u8,
}
/// The next shift is the recovered plaintext's one-based keyed-alphabet position.
pub fn condi(text: &str, p: &Condi, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.condi.decode"
    } else {
        "cipher.condi.encode"
    };
    let a = permutation(&p.alphabet, AZ, op, "alphabet")?;
    if p.shift > 25 {
        return Err(invalid(op, "shift", "shift must be 0..25"));
    }
    let pos = positions(&a);
    let mut shift = usize::from(p.shift);
    let x = letters(text, MAX_INPUT, op, false)?;
    let mut out = String::with_capacity(x.len());
    for c in x {
        let i = pos[usize::from(c - b'A')];
        let plain = if decode { (i + 26 - shift) % 26 } else { i };
        out.push(char::from(a[if decode { plain } else { (i + shift) % 26 }]));
        shift = (plain + 1) % 26;
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Ragbaby {
    #[schemars(length(equal = 24))]
    pub alphabet: String,
    /// Explicitly merge J into I and X into W on encoding only.
    pub merge_aliases: bool,
}
/// Words start at one; apostrophes/hyphens stay inside words, separators survive.
pub fn ragbaby(text: &str, p: &Ragbaby, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.ragbaby.decode"
    } else {
        "cipher.ragbaby.encode"
    };
    let a = permutation(&p.alphabet, "ABCDEFGHIKLMNOPQRSTUVWYZ", op, "alphabet")?;
    bound(op, "text", text.len(), MAX_INPUT)?;
    let mut word = 0usize;
    let mut letter = 0usize;
    let mut inside = false;
    let mut out = String::with_capacity(text.len());
    for (offset, c) in text.char_indices() {
        if c.is_alphabetic() {
            if !c.is_ascii_uppercase() {
                return Err(input_error(op, offset));
            }
            let mut b = c as u8;
            if !decode && p.merge_aliases {
                b = match b {
                    b'J' => b'I',
                    b'X' => b'W',
                    _ => b,
                };
            }
            let i = a
                .iter()
                .position(|&v| v == b)
                .ok_or_else(|| input_error(op, offset))?;
            if !inside {
                word += 1;
                bound(op, "words", word, MAX_SYMBOLS)?;
                letter = 0;
                inside = true;
            }
            letter += 1;
            let shift = (word + letter - 1) % 24;
            out.push(char::from(
                a[if decode {
                    (i + 24 - shift) % 24
                } else {
                    (i + shift) % 24
                }],
            ));
        } else {
            if c != '-' && c != '\'' {
                inside = false;
            }
            out.push(c);
        }
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Chaocipher {
    #[schemars(length(equal = 26))]
    pub left: String,
    #[schemars(length(equal = 26))]
    pub right: String,
}
/// Byrne/Rubin profile: left extract index 1, right extra rotation/extract index 2.
pub fn chaocipher(text: &str, p: &Chaocipher, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.chaocipher.decode"
    } else {
        "cipher.chaocipher.encode"
    };
    let mut left = permutation(&p.left, AZ, op, "left")?;
    let mut right = permutation(&p.right, AZ, op, "right")?;
    let x = letters(text, MAX_INPUT, op, false)?;
    let mut out = String::with_capacity(x.len());
    for c in x {
        let source = if decode { &left } else { &right };
        let i = source
            .iter()
            .position(|&v| v == c)
            .expect("validated permutation");
        out.push(char::from(if decode { right[i] } else { left[i] }));
        left.rotate_left(i);
        let l = left.remove(1);
        left.insert(13, l);
        right.rotate_left((i + 1) % 26);
        let r = right.remove(2);
        right.insert(13, r);
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Solitaire {
    /// Cards 1..54 exactly once; jokers A=53, B=54. Sensitive.
    #[schemars(length(equal = 54),with="Vec<Card>",extend("writeOnly"=true))]
    pub deck: Vec<u8>,
    /// Optional Schneier passphrase keying requires the ordered input deck.
    #[serde(default)]
    #[schemars(length(max = 65536), regex(pattern = r"^[A-Za-z]*$"))]
    #[schemars(extend("writeOnly"=true))]
    pub passphrase: String,
    /// Counts every deck round, including keying and rejected joker outputs.
    #[serde(default = "default_steps")]
    #[schemars(range(min = 0, max = 1048576))]
    pub max_steps: u32,
}
fn default_steps() -> u32 {
    1048576
}
fn count_cut(deck: &mut [u8], count: usize) {
    deck[..53].rotate_left(count % 53);
}
fn deck_round(deck: &mut Vec<u8>) {
    for (joker, moves) in [(53, 1), (54, 2)] {
        for _ in 0..moves {
            let i = deck
                .iter()
                .position(|&c| c == joker)
                .expect("validated deck");
            let card = deck.remove(i);
            let j = if i == 53 { 1 } else { i + 1 };
            deck.insert(j, card);
        }
    }
    let a = deck.iter().position(|&c| c == 53).expect("validated deck");
    let b = deck.iter().position(|&c| c == 54).expect("validated deck");
    let (a, b) = (a.min(b), a.max(b));
    let mut next = Vec::with_capacity(54);
    next.extend_from_slice(&deck[b + 1..]);
    next.extend_from_slice(&deck[a..=b]);
    next.extend_from_slice(&deck[..a]);
    *deck = next;
    let count = usize::from(deck[53].min(53));
    count_cut(deck, count);
}
/// Original Solitaire, with explicit deck, bounded rejection loop, and no padding.
pub fn solitaire(text: &str, p: &Solitaire, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.solitaire.decode"
    } else {
        "cipher.solitaire.encode"
    };
    if p.deck.len() != 54 || (1..=54).any(|v| p.deck.iter().filter(|&&c| c == v).count() != 1) {
        return Err(invalid(
            op,
            "deck",
            "deck must be an exact permutation of cards 1..54",
        ));
    }
    if p.max_steps > 1048576 {
        return Err(invalid(
            op,
            "max_steps",
            "max_steps must not exceed 1048576",
        ));
    }
    let phrase = letter_key(&p.passphrase, MAX_KEY, true, op)?;
    if !phrase.is_empty() && !p.deck.iter().copied().eq(1..=54) {
        return Err(invalid(
            op,
            "passphrase",
            "passphrase keying requires the ordered deck",
        ));
    }
    let x = letters(text, MAX_SYMBOLS, op, false)?;
    let mut deck = p.deck.clone();
    let mut steps = 0u32;
    let mut round = |deck: &mut Vec<u8>| -> Result<(), OperationError> {
        if steps >= p.max_steps {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "Solitaire deck-round budget exhausted",
            )
            .operation(op)
            .parameter("max_steps"));
        }
        steps += 1;
        deck_round(deck);
        Ok(())
    };
    for c in phrase {
        round(&mut deck)?;
        count_cut(&mut deck, usize::from(c + 1));
    }
    let mut out = String::with_capacity(x.len());
    for c in x {
        let k = loop {
            round(&mut deck)?;
            let card = deck[usize::from(deck[0].min(53))];
            if card < 53 {
                break card % 26;
            }
        };
        out.push(char::from(
            b'A' + if decode {
                (c - b'A' + 26 - k) % 26
            } else {
                (c - b'A' + k) % 26
            },
        ));
    }
    Ok(out)
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Card(#[schemars(range(min = 1, max = 54))] u8);
