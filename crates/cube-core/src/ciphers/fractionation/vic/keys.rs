//! Fixed five-row English VIC derivation; zero is the highest digit in ranking.
use super::*;
use crate::ciphers::mono::checkerboard::{Definition, Profile, Table};

pub(super) struct Keys {
    pub first: Vec<usize>,
    pub second: Vec<usize>,
    pub board: Table,
}
fn order(values: &[u8], numeric: bool) -> Vec<usize> {
    let mut result: Vec<_> = (0..values.len()).collect();
    result.sort_by_key(|&i| {
        (
            if numeric && values[i] == 0 {
                10
            } else {
                values[i]
            },
            i,
        )
    });
    result
}
fn sequence(values: &[u8], numeric: bool) -> Vec<u8> {
    let mut result = vec![0; values.len()];
    for (rank, i) in order(values, numeric).into_iter().enumerate() {
        result[i] = ((rank + 1) % 10) as u8;
    }
    result
}
fn chain(seed: &[u8], total: usize) -> Vec<u8> {
    let mut values = seed.to_vec();
    let mut cursor = 0;
    while values.len() < total {
        values.push((values[cursor] + values[cursor + 1]) % 10);
        cursor += 1;
    }
    values
}
fn canonical(
    value: &str,
    width: usize,
    field: &'static str,
    digits: bool,
    op: &'static str,
) -> Result<(), OperationError> {
    if value.len() != width
        || !value.bytes().all(|c| {
            if digits {
                c.is_ascii_digit()
            } else {
                c.is_ascii_uppercase()
            }
        })
    {
        return Err(invalid(
            op,
            field,
            "exact canonical key width and alphabet required",
        ));
    }
    Ok(())
}
fn prime(p: &VicEnglish2010, op: &'static str) -> Result<Vec<u8>, OperationError> {
    canonical(&p.phrase, 20, "phrase", false, op)?;
    canonical(&p.mnemonic, 8, "mnemonic", false, op)?;
    canonical(&p.date_digits, 6, "date_digits", true, op)?;
    canonical(&p.message_indicator, 5, "message_indicator", true, op)?;
    if p.mnemonic
        .bytes()
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != 8
    {
        return Err(invalid(
            op,
            "mnemonic",
            "eight distinct canonical letters required",
        ));
    }
    if !(1..=24).contains(&p.personal_number) {
        return Err(invalid(
            op,
            "personal_number",
            "personal number must be 1..24",
        ));
    }
    let a = sequence(&p.phrase.as_bytes()[..10], false);
    let b = sequence(&p.phrase.as_bytes()[10..], false);
    let seed: Vec<_> = p
        .message_indicator
        .bytes()
        .zip(p.date_digits.bytes())
        .map(|(a, b)| (a + 10 - b) % 10)
        .collect();
    let f = chain(&seed, 10);
    Ok(a.iter()
        .zip(f)
        .map(|(&a, f)| b[usize::from((a + f + 9) % 10)])
        .collect())
}
pub(super) fn derive(p: &VicEnglish2010, op: &'static str) -> Result<Keys, OperationError> {
    let h = prime(p, op)?;
    let expanded = chain(&h, 60);
    let block = &expanded[10..];
    let last = block[49];
    let previous = *block[..49]
        .iter()
        .rev()
        .find(|&&d| d != last)
        .ok_or_else(|| invalid(op, "phrase", "degenerate five-row key schedule"))?;
    let first = p.personal_number + usize::from(previous);
    let second = p.personal_number + usize::from(last);
    if first + second > 50 {
        return Err(invalid(
            op,
            "personal_number",
            "combined derived widths exceed the fixed fifty-digit block",
        ));
    }
    let breeders: Vec<_> = order(&h, true)
        .into_iter()
        .flat_map(|c| (0..5).map(move |r| block[r * 10 + c]))
        .collect();
    let headers = sequence(&block[40..], true);
    let remaining: Vec<_> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ.#"
        .iter()
        .copied()
        .filter(|c| !p.mnemonic.as_bytes().contains(c))
        .collect();
    let mut alphabet = p.mnemonic.clone();
    for row in 0..2 {
        for i in (row..remaining.len()).step_by(2) {
            alphabet.push(char::from(remaining[i]));
        }
    }
    let board = Table::new(
        Profile::Straddling,
        Definition {
            digits: headers.iter().map(|&d| char::from(b'0' + d)).collect(),
            prefixes: headers[8..].iter().map(|&d| char::from(b'0' + d)).collect(),
            alphabet,
            aliases: vec![],
            digit_escape: None,
        },
    )
    .map_err(|e| e.operation(op))?;
    Ok(Keys {
        first: order(&breeders[..first], true),
        second: order(&breeders[first..first + second], true),
        board,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn digits(s: &str) -> Vec<u8> {
        s.bytes().map(|b| b - b'0').collect()
    }
    #[test]
    fn published_2010_rankings_chain_and_derived_keys_match_every_source_stage() {
        assert_eq!(sequence(b"ALLTHEPEOP", false), digits("1560428379"));
        assert_eq!(sequence(b"LEAREDEADB", false), digits("9610748253"));
        assert_eq!(chain(&digits("31040"), 10), digits("3104041444"));
        let p = super::super::fixtures::parameters();
        let h = prime(&p, "cipher.vic.encode").unwrap();
        assert_eq!(h, digits("0440045891"));
        assert_eq!(
            &chain(&h, 60)[10..],
            digits("48404937052244320757468752722104527994314979683745")
        );
        let k = derive(&p, "cipher.vic.encode").unwrap();
        let one_based = |key: &[usize]| {
            let mut ranks = vec![0; key.len()];
            for (i, &c) in key.iter().enumerate() {
                ranks[c] = i + 1;
            }
            ranks
        };
        assert_eq!(
            one_based(&k.first),
            [
                9, 13, 1, 2, 10, 15, 3, 12, 6, 17, 7, 8, 16, 11, 14, 18, 4, 5, 19
            ]
        );
        assert_eq!(
            one_based(&k.second),
            [
                16, 4, 18, 12, 17, 5, 13, 14, 1, 7, 15, 19, 11, 2, 6, 8, 9, 3, 10, 20
            ]
        );
        assert_eq!(k.board.encode("ASINTOER").unwrap(), "29605817");
    }
}
