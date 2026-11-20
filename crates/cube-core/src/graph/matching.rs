//! Deterministic Hopcroft–Karp with an optional Kőnig cover.
use super::{Definition, Edge, Graph, Limits, MAX_EDGES, MAX_VERTICES};
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

const MAX_WORK: usize = 16 << 20;
const INF: usize = usize::MAX;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub left: Vec<String>,
    pub right: Vec<String>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub minimum_cover: bool,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct Match {
    pub left: String,
    pub right: String,
    pub edge: String,
}

#[derive(Serialize, JsonSchema)]
pub struct Duplicate {
    pub kept_edge: String,
    pub parallel_edges: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct Cover {
    pub left: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub matches: Vec<Match>,
    pub unmatched_left: Vec<String>,
    pub unmatched_right: Vec<String>,
    pub duplicates: Vec<Duplicate>,
    pub minimum_cover: Option<Cover>,
    pub work: usize,
}

#[derive(Clone, Copy)]
struct Neighbor {
    right: usize,
    edge: usize,
}

fn charge(work: &mut Work<'_>) -> Result<(), OperationError> {
    work.charge(1).map_err(Stop::error)
}

struct Matching {
    out: Vec<Vec<Neighbor>>,
    left: Vec<Option<Neighbor>>,
    right: Vec<Option<usize>>,
}

impl Matching {
    fn layers(&self, depth: &mut [usize], work: &mut Work<'_>) -> Result<usize, OperationError> {
        let mut queue = VecDeque::new();
        for (i, item) in depth.iter_mut().enumerate() {
            charge(work)?;
            *item = INF;
            if self.left[i].is_none() {
                *item = 0;
                queue.push_back(i);
            }
        }
        let mut shortest = INF;
        while let Some(left) = queue.pop_front() {
            charge(work)?;
            if depth[left] >= shortest {
                continue;
            }
            let layer = depth[left] + 1;
            for visit in &self.out[left] {
                charge(work)?;
                match self.right[visit.right] {
                    None => shortest = shortest.min(layer),
                    Some(next) if depth[next] == INF && layer < shortest => {
                        depth[next] = layer;
                        queue.push_back(next);
                    }
                    _ => (),
                }
            }
        }
        Ok(shortest)
    }

