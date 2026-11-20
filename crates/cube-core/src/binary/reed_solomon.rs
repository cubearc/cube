//! Shortened systematic Reed–Solomon blocks with explicit GF(256) profiles.
use crate::{
    errors::OperationError,
    formal_work::{Work, invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::Deserialize;

const MAX_BLOCKS: usize = 4096;
const MAX_PARITY: usize = 64;
const MAX_LENGTH: usize = 255;
const INPUT_LIMIT: usize = 1 << 20;
const REPORT_LIMIT: usize = 4 << 20;
const WORK_LIMIT: usize = 16 << 20;

/// Field definitions use consecutive generator powers for the roots.
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(tag = "profile", rename_all = "snake_case", deny_unknown_fields)]
pub enum FieldProfile {
    #[serde(rename = "qr-rs-gf256")]
    QrRsGf256,
    Custom {
        #[schemars(range(min = 257, max = 511))]
        polynomial: u16,
        #[schemars(range(min = 1))]
        generator: u8,
        #[schemars(range(max = 254))]
        first_root: u8,
    },
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub field: FieldProfile,
    #[schemars(range(min = 2, max = 64))]
    pub parity_bytes: usize,
    #[schemars(range(max = 16777216))]
    pub work_limit: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct Damaged<'a> {
    pub codeword: &'a [u8],
    pub erasures: &'a [usize],
}

pub struct Encoded {
    pub parity: Vec<u8>,
    pub codeword: Vec<u8>,
}

pub struct Corrected {
    pub message: Vec<u8>,
    pub codeword: Vec<u8>,
    /// Changed byte positions in increasing codeword order.
    pub positions: Vec<usize>,
    pub initial_syndromes: Vec<u8>,
    pub verified_syndromes: Vec<u8>,
}

struct Field {
    exp: [u8; 510],
    log: [u8; 256],
    first: usize,
}

impl Field {
    fn new(profile: FieldProfile, work: &mut Work<'_>) -> Result<Self, OperationError> {
        let (polynomial, generator, first) = match profile {
            FieldProfile::QrRsGf256 => (0x11d, 2, 0),
            FieldProfile::Custom {
                polynomial,
                generator,
                first_root,
            } => (polynomial, generator, usize::from(first_root)),
        };
        if !(0x101..=0x1ff).contains(&polynomial) || polynomial & 1 == 0 {
            return Err(parameter(
                "field polynomial must be monic degree eight with constant term one",
            )
            .parameter("field.polynomial"));
        }
        if first == MAX_LENGTH || generator == 0 {
            return Err(
                parameter("field needs a nonzero generator and first_root in 0..255")
                    .parameter("field"),
            );
        }
        work.charge(MAX_LENGTH).map_err(|stop| stop.error())?;
        let mut field = Self {
            exp: [0; 510],
            log: [0; 256],
            first,
        };
        let mut seen = [false; 256];
        let mut value = 1u8;
        for i in 0..MAX_LENGTH {
            if value == 0 || seen[usize::from(value)] {
                return Err(parameter(
                    "generator does not cycle through all nonzero field elements",
                )
                .parameter("field.generator"));
            }
            seen[usize::from(value)] = true;
            field.exp[i] = value;
            field.exp[i + MAX_LENGTH] = value;
            field.log[usize::from(value)] = i as u8;
            value = polynomial_mul(value, generator, polynomial);
        }
        if value != 1 {
            return Err(
                parameter("field generator period must be 255").parameter("field.generator")
            );
        }
        Ok(field)
    }

    fn mul(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }
        self.exp[usize::from(self.log[usize::from(a)]) + usize::from(self.log[usize::from(b)])]
    }

    fn inv(&self, n: u8) -> u8 {
        self.exp[MAX_LENGTH - usize::from(self.log[usize::from(n)])]
    }

    fn evaluate(&self, descending: impl IntoIterator<Item = u8>, at: u8) -> u8 {
        descending
            .into_iter()
            .fold(0, |sum, next| self.mul(sum, at) ^ next)
    }

    fn generator(&self, parity: usize, work: &mut Work<'_>) -> Result<Vec<u8>, OperationError> {
        work.charge(parity * (parity + 1) / 2)
            .map_err(|stop| stop.error())?;
        let mut divisor = vec![0; parity + 1];
        divisor[0] = 1;
        for j in 0..parity {
            let root = self.exp[self.first + j];
            for i in (1..=j + 1).rev() {
                divisor[i] ^= self.mul(divisor[i - 1], root);
            }
        }
        Ok(divisor)
    }

    fn syndromes(
        &self,
        bytes: &[u8],
        parity: usize,
        work: &mut Work<'_>,
    ) -> Result<Vec<u8>, OperationError> {
        work.charge(bytes.len() * parity)
            .map_err(|stop| stop.error())?;
        Ok((0..parity)
            .map(|i| self.evaluate(bytes.iter().copied(), self.exp[self.first + i]))
            .collect())
    }

    fn location(&self, size: usize, position: usize) -> u8 {
        self.exp[size - 1 - position]
    }
}

