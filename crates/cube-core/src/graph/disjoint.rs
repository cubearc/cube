//! Batch disjoint sets with canonical representatives independent of tree roots.
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const CAP: usize = 1 << 16;
const MAX_WORK: usize = 4 << 20;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Union { left: u32, right: u32 },
    Find { id: u32 },
    SameComponent { left: u32, right: u32 },
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub ids: Vec<u32>,
    pub actions: Vec<Action>,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Answer {
    Find {
        action_index: usize,
        representative: u32,
    },
    SameComponent {
        action_index: usize,
        same: bool,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub components: Vec<Vec<u32>>,
    pub answers: Vec<Answer>,
    pub work: usize,
}

struct Sets {
    parent: Vec<usize>,
    size: Vec<usize>,
    first: Vec<usize>,
}

impl Sets {
    fn new(count: usize) -> Self {
        Self {
            parent: (0..count).collect(),
            size: vec![1; count],
            first: (0..count).collect(),
        }
    }

    fn root(&mut self, mut current: usize, work: &mut Work<'_>) -> Result<usize, OperationError> {
        // Path halving compresses the route without a second stack.
        loop {
            work.charge(1).map_err(Stop::error)?;
            let parent = self.parent[current];
            if parent == current {
                return Ok(current);
            }
            work.charge(1).map_err(Stop::error)?;
            let grandparent = self.parent[parent];
            self.parent[current] = grandparent;
            current = grandparent;
        }
    }

    fn union(&mut self, mut a: usize, mut b: usize) {
        if a == b {
            return;
        }
        if self.size[a] < self.size[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parent[b] = a;
        self.size[a] += self.size[b];
        self.first[a] = self.first[a].min(self.first[b]);
    }
}

/// Apply an isolated ordered batch and return membership in source-index order.
pub fn disjoint_sets(request: &Request) -> Result<Report, OperationError> {
    run(request).map_err(|e| e.operation("graph.disjoint_sets"))
}

fn run(request: &Request) -> Result<Report, OperationError> {
    if request.ids.len() > CAP || request.actions.len() > 1 << 20 {
        return Err(resource(
            "disjoint sets exceed 65536 IDs or 1048576 actions",
        ));
    }
    crate::formal_work::check_bytes(request, 8 << 20)?;
    let mut cancelled = || false;
    let mut work = Work::new(request.max_work, MAX_WORK, &mut cancelled)?;
    work.charge(request.ids.len()).map_err(Stop::error)?;
    let index: BTreeMap<_, _> = request
        .ids
        .iter()
        .copied()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();
    if index.len() != request.ids.len() {
        return Err(invalid("disjoint-set IDs must be unique"));
    }
    let find = |id: u32| {
        index
            .get(&id)
            .copied()
            .ok_or_else(|| invalid("disjoint-set action refers to an unknown ID"))
    };
    let mut sets = Sets::new(index.len());
    let mut answers = Vec::new();
    for (action_index, action) in request.actions.iter().enumerate() {
        work.charge(1).map_err(Stop::error)?;
        match *action {
            Action::Union { left, right } => {
                let a = sets.root(find(left)?, &mut work)?;
                let b = sets.root(find(right)?, &mut work)?;
                sets.union(a, b);
            }
            Action::Find { id } => {
                let a = sets.root(find(id)?, &mut work)?;
                answers.push(Answer::Find {
                    action_index,
                    representative: request.ids[sets.first[a]],
                });
            }
            Action::SameComponent { left, right } => {
                let same =
                    sets.root(find(left)?, &mut work)? == sets.root(find(right)?, &mut work)?;
                answers.push(Answer::SameComponent { action_index, same });
            }
        }
        // Limit retained answers before allocation growth, including room for
        // the largest component-ID list and every numeric JSON delimiter.
        if answers.len() > ((8 << 20) - (CAP * 16) - 128) / 80 {
            return Err(resource(
                "disjoint-set evidence exceeds the report byte budget",
            ));
        }
    }
    let mut groups = vec![Vec::new(); request.ids.len()];
    for (i, &id) in request.ids.iter().enumerate() {
        let root = sets.root(i, &mut work)?;
        groups[sets.first[root]].push(id);
    }
    Ok(Report {
        components: groups.into_iter().filter(|g| !g.is_empty()).collect(),
        answers,
        work: work.used(),
    })
}