    fn augment(
        &mut self,
        root: usize,
        shortest: usize,
        cursor: &mut [usize],
        depth: &mut [usize],
        work: &mut Work<'_>,
    ) -> Result<(), OperationError> {
        let mut lefts = vec![root];
        let mut edges = Vec::new();
        while let Some(&left) = lefts.last() {
            charge(work)?;
            let Some(&edge) = self.out[left].get(cursor[left]) else {
                depth[left] = INF;
                lefts.pop();
                edges.pop();
                continue;
            };
            cursor[left] += 1;
            let layer = depth[left] + 1;
            match self.right[edge.right] {
                None if layer == shortest => {
                    edges.push(edge);
                    for (row, next) in lefts.into_iter().zip(edges) {
                        charge(work)?;
                        self.left[row] = Some(next);
                        self.right[next.right] = Some(row);
                    }
                    return Ok(());
                }
                Some(next) if depth[next] == layer && layer < shortest => {
                    edges.push(edge);
                    lefts.push(next);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn solve(&mut self, work: &mut Work<'_>) -> Result<(), OperationError> {
        let mut depth = vec![INF; self.left.len()];
        let mut cursor = vec![0; self.left.len()];
        loop {
            let shortest = self.layers(&mut depth, work)?;
            if shortest == INF {
                return Ok(());
            }
            cursor.fill(0);
            for root in 0..self.left.len() {
                charge(work)?;
                if self.left[root].is_none() && depth[root] == 0 {
                    self.augment(root, shortest, &mut cursor, &mut depth, work)?;
                }
            }
        }
    }

    fn cover(&self, request: &Request, work: &mut Work<'_>) -> Result<Cover, OperationError> {
        let mut left = vec![false; self.left.len()];
        let mut right = vec![false; self.right.len()];
        let mut queue = VecDeque::new();
        for (index, pair) in self.left.iter().enumerate() {
            charge(work)?;
            if pair.is_none() {
                left[index] = true;
                queue.push_back(index);
            }
        }
        while let Some(index) = queue.pop_front() {
            charge(work)?;
            for edge in &self.out[index] {
                charge(work)?;
                if self.left[index].is_some_and(|pair| pair.right == edge.right)
                    || std::mem::replace(&mut right[edge.right], true)
                {
                    continue;
                }
                if let Some(next) = self.right[edge.right]
                    && !std::mem::replace(&mut left[next], true)
                {
                    queue.push_back(next);
                }
            }
        }
        Ok(Cover {
            left: request
                .left
                .iter()
                .zip(left)
                .filter(|(_, seen)| !seen)
                .map(|(id, _)| id.clone())
                .collect(),
            right: request
                .right
                .iter()
                .zip(right)
                .filter(|(_, seen)| *seen)
                .map(|(id, _)| id.clone())
                .collect(),
        })
    }
}

/// Return a maximum matching, retaining the first edge of each parallel pair.
pub fn maximum(request: &Request) -> Result<Report, OperationError> {
    solve(request).map_err(|e| e.operation("graph.bipartite_matching"))
}

fn solve(request: &Request) -> Result<Report, OperationError> {
    let left = request.left.len();
    let right = request.right.len();
    if left.saturating_add(right) > MAX_VERTICES || request.edges.len() > MAX_EDGES {
        return Err(resource("matching exceeds 4096 vertices or 32768 edges"));
    }
    let mut cancelled = || false;
    let mut work = Work::new(request.max_work, MAX_WORK, &mut cancelled)?;
    work.charge(left + right + request.edges.len())
        .map_err(Stop::error)?;
    let graph = Graph::new(
        Definition {
            directed: true,
            vertices: request.left.iter().chain(&request.right).cloned().collect(),
            edges: request.edges.clone(),
        },
        Limits::default(),
    )?;
    let mut out = vec![Vec::new(); left];
    let mut unique = BTreeMap::new();
    let mut parallel = BTreeMap::<usize, Vec<String>>::new();
    for (edge, &(from, to)) in graph.ends.iter().enumerate() {
        charge(&mut work)?;
        if from >= left || to < left {
            return Err(invalid(
                "each matching edge must go from the left set to the right set",
            ));
        }
        if let Some(&first) = unique.get(&(from, to)) {
            parallel
                .entry(first)
                .or_default()
                .push(request.edges[edge].id.clone());
        } else {
            unique.insert((from, to), edge);
            out[from].push(Neighbor {
                right: to - left,
                edge,
            });
        }
    }
    let mut matching = Matching {
        out,
        left: vec![None; left],
        right: vec![None; right],
    };
    matching.solve(&mut work)?;
    let minimum_cover = request
        .minimum_cover
        .then(|| matching.cover(request, &mut work))
        .transpose()?;
    work.charge(left + right + parallel.len())
        .map_err(Stop::error)?;
    let mut matches = Vec::new();
    let mut unmatched_left = Vec::new();
    for (id, pair) in request.left.iter().zip(matching.left) {
        if let Some(next) = pair {
            matches.push(Match {
                left: id.clone(),
                right: request.right[next.right].clone(),
                edge: request.edges[next.edge].id.clone(),
            });
        } else {
            unmatched_left.push(id.clone());
        }
    }
    let unmatched_right = request
        .right
        .iter()
        .zip(matching.right)
        .filter(|(_, p)| p.is_none())
        .map(|(id, _)| id.clone())
        .collect();
    let duplicates = parallel
        .into_iter()
        .map(|(first, parallel_edges)| Duplicate {
            kept_edge: request.edges[first].id.clone(),
            parallel_edges,
        })
        .collect();
    Ok(Report {
        matches,
        unmatched_left,
        unmatched_right,
        duplicates,
        minimum_cover,
        work: work.used(),
    })
}
