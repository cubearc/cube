//! Iterative depth-first traversal, Kahn ordering, and Tarjan components.
use super::{Budget, Definition, Graph, Limits, Path};
use crate::{errors::OperationError, formal_work::parameter};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BinaryHeap};

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    DepthFirst,
    Topological,
    StrongComponents,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub graph: Definition,
    pub mode: Mode,
    pub start: Option<String>,
    pub max_work: Option<u64>,
}

#[derive(Serialize, JsonSchema)]
pub struct Visit {
    pub vertex: String,
    pub parent_edge: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ResultKind {
    DepthFirst {
        discovery: Vec<Visit>,
        finish: Vec<String>,
    },
    Topological {
        order: Option<Vec<String>>,
        cycle: Option<Path>,
    },
    StrongComponents {
        components: Vec<Vec<String>>,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub result: ResultKind,
    pub work: u64,
}

#[derive(Clone, Copy)]
struct Frame {
    vertex: usize,
    next: usize,
}

impl Frame {
    fn new(vertex: usize) -> Self {
        Self { vertex, next: 0 }
    }
}

impl Graph {
    fn vertex_name(&self, index: usize) -> String {
        self.definition.vertices[index].clone()
    }

    fn depth_first(
        &self,
        start: Option<usize>,
        budget: &mut Budget,
    ) -> Result<ResultKind, OperationError> {
        let mut seen = vec![false; self.out.len()];
        let mut discovery = Vec::with_capacity(self.out.len());
        let mut finish = Vec::with_capacity(self.out.len());
        let mut stack = Vec::new();
        for root in start.into_iter().chain(0..self.out.len()) {
            budget.charge(1)?;
            if seen[root] {
                continue;
            }
            seen[root] = true;
            discovery.push(Visit {
                vertex: self.vertex_name(root),
                parent_edge: None,
            });
            stack.push(Frame::new(root));
            while let Some(frame) = stack.last_mut() {
                budget.charge(1)?;
                let vertex = frame.vertex;
                let Some(arc) = self.out[vertex].get(frame.next) else {
                    finish.push(self.vertex_name(vertex));
                    stack.pop();
                    continue;
                };
                frame.next += 1;
                if !std::mem::replace(&mut seen[arc.vertex], true) {
                    discovery.push(Visit {
                        vertex: self.vertex_name(arc.vertex),
                        parent_edge: Some(self.definition.edges[arc.edge].id.clone()),
                    });
                    stack.push(Frame::new(arc.vertex));
                }
            }
        }
        Ok(ResultKind::DepthFirst { discovery, finish })
    }

    fn topological(&self, budget: &mut Budget) -> Result<ResultKind, OperationError> {
        let n = self.out.len();
        let mut degree = vec![0; n];
        for &(_, to) in &self.ends {
            budget.charge(1)?;
            degree[to] += 1;
        }
        let queue_work = 1 + u64::from(n.max(1).ilog2());
        let mut ready = BinaryHeap::new();
        for (vertex, &incoming) in degree.iter().enumerate() {
            budget.charge(1)?;
            if incoming == 0 {
                budget.charge(queue_work)?;
                ready.push(Reverse(vertex));
            }
        }
        let mut order = Vec::with_capacity(n);
        while let Some(Reverse(vertex)) = ready.pop() {
            budget.charge(queue_work)?;
            order.push(self.vertex_name(vertex));
            for arc in &self.out[vertex] {
                budget.charge(1)?;
                let incoming = &mut degree[arc.vertex];
                *incoming -= 1;
                if *incoming == 0 {
                    budget.charge(queue_work)?;
                    ready.push(Reverse(arc.vertex));
                }
            }
        }
        if order.len() == n {
            return Ok(ResultKind::Topological {
                order: Some(order),
                cycle: None,
            });
        }
        Ok(ResultKind::Topological {
            order: None,
            cycle: self.cycle_path(budget)?,
        })
    }

    fn strong_components(&self, budget: &mut Budget) -> Result<ResultKind, OperationError> {
        let n = self.out.len();
        let mut tarjan = Tarjan::new(n);
        let mut frames = Vec::new();
        for root in 0..n {
            budget.charge(1)?;
            if tarjan.index[root].is_some() {
                continue;
            }
            tarjan.visit(root);
            frames.push(Frame::new(root));
            while let Some(frame) = frames.last_mut() {
                budget.charge(1)?;
                let vertex = frame.vertex;
                if let Some(arc) = self.out[vertex].get(frame.next) {
                    frame.next += 1;
                    let next = arc.vertex;
                    if let Some(number) = tarjan.index[next] {
                        if tarjan.active[next] {
                            tarjan.low[vertex] = tarjan.low[vertex].min(number);
                        }
                    } else {
                        tarjan.visit(next);
                        frames.push(Frame::new(next));
                    }
                    continue;
                }
                frames.pop();
                if tarjan.index[vertex] == Some(tarjan.low[vertex]) {
                    tarjan.finish(vertex, budget)?;
                }
                if let Some(parent) = frames.last() {
                    tarjan.low[parent.vertex] = tarjan.low[parent.vertex].min(tarjan.low[vertex]);
                }
            }
        }
        let mut group = vec![None; tarjan.groups];
        let mut components: Vec<Vec<String>> = Vec::new();
        for (vertex, component) in tarjan.component.into_iter().enumerate() {
            budget.charge(1)?;
            let group = *group[component].get_or_insert_with(|| {
                components.push(Vec::new());
                components.len() - 1
            });
            components[group].push(self.vertex_name(vertex));
        }
        Ok(ResultKind::StrongComponents { components })
    }
}

struct Tarjan {
    serial: usize,
    groups: usize,
    index: Vec<Option<usize>>,
    low: Vec<usize>,
    active: Vec<bool>,
    stack: Vec<usize>,
    component: Vec<usize>,
}
impl Tarjan {
    fn new(n: usize) -> Self {
        Self {
            serial: 0,
            groups: 0,
            index: vec![None; n],
            low: vec![0; n],
            active: vec![false; n],
            stack: Vec::with_capacity(n),
            component: vec![0; n],
        }
    }
    fn visit(&mut self, vertex: usize) {
        self.index[vertex] = Some(self.serial);
        self.low[vertex] = self.serial;
        self.serial += 1;
        self.active[vertex] = true;
        self.stack.push(vertex);
    }
    fn finish(&mut self, root: usize, budget: &mut Budget) -> Result<(), OperationError> {
        while let Some(vertex) = self.stack.pop() {
            budget.charge(1)?;
            self.active[vertex] = false;
            self.component[vertex] = self.groups;
            if vertex == root {
                break;
            }
        }
        self.groups += 1;
        Ok(())
    }
}

/// Traverse the complete caller-owned multigraph in a deterministic order.
pub fn traverse(request: &Request) -> Result<Report, OperationError> {
    const OP: &str = "graph.traversal";
    if !matches!(request.mode, Mode::DepthFirst) && request.start.is_some() {
        return Err(parameter("start is supported only for depth_first").operation(OP));
    }
    if !matches!(request.mode, Mode::DepthFirst) && !request.graph.directed {
        return Err(
            parameter("topological and strong_components require a directed graph").operation(OP),
        );
    }
    let limits = Limits {
        max_work: request.max_work.unwrap_or(super::MAX_WORK),
        ..Limits::default()
    };
    let graph = Graph::new(request.graph.clone(), limits).map_err(|e| e.operation(OP))?;
    let mut budget = graph.budget(limits, OP)?;
    let result = match request.mode {
        Mode::DepthFirst => graph.depth_first(
            request
                .start
                .as_deref()
                .map(|id| graph.vertex(id, OP))
                .transpose()?,
            &mut budget,
        )?,
        Mode::Topological => graph.topological(&mut budget)?,
        Mode::StrongComponents => graph.strong_components(&mut budget)?,
    };
    Ok(Report {
        result,
        work: budget.work,
    })
}
