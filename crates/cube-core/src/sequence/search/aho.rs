//! Sparse Aho–Corasick automaton with preflighted fallible arena allocations.
use super::*;

const NONE: u32 = u32::MAX;
const AGGREGATE: usize = 1 << 20;
const MAX_PATTERNS: usize = 4096;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MultiRequest {
    pub haystack: String,
    #[schemars(length(min = 1, max = 4096))]
    pub patterns: Vec<String>,
    #[serde(default = "default_results")]
    #[schemars(range(max = 65536))]
    pub max_results: usize,
    #[serde(default = "yes")]
    pub overlaps: bool,
}
const fn yes() -> bool {
    true
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
pub struct MultiMatch {
    pub pattern_id: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct MultiReport {
    pub count: usize,
    pub complete: bool,
    pub matches: Vec<MultiMatch>,
}

#[derive(Clone, Copy)]
struct Node {
    edge: u32,
    fail: u32,
    terminal: u32,
    output: u32,
}
impl Node {
    const EMPTY: Self = Self {
        edge: NONE,
        fail: 0,
        terminal: NONE,
        output: NONE,
    };
}
struct Edge {
    child: u32,
    next: u32,
    symbol: u8,
}

struct Automaton {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    next_pattern: Vec<u32>,
}

fn arena<T>(capacity: usize) -> Result<Vec<T>, OperationError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(capacity)
        .map_err(|_| resource("automaton scratch allocation failed"))?;
    Ok(result)
}

impl Automaton {
    fn child(
        &self,
        state: u32,
        symbol: u8,
        work: &mut Work,
    ) -> Result<Option<u32>, OperationError> {
        let mut next = self.nodes[state as usize].edge;
        while next != NONE {
            work.charge(1)?;
            let edge = &self.edges[next as usize];
            if edge.symbol == symbol {
                return Ok(Some(edge.child));
            }
            next = edge.next;
        }
        Ok(None)
    }

    fn advance(&self, mut state: u32, symbol: u8, work: &mut Work) -> Result<u32, OperationError> {
        loop {
            work.charge(1)?;
            if let Some(child) = self.child(state, symbol, work)? {
                return Ok(child);
            }
            if state == 0 {
                return Ok(0);
            }
            state = self.nodes[state as usize].fail;
        }
    }

    fn add(&mut self, pattern: &[u8], id: usize, work: &mut Work) -> Result<(), OperationError> {
        let mut state = 0;
        for &symbol in pattern {
            work.charge(1)?;
            state = if let Some(child) = self.child(state, symbol, work)? {
                child
            } else {
                let child = self.nodes.len() as u32;
                let node = &mut self.nodes[state as usize];
                self.edges.push(Edge {
                    child,
                    next: node.edge,
                    symbol,
                });
                node.edge = (self.edges.len() - 1) as u32;
                self.nodes.push(Node::EMPTY);
                child
            };
        }
        let node = &mut self.nodes[state as usize];
        self.next_pattern[id] = node.terminal;
        node.terminal = id as u32;
        Ok(())
    }

    fn links(&mut self, work: &mut Work) -> Result<(), OperationError> {
        let mut queue = arena(self.nodes.len())?;
        queue.push(0);
        let mut i = 0;
        while i < queue.len() {
            let state = queue[i];
            i += 1;
            let node = self.nodes[state as usize];
            let mut edges = node.edge;
            while edges != NONE {
                work.charge(1)?;
                let edge = &self.edges[edges as usize];
                let (child, next, symbol) = (edge.child, edge.next, edge.symbol);
                let fail = if state == 0 {
                    0
                } else {
                    self.advance(node.fail, symbol, work)?
                };
                let target = self.nodes[fail as usize];
                self.nodes[child as usize].fail = fail;
                self.nodes[child as usize].output = if target.terminal != NONE {
                    fail
                } else {
                    target.output
                };
                queue.push(child);
                edges = next;
            }
        }
        Ok(())
    }
}

fn decode_patterns(input: &[String]) -> Result<Vec<Vec<u8>>, OperationError> {
    if !(1..=MAX_PATTERNS).contains(&input.len()) {
        return Err(parameter("provide 1..=4096 nonempty patterns"));
    }
    let mut remaining = AGGREGATE;
    let mut patterns = Vec::with_capacity(input.len());
    for wire in input {
        let decoded = bytes(wire, PATTERN.min(remaining))?;
        if decoded.is_empty() {
            return Err(invalid("empty multipattern entry"));
        }
        remaining -= decoded.len();
        patterns.push(decoded);
    }
    Ok(patterns)
}

fn search(
    machine: &Automaton,
    h: &[u8],
    patterns: &[Vec<u8>],
    input: &MultiRequest,
    work: &mut Work,
) -> Result<MultiReport, OperationError> {
    let mut hits = MultiReport {
        count: 0,
        complete: true,
        matches: Vec::new(),
    };
    let mut cursor = 0;
    let mut state = 0;
    for (index, &byte) in h.iter().enumerate() {
        let end = index + 1;
        state = machine.advance(state, byte, work)?;
        let mut next = if machine.nodes[state as usize].terminal != NONE {
            state
        } else {
            machine.nodes[state as usize].output
        };
        while next != NONE {
            let node = machine.nodes[next as usize];
            let mut entry = node.terminal;
            if entry == NONE {
                break;
            }
            let start = end - patterns[entry as usize].len();
            while entry != NONE {
                work.charge(1)?;
                if input.overlaps || start >= cursor {
                    hits.count += 1;
                    if hits.matches.len() < input.max_results {
                        hits.matches.push(MultiMatch {
                            pattern_id: entry as usize,
                            start,
                            end,
                        });
                    }
                    if !input.overlaps {
                        cursor = end;
                    }
                }
                entry = machine.next_pattern[entry as usize];
            }
            next = node.output;
        }
    }
    hits.complete = hits.count == hits.matches.len();
    Ok(hits)
}

/// Search ordered literal byte patterns with overlapping matches by default.
pub fn multi_search(input: &MultiRequest) -> Result<MultiReport, OperationError> {
    matches_limit(input.max_results)?;
    let patterns = decode_patterns(&input.patterns)?;
    let symbols = patterns.iter().map(Vec::len).sum::<usize>();
    // One 16-byte node and one 12-byte edge per input pattern byte,
    // a four-byte BFS slot per node, and bounded input and result buffers.
    let mut machine = Automaton {
        nodes: arena(symbols + 1)?,
        edges: arena(symbols)?,
        next_pattern: vec![NONE; patterns.len()],
    };
    machine.nodes.push(Node::EMPTY);
    let mut work = Work(TRANSITIONS);
    for (id, pattern) in patterns.iter().enumerate().rev() {
        machine.add(pattern, id, &mut work)?;
    }
    machine.links(&mut work)?;
    search(
        &machine,
        &bytes(&input.haystack, HAYSTACK)?,
        &patterns,
        input,
        &mut work,
    )
}