fn polynomial_mul(a: u8, b: u8, polynomial: u16) -> u8 {
    let mut a = u16::from(a);
    let mut product = 0;
    for bit in 0..8 {
        if b & (1 << bit) != 0 {
            product ^= a;
        }
        a <<= 1;
        if a & 0x100 != 0 {
            a ^= polynomial;
        }
    }
    product as u8
}

fn preflight(
    blocks: impl ExactSizeIterator<Item = (usize, usize)>,
    parity: usize,
    encode: bool,
) -> Result<(), OperationError> {
    if !(2..=MAX_PARITY).contains(&parity) {
        return Err(parameter("parity_bytes must be between 2 and 64").parameter("parity_bytes"));
    }
    if blocks.len() > MAX_BLOCKS {
        return Err(resource("Reed–Solomon batch exceeds 4096 blocks"));
    }
    let mut input = 0;
    let mut report = 2;
    let base64 = |n: usize| n.div_ceil(3) * 4;
    for (len, erasures) in blocks {
        let valid = if encode {
            (1..=MAX_LENGTH - parity).contains(&len)
        } else {
            (parity + 1..=MAX_LENGTH).contains(&len)
        };
        if !valid {
            return Err(invalid("invalid Reed–Solomon shortened block length"));
        }
        input += len;
        if erasures > parity {
            return Err(uncorrectable());
        }
        report += if encode {
            64 + base64(parity) + base64(len + parity)
        } else {
            // Include a maximum of parity positions and both syndrome arrays.
            128 + base64(len) + base64(len - parity) + 12 * parity
        };
    }
    if input > INPUT_LIMIT || report > REPORT_LIMIT {
        return Err(resource(
            "Reed–Solomon batch exceeds its input or report budget",
        ));
    }
    // Each block uses at most 255 codeword bytes, 64 syndromes and bounded
    // polynomials. Only the admitted reports are retained between blocks.
    Ok(())
}

/// Encode shortened messages as systematic message followed by parity.
pub fn encode(blocks: &[&[u8]], options: Options) -> Result<Vec<Encoded>, OperationError> {
    preflight(
        blocks.iter().map(|block| (block.len(), 0)),
        options.parity_bytes,
        true,
    )?;
    let mut cancelled = || false;
    let mut work = Work::new(options.work_limit, WORK_LIMIT, &mut cancelled)?;
    let field = Field::new(options.field, &mut work)?;
    let divisor = field.generator(options.parity_bytes, &mut work)?;
    let parity = options.parity_bytes;
    blocks
        .iter()
        .map(|data| {
            work.charge(data.len() * parity)
                .map_err(|stop| stop.error())?;
            let mut remainder = vec![0; parity];
            for &byte in *data {
                let factor = byte ^ remainder[0];
                remainder.rotate_left(1);
                remainder[parity - 1] = 0;
                for (term, &coefficient) in remainder.iter_mut().zip(&divisor[1..]) {
                    *term ^= field.mul(factor, coefficient);
                }
            }
            let mut codeword = Vec::with_capacity(data.len() + parity);
            codeword.extend_from_slice(data);
            codeword.extend_from_slice(&remainder);
            Ok(Encoded {
                parity: remainder,
                codeword,
            })
        })
        .collect()
}

