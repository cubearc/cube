use super::*;
use crate::ciphers::mono::alphabet::Mapping;

struct Search<'a, 'b, 'c> {
    request: &'a Request,
    prepared: &'a prepare::Prepared,
    model: Option<&'a scoring::Prepared>,
    budget: &'b mut Budget<'c>,
    candidates: &'b mut Vec<Candidate>,
    retained_bytes: usize,
}
// A domain support test charges every examined scalar, including failed choices.
fn extend(
    p: &prepare::Prepared,
    map: &[Option<usize>],
    word: &[char],
    target: &[char],
    b: &mut Budget<'_>,
) -> Option<Vec<Option<usize>>> {
    if !b.take(word.len().max(1)) {
        return None;
    }
    let mut next = map.to_vec();
    for (&cipher, &plain) in word.iter().zip(target) {
        if let Some(&c) = p.indices.get(&cipher) {
            let v = p.indices[&plain];
            if next[c].is_some_and(|x| x != v)
                || next
                    .iter()
                    .enumerate()
                    .any(|(i, &x)| i != c && x == Some(v))
            {
                return None;
            }
            next[c] = Some(v);
        }
    }
    Some(next)
}
impl Search<'_, '_, '_> {
    fn visit(&mut self, map: Vec<Option<usize>>) -> Result<(), OperationError> {
        if !self.budget.status.complete || !self.budget.take(1) {
            return Ok(());
        }
        if self.request.dictionary.is_some() {
            self.dictionary(map)
        } else {
            self.letters(map)
        }
    }
    fn letters(&mut self, map: Vec<Option<usize>>) -> Result<(), OperationError> {
        let p = self.prepared;
        let Some(&c) = p.observed.iter().find(|&&c| map[c].is_none()) else {
            return self.emit(&map);
        };
        for v in 0..p.alphabet.len() {
            if !self.budget.take(1) {
                break;
            }
            if map.contains(&Some(v)) {
                continue;
            }
            let mut next = map.clone();
            next[c] = Some(v);
            self.visit(next)?;
            if !self.budget.status.complete {
                break;
            }
        }
        Ok(())
    }
    fn dictionary(&mut self, map: Vec<Option<usize>>) -> Result<(), OperationError> {
        let p = self.prepared;
        let mut best: Option<(usize, Vec<usize>)> = None;
        for (group, g) in p.groups.iter().enumerate() {
            let unresolved = g
                .symbols
                .iter()
                .any(|c| p.indices.get(c).is_some_and(|&i| map[i].is_none()));
            let mut compatible = Vec::new();
            for &word in &g.domain {
                if extend(p, &map, &g.symbols, &p.words[word], self.budget).is_some() {
                    compatible.push(word);
                }
                if !self.budget.status.complete {
                    return Ok(());
                }
                if !unresolved && !compatible.is_empty() {
                    break;
                }
            }
            if compatible.is_empty() {
                return Ok(());
            }
            if unresolved
                && best
                    .as_ref()
                    .is_none_or(|old| compatible.len() < old.1.len())
            {
                best = Some((group, compatible));
            }
        }
        if let Some((group, domain)) = best {
            for word in domain {
                let g = &p.groups[group];
                let Some(next) = extend(p, &map, &g.symbols, &p.words[word], self.budget) else {
                    break;
                };
                self.visit(next)?;
                if !self.budget.status.complete {
                    break;
                }
            }
            Ok(())
        } else {
            self.emit(&map)
        }
    }
    fn emit(&mut self, map: &[Option<usize>]) -> Result<(), OperationError> {
        if self.candidates.len() == self.request.limits.max_results {
            self.budget.stop(Stop::ResultLimit);
            return Ok(());
        }
        let p = self.prepared;
        if !self
            .budget
            .take(p.text.chars().count().max(1) + p.alphabet.len())
        {
            return Ok(());
        }
        let mut free_plain = Vec::new();
        let mut free_cipher = Vec::new();
        for (i, &c) in p.alphabet.iter().enumerate() {
            if map[i].is_none() {
                free_cipher.push(c);
            }
            if !map.contains(&Some(i)) {
                free_plain.push(c);
            }
        }
        let mut available = free_plain.iter();
        let key: String = map
            .iter()
            .map(|v| {
                v.map(|i| p.alphabet[i])
                    .unwrap_or_else(|| *available.next().expect("bijection"))
            })
            .collect();
        let plaintext = Mapping::new(&self.request.alphabet, &key)?.encode(&p.text, "preserve")?;
        let score = if let Some(m) = self.model {
            let Some(s) = m.score(&plaintext, self.candidates.len(), self.budget) else {
                return Ok(());
            };
            Some(s)
        } else {
            None
        };
        let mut count = num_bigint::BigUint::from(1u8);
        for n in 2..=free_cipher.len() {
            count *= n;
        }
        let mut word_choices = Vec::new();
        for g in &p.groups {
            if !self.budget.take(g.symbols.len().max(1)) {
                return Ok(());
            }
            let token: String = g
                .symbols
                .iter()
                .map(|&c| {
                    p.indices
                        .get(&c)
                        .map(|&i| p.alphabet[map[i].expect("observed mapped")])
                        .unwrap_or(c)
                })
                .collect();
            word_choices.push(
                g.domain
                    .iter()
                    .copied()
                    .find(|&i| p.classes[i].word == token)
                    .expect("supported word"),
            );
        }
        let candidate = Candidate {
            plaintext,
            canonical_key: key,
            mapping: map
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| {
                    v.map(|j| Pair {
                        cipher: p.alphabet[i],
                        plain: p.alphabet[j],
                    })
                })
                .collect(),
            free_cipher: free_cipher.iter().collect(),
            free_plain: free_plain.iter().collect(),
            key_count: count.to_string(),
            word_choices,
            score,
        };
        let value = serde_json::to_value(&candidate).map_err(|_| invalid("invalid candidate"))?;
        let bytes = serde_json::to_vec(&crate::operations::contracts::canonical_value(value))
            .map_err(|_| invalid("invalid candidate"))?
            .len();
        if self.retained_bytes + bytes > 3 << 20 {
            self.budget.stop(Stop::ResultLimit);
            return Ok(());
        }
        self.retained_bytes += bytes;
        self.candidates.push(candidate);
        Ok(())
    }
}
pub(super) fn run(
    r: &Request,
    p: &prepare::Prepared,
    model: Option<&scoring::Prepared>,
    b: &mut Budget<'_>,
    candidates: &mut Vec<Candidate>,
) -> Result<(), OperationError> {
    Search {
        request: r,
        prepared: p,
        model,
        budget: b,
        candidates,
        retained_bytes: 0,
    }
    .visit(p.initial.clone())
}
