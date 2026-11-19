//! Exact ranking, finite enumeration, and canonical cycles.
use super::{Unsigned, Work, invalid, overflow, parameter, resource};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

fn factorials() -> [u128; 35] {
    let mut values = [1; 35];
    for i in 1..values.len() {
        values[i] = values[i - 1] * i as u128;
    }
    values
}

fn validate_lehmer(digits: &[u8]) -> Result<(), OperationError> {
    if digits.len() > 34
        || digits
            .iter()
            .enumerate()
            .any(|(i, &digit)| usize::from(digit) >= digits.len() - i)
    {
        return Err(invalid(
            "Lehmer digits must fit their decreasing position widths",
        ));
    }
    Ok(())
}

fn lehmer_rank(digits: &[u8], facts: &[u128; 35]) -> u128 {
    digits
        .iter()
        .enumerate()
        .map(|(i, &digit)| u128::from(digit) * facts[digits.len() - 1 - i])
        .sum()
}

fn lehmer_digits(mut rank: u128, size: usize, facts: &[u128; 35]) -> Vec<u8> {
    (0..size)
        .rev()
        .map(|i| {
            let digit = rank / facts[i];
            rank %= facts[i];
            digit as u8
        })
        .collect()
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum LehmerRequest {
    Rank { permutation: Vec<u8> },
    Unrank { size: usize, rank: Unsigned },
    FromDigits { digits: Vec<u8> },
}
#[derive(Serialize, JsonSchema)]
pub struct LehmerReport {
    pub rank: Unsigned,
    pub digits: Vec<u8>,
    pub permutation: Vec<u8>,
}

/// Convert a distinct zero-based permutation, a Lehmer code, and its rank.
pub fn lehmer(request: &LehmerRequest) -> Result<LehmerReport, OperationError> {
    let facts = factorials();
    let digits = match request {
        LehmerRequest::Rank { permutation } => {
            if permutation.len() > 34 {
                return Err(resource("permutation exceeds 34 elements"));
            }
            let mut remaining: Vec<_> = (0..permutation.len() as u8).collect();
            permutation
                .iter()
                .map(|item| {
                    let position = remaining
                        .binary_search(item)
                        .map_err(|_| invalid("permutation requires each ID 0..n-1 exactly once"))?;
                    remaining.remove(position);
                    Ok::<_, OperationError>(position as u8)
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        LehmerRequest::Unrank { size, rank } => {
            if *size > 34 || rank.0 >= facts[*size] {
                return Err(parameter("rank is outside the declared permutation size"));
            }
            lehmer_digits(rank.0, *size, &facts)
        }
        LehmerRequest::FromDigits { digits } => {
            validate_lehmer(digits)?;
            digits.clone()
        }
    };
    let rank = Unsigned(lehmer_rank(&digits, &facts));
    let mut remaining: Vec<_> = (0..digits.len() as u8).collect();
    let permutation = digits
        .iter()
        .map(|&digit| remaining.remove(usize::from(digit)))
        .collect();
    Ok(LehmerReport {
        rank,
        digits,
        permutation,
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "direction", rename_all = "snake_case", deny_unknown_fields)]
pub enum FactoradicRequest {
    Encode { value: Unsigned },
    Decode { digits: Vec<u8> },
}
#[derive(Serialize, JsonSchema)]
pub struct FactoradicReport {
    pub value: Unsigned,
    pub digits: Vec<u8>,
}

fn factoradic_digits(mut value: u128) -> Vec<u8> {
    let mut result = Vec::with_capacity(35);
    for radix in 1..=35 {
        result.push((value % radix) as u8);
        value /= radix;
        if value == 0 {
            break;
        }
    }
    result.reverse();
    result
}

fn factoradic_value(digits: &[u8]) -> Result<u128, OperationError> {
    if digits.is_empty() || digits.len() > 35 || digits.len() > 1 && digits[0] == 0 {
        return Err(invalid("factoradic requires 1 to 35 canonical digits"));
    }
    let mut value = 0_u128;
    for (i, &digit) in digits.iter().enumerate() {
        let radix = digits.len() - i;
        if usize::from(digit) >= radix {
            return Err(invalid("factoradic digit is outside its factorial weight"));
        }
        value = value
            .checked_mul(radix as u128)
            .and_then(|n| n.checked_add(u128::from(digit)))
            .ok_or_else(overflow)?;
    }
    Ok(value)
}

/// Convert a u128 to or from most-significant-first factoradic digits.
pub fn factoradic(request: &FactoradicRequest) -> Result<FactoradicReport, OperationError> {
    let value = match request {
        FactoradicRequest::Encode { value } => *value,
        FactoradicRequest::Decode { digits } => Unsigned(factoradic_value(digits)?),
    };
    Ok(FactoradicReport {
        value,
        digits: factoradic_digits(value.0),
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum LexRequest {
    Rank {
        alphabet: String,
        multiset: String,
        text: String,
    },
    Unrank {
        alphabet: String,
        multiset: String,
        rank: Unsigned,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct LexReport {
    pub rank: Unsigned,
    pub total: Unsigned,
    pub text: String,
}

fn multinomial(counts: &[usize], facts: &[u128; 35]) -> u128 {
    facts[counts.iter().sum::<usize>()] / counts.iter().map(|&count| facts[count]).product::<u128>()
}

fn block(total: u128, count: usize, remaining: usize) -> u128 {
    if count == 0 {
        return 0;
    }
    let (mut x, mut y) = (count, remaining);
    while y != 0 {
        (x, y) = (y, x % y);
    }
    (total / (remaining / x) as u128) * (count / x) as u128
}

struct Lexicon {
    symbols: Vec<char>,
    indexes: BTreeMap<char, usize>,
    counts: Vec<usize>,
    total: u128,
    remaining: usize,
}
impl Lexicon {
    fn new(alphabet: &str, multiset: &str) -> Result<Self, OperationError> {
        if alphabet.len() > 4096 || multiset.len() > 136 {
            return Err(resource("ranking source exceeds its byte limit"));
        }
        let symbols: Vec<_> = alphabet.chars().collect();
        let indexes: BTreeMap<_, _> = symbols
            .iter()
            .copied()
            .enumerate()
            .map(|(i, c)| (c, i))
            .collect();
        if indexes.len() != symbols.len() || symbols.len() > 34 {
            return Err(invalid(
                "alphabet requires at most 34 distinct ordered scalars",
            ));
        }
        let mut counts = vec![0; symbols.len()];
        let mut remaining = 0;
        for c in multiset.chars() {
            let &index = indexes
                .get(&c)
                .ok_or_else(|| invalid("multiset symbol is outside the alphabet"))?;
            counts[index] += 1;
            remaining += 1;
            if remaining > 34 {
                return Err(resource("multiset exceeds 34 scalars"));
            }
        }
        let total = multinomial(&counts, &factorials());
        Ok(Self {
            symbols,
            indexes,
            counts,
            total,
            remaining,
        })
    }
    fn block(&self, index: usize) -> u128 {
        block(self.total, self.counts[index], self.remaining)
    }
    fn take(&mut self, index: usize) {
        self.total = self.block(index);
        self.counts[index] -= 1;
        self.remaining -= 1;
    }
    fn rank(&mut self, text: &str) -> Result<u128, OperationError> {
        if text.len() > 136 {
            return Err(resource("ranking text exceeds 34 scalars"));
        }
        let mut rank = 0;
        for c in text.chars() {
            let &index = self
                .indexes
                .get(&c)
                .ok_or_else(|| invalid("text symbol is outside the alphabet"))?;
            if self.counts[index] == 0 {
                return Err(invalid("text does not match the declared multiset"));
            }
            rank += (0..index).map(|i| self.block(i)).sum::<u128>();
            self.take(index);
        }
        if self.remaining != 0 {
            return Err(invalid("text does not match the declared multiset"));
        }
        Ok(rank)
    }
    fn unrank(&mut self, mut rank: u128) -> Result<String, OperationError> {
        if rank >= self.total {
            return Err(parameter("rank exceeds the multiset permutation count"));
        }
        let mut text = String::new();
        while self.remaining != 0 {
            let index = (0..self.symbols.len())
                .find(|&i| {
                    let size = self.block(i);
                    if rank < size {
                        true
                    } else {
                        rank -= size;
                        false
                    }
                })
                .expect("validated rank has a block");
            text.push(self.symbols[index]);
            self.take(index);
        }
        Ok(text)
    }
}

/// Rank or unrank distinct multiset strings in the supplied scalar order.
pub fn lexicographic_rank(request: &LexRequest) -> Result<LexReport, OperationError> {
    let (alphabet, multiset) = match request {
        LexRequest::Rank {
            alphabet, multiset, ..
        }
        | LexRequest::Unrank {
            alphabet, multiset, ..
        } => (alphabet, multiset),
    };
    let mut alphabet = Lexicon::new(alphabet, multiset)?;
    let total = Unsigned(alphabet.total);
    let (text, rank) = match request {
        LexRequest::Rank { text, .. } => (text.clone(), Unsigned(alphabet.rank(text)?)),
        LexRequest::Unrank { rank, .. } => (alphabet.unrank(rank.0)?, *rank),
    };
    Ok(LexReport { rank, total, text })
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stop {
    Complete,
    ResultLimit,
    WorkLimit,
}
fn default_results() -> usize {
    100
}
fn default_work() -> usize {
    1 << 20
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PartitionRequest {
    Count {
        n: u16,
        #[serde(default)]
        max_part: Option<u16>,
        #[serde(default)]
        part_count: Option<u16>,
    },
    Enumerate {
        n: u16,
        #[serde(default)]
        max_part: Option<u16>,
        #[serde(default)]
        part_count: Option<u16>,
        #[serde(default = "default_results")]
        max_results: usize,
        #[serde(default = "default_work")]
        max_work: usize,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct PartitionReport {
    pub count: Unsigned,
    pub partitions: Option<Vec<Vec<u16>>>,
    pub complete: bool,
    pub stop_reason: Stop,
}

fn partitions_count(
    n: usize,
    max_part: usize,
    exact: Option<usize>,
    work: &mut Work,
) -> Result<u128, OperationError> {
    work.charge(n + 1)?;
    let mut table = vec![0u128; n + 1];
    table[0] = 1;
    if let Some(parts) = exact {
        if parts > n || max_part == 0 && n != 0 {
            return Ok(0);
        }
        let target = n - parts;
        // Remove one from each part. The residual Ferrers diagram fits in a
        // parts-by-(max_part - 1) rectangle. Use its Gaussian binomial product.
        for factor in 1..=parts {
            for sum in factor..=target {
                work.charge(1)?;
                table[sum] = table[sum]
                    .checked_add(table[sum - factor])
                    .ok_or_else(overflow)?;
            }
            let shift = max_part.saturating_sub(1) + factor;
            for sum in (shift..=target).rev() {
                work.charge(1)?;
                table[sum] = table[sum]
                    .checked_sub(table[sum - shift])
                    .ok_or_else(overflow)?;
            }
        }
        Ok(table[target])
    } else {
        for part in 1..=max_part.min(n) {
            for sum in part..=n {
                work.charge(1)?;
                table[sum] = table[sum]
                    .checked_add(table[sum - part])
                    .ok_or_else(overflow)?;
            }
        }
        Ok(table[n])
    }
}
struct Partitions {
    cap: usize,
    work: Work,
    rows: Vec<Vec<u16>>,
    prefix: Vec<u16>,
}
impl Partitions {
    fn visit(
        &mut self,
        remaining: u16,
        ceiling: u16,
        parts: Option<u16>,
    ) -> Result<(), OperationError> {
        if self.rows.len() == self.cap {
            return Ok(());
        }
        self.work.charge(1)?;
        if remaining == 0 {
            if parts.is_none_or(|n| n == 0) {
                self.work.charge(self.prefix.len())?;
                self.rows.push(self.prefix.clone());
            }
            return Ok(());
        }
        if parts.is_some_and(|n| n == 0 || n > remaining || n * ceiling < remaining) {
            return Ok(());
        }
        for next in (1..=remaining.min(ceiling)).rev() {
            self.prefix.push(next);
            self.visit(remaining - next, next, parts.map(|n| n - 1))?;
            self.prefix.pop();
            if self.rows.len() == self.cap {
                break;
            }
        }
        Ok(())
    }
}

/// Count exactly or enumerate a bounded reverse-lexicographic prefix.
pub fn partitions(request: &PartitionRequest) -> Result<PartitionReport, OperationError> {
    let (n, max_part, part_count) = match *request {
        PartitionRequest::Count {
            n,
            max_part,
            part_count,
        }
        | PartitionRequest::Enumerate {
            n,
            max_part,
            part_count,
            ..
        } => (n, max_part.unwrap_or(n).min(n), part_count),
    };
    if n > 256 {
        return Err(resource("partition count exceeds n=256"));
    }
    if matches!(request, PartitionRequest::Enumerate { .. }) && n > 64 {
        return Err(resource("partition enumeration exceeds n=64"));
    }
    let mut work = Work(1 << 20);
    let count = Unsigned(partitions_count(
        usize::from(n),
        usize::from(max_part),
        part_count.map(usize::from),
        &mut work,
    )?);
    let mut result = PartitionReport {
        count,
        partitions: None,
        complete: true,
        stop_reason: Stop::Complete,
    };
    if let PartitionRequest::Enumerate {
        max_results,
        max_work,
        ..
    } = *request
    {
        if max_results > 10_000 || max_work > 1 << 20 {
            return Err(parameter("partition enumeration limits are out of range"));
        }
        let mut state = Partitions {
            cap: max_results,
            work: Work(max_work.min(work.0)),
            rows: Vec::new(),
            prefix: Vec::new(),
        };
        let budget_hit = count.0 != 0 && state.visit(n, max_part, part_count).is_err();
        result.complete = count.0 == state.rows.len() as u128;
        result.stop_reason = stop(result.complete, budget_hit);
        result.partitions = Some(state.rows);
    }
    Ok(result)
}

fn stop(complete: bool, budget_hit: bool) -> Stop {
    if complete {
        Stop::Complete
    } else if budget_hit {
        Stop::WorkLimit
    } else {
        Stop::ResultLimit
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BruijnRequest {
    pub alphabet: String,
    pub order: usize,
    #[serde(default)]
    pub linearized: bool,
}
#[derive(Serialize, JsonSchema)]
pub struct BruijnReport {
    pub sequence: String,
    pub cyclic_symbols: usize,
    pub linearized: bool,
}
struct Necklaces<'a> {
    alphabet: &'a [char],
    order: usize,
    period: Vec<usize>,
    output: Vec<char>,
    work: Work,
}
impl Necklaces<'_> {
    fn visit(&mut self, at: usize, period: usize) -> Result<(), OperationError> {
        self.work.charge(1)?;
        if at > self.order {
            if self.order.is_multiple_of(period) {
                self.work.charge(period)?;
                self.output
                    .extend(self.period[1..=period].iter().map(|&i| self.alphabet[i]));
            }
        } else {
            let first = self.period[at - period];
            self.period[at] = first;
            self.visit(at + 1, period)?;
            for symbol in first + 1..self.alphabet.len() {
                self.period[at] = symbol;
                self.visit(at + 1, at)?;
            }
        }
        Ok(())
    }
}

/// Generate the FKM cycle starting at the lexicographic canonical rotation.
pub fn de_bruijn(request: &BruijnRequest) -> Result<BruijnReport, OperationError> {
    if !(1..=32).contains(&request.order) || request.alphabet.len() > 1024 {
        return Err(parameter("de Bruijn order or alphabet is out of range"));
    }
    let alphabet: Vec<_> = request.alphabet.chars().collect();
    if !(1..=256).contains(&alphabet.len())
        || alphabet.iter().copied().collect::<BTreeSet<_>>().len() != alphabet.len()
    {
        return Err(invalid(
            "de Bruijn alphabet requires 1..256 distinct scalars",
        ));
    }
    let span = alphabet
        .len()
        .checked_pow(request.order as u32)
        .filter(|&n| n <= 1 << 20)
        .ok_or_else(|| resource("cycle exceeds 1048576 symbols"))?;
    let mut state = Necklaces {
        alphabet: &alphabet,
        order: request.order,
        period: vec![0; request.order + 1],
        output: Vec::with_capacity(span),
        work: Work(16 << 20),
    };
    state.visit(1, 1)?;
    let mut sequence: String = state.output.iter().collect();
    if request.linearized {
        sequence.extend(state.output.iter().cycle().take(request.order - 1));
    }
    Ok(BruijnReport {
        sequence,
        cyclic_symbols: span,
        linearized: request.linearized,
    })
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermMode {
    Multiset,
    JohnsonTrotter,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PermRequest {
    pub tokens: Vec<u32>,
    pub mode: PermMode,
    #[serde(default = "default_results")]
    pub max_results: usize,
    #[serde(default = "default_work")]
    pub max_work: usize,
}
#[derive(Serialize, JsonSchema)]
pub struct TokenSources {
    pub token: u32,
    pub source_indices: Vec<usize>,
}
#[derive(Serialize, JsonSchema)]
pub struct PermReport {
    pub mode: PermMode,
    pub count: Unsigned,
    pub permutations: Vec<Vec<u32>>,
    pub sources: Vec<TokenSources>,
    pub complete: bool,
    pub stop_reason: Stop,
}
fn next_lex(values: &mut [u32]) -> bool {
    let Some(i) = values.windows(2).rposition(|p| p[0] < p[1]) else {
        return false;
    };
    let j = values
        .iter()
        .rposition(|&x| x > values[i])
        .expect("greater suffix");
    values.swap(i, j);
    values[i + 1..].reverse();
    true
}
fn next_mobile(values: &mut [u32], left: &mut [bool]) -> bool {
    let neighbour = |at: usize, is_left: bool| {
        if is_left {
            at.checked_sub(1)
        } else {
            Some(at + 1).filter(|&i| i < values.len())
        }
    };
    let best = values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            let j = neighbour(i, left[i])?;
            (v > values[j]).then_some((v, i, j))
        })
        .max();
    let Some((mobile, i, j)) = best else {
        return false;
    };
    values.swap(i, j);
    left.swap(i, j);
    for (value, direction) in values.iter().zip(left) {
        if *value > mobile {
            *direction = !*direction;
        }
    }
    true
}

/// Enumerate distinct multiset permutations or largest-mobile adjacent swaps.
pub fn permutation_list(request: &PermRequest) -> Result<PermReport, OperationError> {
    if request.tokens.len() > 32 || request.max_results > 10_000 || request.max_work > 1 << 20 {
        return Err(parameter(
            "permutation listing exceeds its dimension or budgets",
        ));
    }
    let mut sources = BTreeMap::<u32, Vec<usize>>::new();
    for (source, &token) in request.tokens.iter().enumerate() {
        sources.entry(token).or_default().push(source);
    }
    if matches!(request.mode, PermMode::JohnsonTrotter) && sources.len() != request.tokens.len() {
        return Err(invalid("Johnson–Trotter requires distinct tokens"));
    }
    let counts: Vec<_> = sources
        .values()
        .map(|indexes| indexes.len() as u64)
        .collect();
    let preflight = crate::numbers::combinatorics::multinomial(&counts, Default::default())?;
    let count = Unsigned(preflight.value);
    let mut values = request.tokens.clone();
    values.sort_unstable();
    let mut directions = vec![true; values.len()];
    let mut permutations = Vec::new();
    let mut work = Work(request.max_work.min((1 << 20) - preflight.work as usize));
    let mut budget_hit = false;
    while permutations.len() < request.max_results {
        if work.charge(1 + values.len()).is_err() {
            budget_hit = true;
            break;
        }
        permutations.push(values.clone());
        if permutations.len() as u128 == count.0 || permutations.len() == request.max_results {
            break;
        }
        if work.charge(1 + values.len() * 3).is_err() {
            budget_hit = true;
            break;
        }
        let advanced = match request.mode {
            PermMode::Multiset => next_lex(&mut values),
            PermMode::JohnsonTrotter => next_mobile(&mut values, &mut directions),
        };
        if !advanced {
            break;
        }
    }
    let complete = permutations.len() as u128 == count.0;
    let sources = sources
        .into_iter()
        .map(|(token, source_indices)| TokenSources {
            token,
            source_indices,
        })
        .collect();
    Ok(PermReport {
        mode: request.mode,
        count,
        permutations,
        sources,
        complete,
        stop_reason: stop(complete, budget_hit),
    })
}