fn uncorrectable() -> OperationError {
    invalid("Reed–Solomon block is uncorrectable")
}

fn erasure_locations(
    field: &Field,
    block: Damaged<'_>,
) -> Result<Vec<(usize, u8)>, OperationError> {
    let n = block.codeword.len();
    let mut seen = [false; MAX_LENGTH];
    let mut locations = Vec::with_capacity(block.erasures.len());
    for &pos in block.erasures {
        if pos >= n || seen[pos] {
            return Err(invalid(
                "Reed–Solomon erasure positions must be distinct and within the block",
            ));
        }
        seen[pos] = true;
        locations.push((pos, field.location(n, pos)));
    }
    Ok(locations)
}

/// Eliminate each known location from the recurrence before locating unknown errors.
fn eliminate(
    field: &Field,
    syndromes: &[u8],
    erasures: &[(usize, u8)],
    work: &mut Work<'_>,
) -> Result<Vec<u8>, OperationError> {
    work.charge(syndromes.len() * erasures.len())
        .map_err(|stop| stop.error())?;
    let mut reduced = syndromes.to_vec();
    for &(_, x) in erasures {
        for i in 0..reduced.len() - 1 {
            reduced[i] = reduced[i + 1] ^ field.mul(x, reduced[i]);
        }
        reduced.pop();
    }
    Ok(reduced)
}

fn berlekamp_massey(
    field: &Field,
    seq: &[u8],
    work: &mut Work<'_>,
) -> Result<Vec<u8>, OperationError> {
    let mut current = vec![0; seq.len() + 1];
    let mut previous = current.clone();
    current[0] = 1;
    previous[0] = 1;
    let (mut degree, mut offset, mut scale) = (0, 1, 1);
    for (i, &sample) in seq.iter().enumerate() {
        work.charge(degree).map_err(|stop| stop.error())?;
        let mut delta = sample;
        for j in 1..=degree {
            delta ^= field.mul(current[j], seq[i - j]);
        }
        if delta == 0 {
            offset += 1;
            continue;
        }
        work.charge(seq.len() + 2 - offset)
            .map_err(|stop| stop.error())?;
        let prior = current.clone();
        let factor = field.mul(delta, field.inv(scale));
        for (term, &coefficient) in current[offset..].iter_mut().zip(&previous) {
            *term ^= field.mul(factor, coefficient);
        }
        if 2 * degree <= i {
            degree = i + 1 - degree;
            previous = prior;
            scale = delta;
            offset = 1;
        } else {
            offset += 1;
        }
    }
    current.truncate(degree + 1);
    Ok(current)
}

fn locate(
    field: &Field,
    n: usize,
    reduced: &[u8],
    mut locations: Vec<(usize, u8)>,
    work: &mut Work<'_>,
) -> Result<Vec<(usize, u8)>, OperationError> {
    let locator = berlekamp_massey(field, reduced, work)?;
    let degree = locator.len() - 1;
    if 2 * degree > reduced.len() {
        return Err(uncorrectable());
    }
    let start = locations.len();
    work.charge(n * (degree + 1)).map_err(|stop| stop.error())?;
    for i in 0..n {
        let x = field.location(n, i);
        if field.evaluate(locator.iter().rev().copied(), field.inv(x)) == 0 {
            if locations.iter().any(|&(pos, _)| pos == i) {
                return Err(uncorrectable());
            }
            locations.push((i, x));
        }
    }
    if locations.len() != start + degree {
        return Err(uncorrectable());
    }
    locations.sort_unstable_by_key(|&(pos, _)| pos);
    Ok(locations)
}

