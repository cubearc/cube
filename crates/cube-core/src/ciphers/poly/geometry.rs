//! Concrete 5x5 and 27-symbol geometries. Padding evidence stays visible.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Squares {
    #[schemars(length(equal = 25))]
    pub first: String,
    #[schemars(length(equal = 25))]
    pub second: String,
}
struct Square {
    cells: Vec<u8>,
    positions: [Option<usize>; 26],
}
impl Square {
    fn new(value: &str, op: &'static str, field: &'static str) -> Result<Self, OperationError> {
        let cells = permutation(value, IJ, op, field)?;
        let mut positions = [None; 26];
        for (i, &c) in cells.iter().enumerate() {
            positions[usize::from(c - b'A')] = Some(i);
        }
        Ok(Self { cells, positions })
    }
    fn pos(&self, c: u8, i: usize, op: &'static str) -> Result<(usize, usize), OperationError> {
        let idx = self.positions[usize::from(c - b'A')].ok_or_else(|| input_error(op, i))?;
        Ok((idx / 5, idx % 5))
    }
    fn at(&self, r: usize, c: usize) -> u8 {
        self.cells[5 * r + c]
    }
}
fn even(text: &str, max: usize, op: &'static str) -> Result<Vec<u8>, OperationError> {
    let x = letters(text, max, op, false)?;
    if x.len() % 2 != 0 {
        return Err(invalid(
            op,
            "text",
            "profile requires even canonical text; padding must be explicit",
        ));
    }
    Ok(x)
}
/// Horizontal Two-square: ciphertext square order is second, then first.
pub fn two_square(text: &str, p: &Squares, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.two_square.decode"
    } else {
        "cipher.two_square.encode"
    };
    let a = Square::new(&p.first, op, "first")?;
    let b = Square::new(&p.second, op, "second")?;
    let x = even(text, MAX_INPUT, op)?;
    let mut out = String::with_capacity(x.len());
    for (i, pair) in x.as_chunks::<2>().0.iter().enumerate() {
        let (left, right) = if decode { (&b, &a) } else { (&a, &b) };
        let (r, c) = left.pos(pair[0], 2 * i, op)?;
        let (s, d) = right.pos(pair[1], 2 * i + 1, op)?;
        out.push(char::from(if decode { a.at(r, d) } else { b.at(r, d) }));
        out.push(char::from(if decode { b.at(s, c) } else { a.at(s, c) }));
    }
    Ok(out)
}
/// Straight squares at top-left/bottom-right, keyed first at top-right.
pub fn four_square(text: &str, p: &Squares, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.four_square.decode"
    } else {
        "cipher.four_square.encode"
    };
    let a = Square::new(&p.first, op, "first")?;
    let b = Square::new(&p.second, op, "second")?;
    let plain = Square::new(IJ, op, "square")?;
    let x = even(text, MAX_INPUT, op)?;
    let mut out = String::with_capacity(x.len());
    for (i, pair) in x.as_chunks::<2>().0.iter().enumerate() {
        let (left, right) = if decode { (&a, &b) } else { (&plain, &plain) };
        let (r, c) = left.pos(pair[0], 2 * i, op)?;
        let (s, d) = right.pos(pair[1], 2 * i + 1, op)?;
        out.push(char::from(if decode { plain.at(r, d) } else { a.at(r, d) }));
        out.push(char::from(if decode { plain.at(s, c) } else { b.at(s, c) }));
    }
    Ok(out)
}
fn filler() -> char {
    'X'
}
fn alternate() -> char {
    'Q'
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Playfair {
    #[schemars(length(equal = 25))]
    pub square: String,
    #[serde(default = "filler")]
    pub filler: char,
    #[serde(default = "alternate")]
    pub alternate: char,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct SeriatedPlayfair {
    #[schemars(length(equal = 25))]
    pub square: String,
    #[schemars(range(min = 1, max = 4096))]
    pub period: u16,
    #[serde(default = "filler")]
    pub filler: char,
    #[serde(default = "alternate")]
    pub alternate: char,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    #[schemars(length(max = 2097152))]
    pub text: String,
    #[schemars(length(max = 2097152))]
    pub prepared: String,
    /// Zero-based scalar indexes in prepared text. Decode does not infer fillers.
    #[schemars(length(max = 1048576))]
    pub fillers: Vec<u32>,
}
fn check_fillers(f: char, a: char, op: &'static str) -> Result<(u8, u8), OperationError> {
    if f == a || !IJ.contains(f) || !IJ.contains(a) {
        return Err(invalid(
            op,
            "filler",
            "filler and alternate must be distinct I/J-square symbols",
        ));
    }
    Ok((f as u8, a as u8))
}
fn pair(
    square: &Square,
    a: u8,
    b: u8,
    decode: bool,
    i: usize,
    op: &'static str,
) -> Result<(u8, u8), OperationError> {
    let (r, c) = square.pos(a, i, op)?;
    let (s, d) = square.pos(b, i + 1, op)?;
    if a == b {
        return Err(invalid(op, "text", "Playfair paired symbols must differ"));
    }
    let delta = if decode { 4 } else { 1 };
    Ok(if r == s {
        (square.at(r, (c + delta) % 5), square.at(s, (d + delta) % 5))
    } else if c == d {
        (square.at((r + delta) % 5, c), square.at((s + delta) % 5, d))
    } else {
        (square.at(r, d), square.at(s, c))
    })
}
fn prepared_result(
    output: Vec<u8>,
    prepared: Vec<u8>,
    fillers: Vec<u32>,
    op: &'static str,
) -> Result<Prepared, OperationError> {
    let result = Prepared {
        text: output.into_iter().map(char::from).collect(),
        prepared: prepared.into_iter().map(char::from).collect(),
        fillers,
    };
    json_bound(&result, MAX_REPORT, op)?;
    Ok(result)
}
/// Adjacent digraphs; encode reports inserted fillers, decode returns prepared text.
pub fn playfair(text: &str, p: &Playfair, decode: bool) -> Result<Prepared, OperationError> {
    let op = if decode {
        "cipher.playfair.decode"
    } else {
        "cipher.playfair.encode"
    };
    let square = Square::new(&p.square, op, "square")?;
    let (f, a) = check_fillers(p.filler, p.alternate, op)?;
    let x = letters(
        text,
        if decode { MAX_PREPARED } else { MAX_INPUT },
        op,
        false,
    )?;
    let mut prepared = Vec::with_capacity(x.len());
    let mut fillers = Vec::new();
    if decode {
        if x.len() % 2 != 0 {
            return Err(invalid(
                op,
                "text",
                "Playfair ciphertext must have even length",
            ));
        }
        prepared = x;
    } else {
        let mut i = 0;
        while i < x.len() {
            let c = x[i];
            square.pos(c, i, op)?;
            prepared.push(c);
            i += 1;
            if i == x.len() || x[i] == c {
                fillers.push(prepared.len() as u32);
                prepared.push(if c == f { a } else { f });
            } else {
                square.pos(x[i], i, op)?;
                prepared.push(x[i]);
                i += 1;
            }
        }
    }
    let mut out = Vec::with_capacity(prepared.len());
    for (i, pair_in) in prepared.as_chunks::<2>().0.iter().enumerate() {
        let (c, d) = pair(&square, pair_in[0], pair_in[1], decode, 2 * i, op)?;
        out.extend([c, d]);
    }
    if decode {
        prepared = out.clone();
    }
    prepared_result(out, prepared, fillers, op)
}
/// Two-row groups, vertical digraphs; filler insertion consumes no original symbol.
pub fn seriated_playfair(
    text: &str,
    p: &SeriatedPlayfair,
    decode: bool,
) -> Result<Prepared, OperationError> {
    let op = if decode {
        "cipher.seriated_playfair.decode"
    } else {
        "cipher.seriated_playfair.encode"
    };
    if !(1..=4096).contains(&p.period) {
        return Err(invalid(op, "period", "period must be 1..4096"));
    }
    let period = usize::from(p.period);
    let square = Square::new(&p.square, op, "square")?;
    let (f, a) = check_fillers(p.filler, p.alternate, op)?;
    let x = letters(
        text,
        if decode { MAX_PREPARED } else { MAX_INPUT },
        op,
        false,
    )?;
    for (i, &c) in x.iter().enumerate() {
        square.pos(c, i, op)?;
    }
    let mut prepared = Vec::with_capacity(x.len());
    let mut fillers = Vec::new();
    if decode {
        if x.len() % 2 != 0 {
            return Err(invalid(
                op,
                "text",
                "Seriated Playfair ciphertext must be even",
            ));
        }
        prepared = x;
    } else {
        let mut cursor = 0;
        while cursor < x.len() {
            let remaining = &x[cursor..];
            // Full groups stay full. For a final short group find the smallest
            // top row allowing the greedy bottom row to consume its suffix.
            // Bounded binary search (period <=4096), no quadratic text insertion.
            let consumes = |m: usize| {
                let mut next = m;
                for &top in &remaining[..m] {
                    if next < remaining.len() && remaining[next] != top {
                        next += 1;
                    }
                }
                next == remaining.len()
            };
            let m = if remaining.len() >= 2 * period {
                period
            } else {
                let mut low = remaining.len().div_ceil(2);
                let mut high = period.min(remaining.len());
                if consumes(high) {
                    while low < high {
                        let mid = (low + high) / 2;
                        if consumes(mid) {
                            high = mid;
                        } else {
                            low = mid + 1;
                        }
                    }
                    low
                } else {
                    high
                }
            };
            let start = prepared.len();
            prepared.extend_from_slice(&remaining[..m]);
            let mut next = m;
            for &top in &remaining[..m] {
                if next == remaining.len() || remaining[next] == top {
                    fillers.push(prepared.len() as u32);
                    prepared.push(if top == f { a } else { f });
                } else {
                    prepared.push(remaining[next]);
                    next += 1;
                }
            }
            debug_assert_eq!(prepared.len() - start, 2 * m);
            cursor += next;
        }
    }
    let mut out = Vec::with_capacity(prepared.len());
    for block in prepared.chunks(2 * period) {
        let m = block.len() / 2;
        let mut bottom = Vec::with_capacity(m);
        for j in 0..m {
            let (c, d) = pair(&square, block[j], block[m + j], decode, j, op)?;
            out.push(c);
            bottom.push(d);
        }
        out.extend(bottom);
    }
    if decode {
        prepared = out.clone();
    }
    prepared_result(out, prepared, fillers, op)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct TriSquare {
    #[schemars(length(equal = 25))]
    pub first: String,
    #[schemars(length(equal = 25))]
    pub second: String,
    #[schemars(length(equal = 25))]
    pub middle: String,
    /// Empty chooses pair_index modulo 5, reset on each call.
    #[serde(default)]
    #[schemars(length(max = 32768), with = "Vec<Choice>")]
    pub rows: Vec<u8>,
    #[serde(default)]
    #[schemars(length(max = 32768), with = "Vec<Choice>")]
    pub columns: Vec<u8>,
}
/// Three-square geometry; encode choices are explicit or deterministic cycles.
pub fn tri_square(text: &str, p: &TriSquare, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.tri_square.decode"
    } else {
        "cipher.tri_square.encode"
    };
    let a = Square::new(&p.first, op, "first")?;
    let b = Square::new(&p.second, op, "second")?;
    let c = Square::new(&p.middle, op, "middle")?;
    let x = letters(text, if decode { 98304 } else { MAX_SYMBOLS }, op, false)?;
    let width = if decode { 3 } else { 2 };
    if x.len() % width != 0 {
        return Err(invalid(
            op,
            "text",
            "Tri-square requires even plaintext or complete ciphertext triples",
        ));
    }
    let pairs = x.len() / width;
    for stream in [&p.rows, &p.columns] {
        if stream.len() > 32768
            || stream.iter().any(|&v| v > 4)
            || (!stream.is_empty() && stream.len() != pairs)
        {
            return Err(invalid(
                op,
                "choices",
                "choice streams must be empty or contain exactly one 0..4 value per pair",
            ));
        }
    }
    let mut out = String::with_capacity(pairs * if decode { 2 } else { 3 });
    for (i, part) in x.chunks_exact(width).enumerate() {
        if decode {
            let (_, col) = a.pos(part[0], i * 3, op)?;
            let (row, _) = b.pos(part[2], i * 3 + 2, op)?;
            let (r, d) = c.pos(part[1], i * 3 + 1, op)?;
            out.push(char::from(a.at(r, col)));
            out.push(char::from(b.at(row, d)));
        } else {
            let (r, col) = a.pos(part[0], i * 2, op)?;
            let (row, d) = b.pos(part[1], i * 2 + 1, op)?;
            let u = p.rows.get(i).copied().map(usize::from).unwrap_or(i % 5);
            let v = p.columns.get(i).copied().map(usize::from).unwrap_or(i % 5);
            out.push(char::from(a.at(u, col)));
            out.push(char::from(c.at(r, d)));
            out.push(char::from(b.at(row, v)));
        }
    }
    Ok(out)
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = 2097152))]
pub struct Digrafid {
    #[schemars(length(equal = 27))]
    pub horizontal: String,
    #[schemars(length(equal = 27))]
    pub vertical: String,
    pub extra: char,
    #[schemars(range(min = 1, max = 4096))]
    pub period: u16,
    /// Required for odd input. Decode keeps this symbol visible.
    pub padding: Option<char>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DigrafidResult {
    #[schemars(length(max = 131072))]
    pub text: String,
    #[schemars(length(max = 131072))]
    pub prepared: String,
    #[schemars(range(min = 0, max = 1))]
    pub padding_count: u8,
}
/// Horizontal 3x9/vertical 9x3; coordinate rows are flattened per period.
pub fn digrafid(text: &str, p: &Digrafid, decode: bool) -> Result<DigrafidResult, OperationError> {
    let op = if decode {
        "cipher.digrafid.decode"
    } else {
        "cipher.digrafid.encode"
    };
    if p.extra.is_alphabetic() {
        return Err(invalid(op, "extra", "extra symbol must differ from A..Z"));
    }
    if !(1..=4096).contains(&p.period) {
        return Err(invalid(op, "period", "period must be 1..4096"));
    }
    let base = format!("{AZ}{}", p.extra);
    bound(op, "horizontal", p.horizontal.len(), 108)?;
    bound(op, "vertical", p.vertical.len(), 108)?;
    if p.horizontal.chars().count() != 27
        || p.vertical.chars().count() != 27
        || !p.horizontal.chars().all(|c| base.contains(c))
        || !p.vertical.chars().all(|c| base.contains(c))
    {
        return Err(invalid(
            op,
            "alphabets",
            "alphabets must permute A..Z plus extra",
        ));
    }
    crate::ciphers::mono::alphabet::Mapping::new(&base, &p.horizontal)
        .map_err(|_| invalid(op, "horizontal", "horizontal must permute A..Z plus extra"))?;
    crate::ciphers::mono::alphabet::Mapping::new(&base, &p.vertical)
        .map_err(|_| invalid(op, "vertical", "vertical must permute A..Z plus extra"))?;
    let h: Vec<char> = p.horizontal.chars().collect();
    let v: Vec<char> = p.vertical.chars().collect();
    let mut x: Vec<char> = Vec::new();
    bound(
        op,
        "text",
        text.len(),
        if decode {
            8 * MAX_SYMBOLS
        } else {
            4 * MAX_SYMBOLS
        },
    )?;
    for (offset, c) in text.char_indices() {
        if !base.contains(c) {
            return Err(input_error(op, offset));
        }
        x.push(c);
    }
    bound(
        op,
        "symbols",
        x.len(),
        if decode { 131072 } else { MAX_SYMBOLS },
    )?;
    if p.padding.is_some_and(|c| !base.contains(c)) {
        return Err(invalid(
            op,
            "padding",
            "padding must belong to the declared alphabet",
        ));
    }
    let padding_count = if !x.len().is_multiple_of(2) {
        if decode {
            return Err(invalid(
                op,
                "text",
                "Digrafid ciphertext must have even length",
            ));
        }
        x.push(
            p.padding
                .ok_or_else(|| invalid(op, "padding", "odd input requires explicit padding"))?,
        );
        1
    } else {
        0
    };
    let triple = |a: char, b: char| {
        let i = h.iter().position(|&c| c == a).expect("validated symbol");
        let j = v.iter().position(|&c| c == b).expect("validated symbol");
        [i % 9, (i / 9) * 3 + j % 3, j / 3]
    };
    let mut out = String::new();
    for block in x.chunks(2 * usize::from(p.period)) {
        let n = block.len() / 2;
        let triples: Vec<[usize; 3]> = block
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| triple(pair[0], pair[1]))
            .collect();
        let stream: Vec<usize> = if decode {
            triples.iter().flatten().copied().collect()
        } else {
            (0..3)
                .flat_map(|r| triples.iter().map(move |t| t[r]))
                .collect()
        };
        for i in 0..n {
            let t = if decode {
                [stream[i], stream[n + i], stream[2 * n + i]]
            } else {
                [stream[3 * i], stream[3 * i + 1], stream[3 * i + 2]]
            };
            out.push(h[(t[1] / 3) * 9 + t[0]]);
            out.push(v[t[2] * 3 + t[1] % 3]);
        }
    }
    let prepared = if decode {
        out.clone()
    } else {
        x.into_iter().collect()
    };
    Ok(DigrafidResult {
        text: out,
        prepared,
        padding_count,
    })
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct Choice(#[schemars(range(min = 0, max = 4))] u8);
