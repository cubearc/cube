use super::*;

pub(super) struct Group {
    pub symbols: Vec<char>,
    pub domain: Vec<usize>,
}
pub(super) struct Prepared {
    pub alphabet: Vec<char>,
    pub indices: BTreeMap<char, usize>,
    pub text: String,
    pub observed: Vec<usize>,
    pub initial: Vec<Option<usize>>,
    pub words: Vec<Vec<char>>,
    pub groups: Vec<Group>,
    pub constraints: Vec<Constraint>,
    pub classes: Vec<DictionaryClass>,
    pub alias_uses: Vec<AliasUse>,
    pub conflict: bool,
    pub memory: usize,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Pattern {
    Variable(usize),
    Literal(char),
}
fn pattern(word: &[char], alphabet: &BTreeMap<char, usize>) -> Vec<Pattern> {
    let mut seen = BTreeMap::new();
    word.iter()
        .map(|&c| {
            if alphabet.contains_key(&c) {
                let next = seen.len();
                Pattern::Variable(*seen.entry(c).or_insert(next))
            } else {
                Pattern::Literal(c)
            }
        })
        .collect()
}
fn alphabet(r: &Request) -> Result<(Vec<char>, BTreeMap<char, usize>), OperationError> {
    let chars: Vec<_> = r.alphabet.chars().collect();
    if !(26..=64).contains(&chars.len()) || chars.iter().any(|c| c.is_whitespace()) {
        return Err(invalid(
            "alphabet requires 26..64 unique non-whitespace scalars",
        ));
    }
    let indices: BTreeMap<_, _> = chars.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    if indices.len() != chars.len() {
        return Err(invalid("duplicate alphabet scalar"));
    }
    Ok((chars, indices))
}
fn aliases(
    r: &Request,
    indices: &BTreeMap<char, usize>,
) -> Result<BTreeMap<char, char>, OperationError> {
    bound(r.aliases.len(), 256)?;
    let mut result = BTreeMap::new();
    for a in &r.aliases {
        if a.alias.is_whitespace()
            || indices.contains_key(&a.alias)
            || !indices.contains_key(&a.symbol)
            || result.insert(a.alias, a.symbol).is_some()
        {
            return Err(invalid(
                "aliases must uniquely map noncanonical non-whitespace scalars to alphabet symbols",
            ));
        }
    }
    Ok(result)
}
fn cribs(
    r: &Request,
    indices: &BTreeMap<char, usize>,
    n: usize,
) -> Result<(Vec<Option<usize>>, bool), OperationError> {
    bound(r.cribs.len(), 4096)?;
    let mut map = vec![None; n];
    let mut reverse = vec![None; n];
    let mut conflict = false;
    for pair in &r.cribs {
        let (&c, &p) = indices
            .get(&pair.cipher)
            .zip(indices.get(&pair.plain))
            .ok_or_else(|| invalid("cribs require canonical alphabet scalars"))?;
        conflict |= map[c].is_some_and(|v| v != p) || reverse[p].is_some_and(|v| v != c);
        map[c] = Some(p);
        reverse[p] = Some(c);
    }
    Ok((map, conflict))
}
impl Prepared {
    pub fn new(r: &Request, b: &mut Budget<'_>) -> Result<Self, OperationError> {
        text_bound(&r.ciphertext, 65536)?;
        let (alphabet, indices) = alphabet(r)?;
        let aliases = aliases(r, &indices)?;
        let (initial, conflict) = cribs(r, &indices, alphabet.len())?;
        // Includes caller request duplication, bounded scoring scratch, report and
        // serialization copies, stack, trees and all index allocations. Domain
        // edges are additionally reserved before allocation below.
        let request_bytes = serde_json::to_vec(r)
            .map_err(|_| invalid("invalid request"))?
            .len();
        let memory = (16 << 20) + request_bytes * 12;
        bound(memory, r.limits.max_memory_bytes)?;
        let mut p = Self {
            alphabet,
            indices,
            text: String::new(),
            observed: Vec::new(),
            initial,
            words: Vec::new(),
            groups: Vec::new(),
            constraints: Vec::new(),
            classes: Vec::new(),
            alias_uses: Vec::new(),
            conflict,
            memory,
        };
        p.canonicalize(r, &aliases)?;
        p.dictionary(r)?;
        if !b.take(
            r.ciphertext.chars().count().max(1)
                + r.cribs.len()
                + r.aliases.len()
                + r.dictionary.as_ref().map_or(0, |d| {
                    d.words
                        .iter()
                        .map(|w| w.chars().count().max(1))
                        .sum::<usize>()
                }),
        ) {
            return Ok(p);
        }
        p.index(r, b)?;
        Ok(p)
    }
    fn canonicalize(
        &mut self,
        r: &Request,
        aliases: &BTreeMap<char, char>,
    ) -> Result<(), OperationError> {
        let mut seen = BTreeSet::new();
        let mut uses = BTreeMap::new();
        for c in r.ciphertext.chars() {
            let normalized = aliases.get(&c).copied().unwrap_or(c);
            if let Some(&i) = self.indices.get(&normalized) {
                seen.insert(i);
            } else if matches!(r.outside, Outside::Reject) {
                return Err(invalid("ciphertext scalar outside alphabet/aliases"));
            }
            if aliases.contains_key(&c) {
                *uses.entry(c).or_insert(0) += 1;
            }
            self.text.push(normalized);
        }
        self.observed = seen.into_iter().collect();
        self.alias_uses = r
            .aliases
            .iter()
            .map(|a| AliasUse {
                alias: a.alias,
                symbol: a.symbol,
                occurrences: uses.get(&a.alias).copied().unwrap_or(0),
            })
            .collect();
        Ok(())
    }
    fn dictionary(&mut self, r: &Request) -> Result<(), OperationError> {
        let Some(d) = &r.dictionary else {
            return Ok(());
        };
        bound(d.words.len(), 4096)?;
        let mut unique = BTreeMap::new();
        let mut bytes = 0;
        for (i, word) in d.words.iter().enumerate() {
            let count = word.chars().count();
            if count == 0 || word.chars().any(char::is_whitespace) {
                return Err(invalid(
                    "dictionary requires nonempty exact whitespace-free tokens",
                ));
            }
            bound(count, 256)?;
            bytes += word.len();
            bound(bytes, MAX_TEXT)?;
            let next = self.classes.len();
            let class = *unique.entry(word).or_insert(next);
            if class == next {
                self.words.push(word.chars().collect());
                self.classes.push(DictionaryClass {
                    word: word.clone(),
                    input_indices: Vec::new(),
                });
            }
            self.classes[class].input_indices.push(i);
        }
        Ok(())
    }
    fn index(&mut self, r: &Request, b: &mut Budget<'_>) -> Result<(), OperationError> {
        if r.dictionary.is_none() {
            return Ok(());
        }
        let mut index: BTreeMap<Vec<Pattern>, Vec<usize>> = BTreeMap::new();
        for (i, w) in self.words.iter().enumerate() {
            if !b.take(w.len().max(1)) {
                return Ok(());
            }
            index.entry(pattern(w, &self.indices)).or_default().push(i);
        }
        let alias_map: BTreeMap<_, _> = r.aliases.iter().map(|a| (a.alias, a.symbol)).collect();
        let mut unique = BTreeMap::new();
        let mut offset = 0;
        for original in r.ciphertext.split_whitespace() {
            let relative = r.ciphertext[offset..].find(original).expect("source token");
            let start = offset + relative;
            offset = start + original.len();
            let token: String = original
                .chars()
                .map(|c| alias_map.get(&c).copied().unwrap_or(c))
                .collect();
            let next = self.groups.len();
            let group = *unique.entry(token.clone()).or_insert(next);
            if group == next {
                bound(next + 1, 4096)?;
                let symbols: Vec<_> = token.chars().collect();
                // A token longer than any allowed word has an empty domain.
                let support = index.get(&pattern(&symbols, &self.indices));
                let edges = support.map_or(0, Vec::len);
                self.memory += edges * 16 + symbols.len() * 32 + 256;
                bound(self.memory, r.limits.max_memory_bytes)?;
                let domain = support.cloned().unwrap_or_default();
                if !b.take(symbols.len().max(1) + domain.len()) {
                    return Ok(());
                }
                self.groups.push(Group { symbols, domain });
                self.constraints.push(Constraint {
                    token,
                    original_byte_offsets: Vec::new(),
                });
            }
            self.constraints[group].original_byte_offsets.push(start);
        }
        Ok(())
    }
}