fn repair(
    field: &Field,
    codeword: &mut [u8],
    syndromes: &[u8],
    locations: &[(usize, u8)],
    work: &mut Work<'_>,
) -> Result<Vec<usize>, OperationError> {
    let m = locations.len();
    work.charge(m * (syndromes.len() + 4 * m + 4))
        .map_err(|stop| stop.error())?;
    let mut locator = vec![1u8];
    for &(_, x) in locations {
        locator.push(0);
        for j in (1..locator.len()).rev() {
            locator[j] ^= field.mul(locator[j - 1], x);
        }
    }
    // Omega is the low-degree part of S(z) * Λ(z). Forney evaluates it and
    // the characteristic-two derivative of Λ at the inverse locations.
    let mut evaluator = vec![0; m];
    for i in 0..m {
        for j in 0..=i {
            evaluator[i] ^= field.mul(locator[j], syndromes[i - j]);
        }
    }
    let derivative: Vec<_> = locator
        .iter()
        .skip(1)
        .enumerate()
        .map(|(i, &v)| if i % 2 == 0 { v } else { 0 })
        .collect();
    let mut changed = Vec::with_capacity(m);
    for &(pos, x) in locations {
        let inv = field.inv(x);
        let denominator = field.evaluate(derivative.iter().rev().copied(), inv);
        if denominator == 0 {
            return Err(uncorrectable());
        }
        let numerator = field.evaluate(evaluator.iter().rev().copied(), inv);
        let power = (256 - field.first) % MAX_LENGTH;
        let factor = field.exp[power * usize::from(field.log[usize::from(x)]) % MAX_LENGTH];
        let magnitude = field.mul(field.mul(numerator, field.inv(denominator)), factor);
        if magnitude != 0 {
            codeword[pos] ^= magnitude;
            changed.push(pos);
        }
    }
    Ok(changed)
}

fn decode_block(
    field: &Field,
    block: Damaged<'_>,
    parity: usize,
    work: &mut Work<'_>,
) -> Result<Corrected, OperationError> {
    let erasures = erasure_locations(field, block)?;
    let initial_syndromes = field.syndromes(block.codeword, parity, work)?;
    let mut codeword = block.codeword.to_vec();
    let (positions, verified_syndromes) = if initial_syndromes.iter().all(|&v| v == 0) {
        (Vec::new(), initial_syndromes.clone())
    } else {
        let reduced = eliminate(field, &initial_syndromes, &erasures, work)?;
        let locations = locate(field, codeword.len(), &reduced, erasures, work)?;
        let positions = repair(field, &mut codeword, &initial_syndromes, &locations, work)?;
        let verified = field.syndromes(&codeword, parity, work)?;
        if verified.iter().any(|&v| v != 0) {
            return Err(uncorrectable());
        }
        (positions, verified)
    };
    let message = codeword[..codeword.len() - parity].to_vec();
    Ok(Corrected {
        message,
        codeword,
        positions,
        initial_syndromes,
        verified_syndromes,
    })
}

/// Correct a complete batch or fail without returning a partial result.
pub fn decode(blocks: &[Damaged<'_>], options: Options) -> Result<Vec<Corrected>, OperationError> {
    preflight(
        blocks
            .iter()
            .map(|block| (block.codeword.len(), block.erasures.len())),
        options.parity_bytes,
        false,
    )?;
    let mut cancelled = || false;
    let mut work = Work::new(options.work_limit, WORK_LIMIT, &mut cancelled)?;
    let field = Field::new(options.field, &mut work)?;
    blocks
        .iter()
        .map(|&block| decode_block(&field, block, options.parity_bytes, &mut work))
        .collect()
}
