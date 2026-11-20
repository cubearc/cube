//! Bounded exact shortest paths over ordered multigraph edge identities.
use super::{Definition, Edge, Graph, Limits, MAX_EDGES, MAX_VERTICES, Path};
use crate::{
    errors::OperationError,
    formal_work::{Stop, Work, invalid, parameter, resource},
    numbers::analysis::Signed,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

const MAX_WORK: usize = 16 << 20;
const MAX_MATRIX: usize = 256;
type Parent = Option<(usize, usize)>;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WeightedEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub weight: i64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WeightedDefinition {
    pub vertices: Vec<String>,
    pub directed: bool,
    pub edges: Vec<WeightedEdge>,
}

struct Weighted {
    graph: Graph,
    weights: Vec<i64>,
}

impl Weighted {
    fn new(input: &WeightedDefinition, vertex_limit: usize) -> Result<Self, OperationError> {
        if input.vertices.len() > vertex_limit || input.edges.len() > MAX_EDGES {
            return Err(resource(
                "weighted graph exceeds its vertex or edge ceiling",
            ));
        }
        let definition = Definition {
            vertices: input.vertices.clone(),
            directed: input.directed,
            edges: input
                .edges
                .iter()
                .map(|e| Edge {
                    id: e.id.clone(),
                    from: e.from.clone(),
                    to: e.to.clone(),
                })
                .collect(),
        };
        Ok(Self {
            graph: Graph::new(definition, Limits::default())?,
            weights: input.edges.iter().map(|e| e.weight).collect(),
        })
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Algorithm {
    Dijkstra,
    BellmanFord,
    AStar { heuristic: Vec<u64> },
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub graph: WeightedDefinition,
    pub start: String,
    pub goal: String,
    pub algorithm: Algorithm,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PathResult {
    Finite { cost: Signed, path: Path },
    Unreachable,
    Unbounded,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub result: PathResult,
    pub negative_cycle: Option<Path>,
    pub work: usize,
}

fn charge(work: &mut Work<'_>, units: usize) -> Result<(), OperationError> {
    work.charge(units).map_err(Stop::error)
}

fn distance(a: i128, b: i128) -> Result<i128, OperationError> {
    a.checked_add(b)
        .ok_or_else(|| resource("weighted distance exceeds signed 128-bit arithmetic"))
}

fn path(
    graph: &Graph,
    parent: &[Parent],
    start: usize,
    goal: usize,
    work: &mut Work<'_>,
) -> Result<Path, OperationError> {
    let mut vertices = vec![goal];
    let mut edges = Vec::new();
    let mut current = goal;
    while current != start {
        charge(work, 1)?;
        if edges.len() >= graph.out.len() {
            return Err(invalid(
                "path predecessors do not lead to the declared start",
            ));
        }
        let (previous, edge) =
            parent[current].ok_or_else(|| invalid("finite path has no predecessor"))?;
        vertices.push(previous);
        edges.push(edge);
        current = previous;
    }
    vertices.reverse();
    edges.reverse();
    Ok(graph.path(vertices, edges))
}

fn heuristic<'a>(
    graph: &Weighted,
    algorithm: &'a Algorithm,
    goal: usize,
    work: &mut Work<'_>,
) -> Result<Option<&'a [u64]>, OperationError> {
    charge(work, graph.weights.len())?;
    if graph.weights.iter().any(|&w| w < 0) {
        return Err(invalid("Dijkstra and A* require nonnegative edge weights"));
    }
    let Algorithm::AStar { heuristic } = algorithm else {
        return Ok(None);
    };
    if heuristic.len() != graph.graph.out.len() || heuristic[goal] != 0 {
        return Err(parameter(
            "A* requires one heuristic per vertex with zero at the goal",
        ));
    }
    for (from, out) in graph.graph.out.iter().enumerate() {
        for arc in out {
            charge(work, 1)?;
            if i128::from(heuristic[from])
                > i128::from(graph.weights[arc.edge]) + i128::from(heuristic[arc.vertex])
            {
                return Err(invalid(
                    "A* heuristic is inconsistent for a directed traversal edge",
                ));
            }
        }
    }
    Ok(Some(heuristic))
}

fn heap_path(
    graph: &Weighted,
    start: usize,
    goal: usize,
    heuristic: Option<&[u64]>,
    work: &mut Work<'_>,
) -> Result<PathResult, OperationError> {
    let estimate = |v| heuristic.map_or(0, |h| i128::from(h[v]));
    let mut heap = BinaryHeap::from([Reverse((estimate(start), start, 0))]);
    let mut distances = vec![None; graph.graph.out.len()];
    let mut parent = vec![None; graph.graph.out.len()];
    distances[start] = Some(0);
    while !heap.is_empty() {
        charge(work, 1 + heap.len().ilog2() as usize)?;
        let Reverse((_, from, cost)) = heap.pop().expect("nonempty queue");
        if distances[from] != Some(cost) {
            continue;
        }
        if from == goal {
            return Ok(PathResult::Finite {
                cost: Signed(cost),
                path: path(&graph.graph, &parent, start, goal, work)?,
            });
        }
        for arc in &graph.graph.out[from] {
            charge(work, 1)?;
            let next = distance(cost, i128::from(graph.weights[arc.edge]))?;
            if distances[arc.vertex].is_none_or(|old| next < old) {
                distances[arc.vertex] = Some(next);
                parent[arc.vertex] = Some((from, arc.edge));
                charge(work, 1 + (heap.len() + 1).ilog2() as usize)?;
                heap.push(Reverse((
                    distance(next, estimate(arc.vertex))?,
                    arc.vertex,
                    next,
                )));
            }
        }
    }
    Ok(PathResult::Unreachable)
}

struct Bellman {
    distances: Vec<Option<i128>>,
    parent: Vec<Parent>,
}

impl Bellman {
    fn sweep(
        &mut self,
        graph: &Weighted,
        mut updated: impl FnMut(usize),
        work: &mut Work<'_>,
    ) -> Result<bool, OperationError> {
        let mut changed = false;
        for (edge, &(a, b)) in graph.graph.ends.iter().enumerate() {
            if self.relax(a, b, edge, graph.weights[edge], work)? {
                updated(b);
                changed = true;
            }
            if !graph.graph.definition.directed
                && self.relax(b, a, edge, graph.weights[edge], work)?
            {
                updated(a);
                changed = true;
            }
        }
        Ok(changed)
    }

    fn relax(
        &mut self,
        a: usize,
        b: usize,
        edge: usize,
        weight: i64,
        work: &mut Work<'_>,
    ) -> Result<bool, OperationError> {
        charge(work, 1)?;
        let Some(cost) = self.distances[a] else {
            return Ok(false);
        };
        let next = distance(cost, i128::from(weight))?;
        if self.distances[b].is_some_and(|old| next >= old) {
            return Ok(false);
        }
        self.distances[b] = Some(next);
        self.parent[b] = Some((a, edge));
        Ok(true)
    }
}

fn cycle(
    graph: &Graph,
    parent: &[Parent],
    mut vertex: usize,
    work: &mut Work<'_>,
) -> Result<Path, OperationError> {
    let parent_at =
        |v: usize| parent[v].ok_or_else(|| invalid("reachable negative cycle has no predecessor"));
    for _ in 0..graph.out.len() {
        charge(work, 1)?;
        vertex = parent_at(vertex)?.0;
    }
    let start = vertex;
    let mut vertices = vec![start];
    let mut edges = Vec::new();
    loop {
        charge(work, 1)?;
        let (previous, edge) = parent_at(vertex)?;
        edges.push(edge);
        vertices.push(previous);
        if previous == start {
            break;
        }
        vertex = previous;
    }
    vertices.reverse();
    edges.reverse();
    Ok(graph.path(vertices, edges))
}

fn bellman_path(
    graph: &Weighted,
    start: usize,
    goal: usize,
    work: &mut Work<'_>,
) -> Result<(PathResult, Option<Path>), OperationError> {
    let n = graph.graph.out.len();
    let mut state = Bellman {
        distances: vec![None; n],
        parent: vec![None; n],
    };
    state.distances[start] = Some(0);
    let mut affected = vec![false; n];
    let mut last = None;
    for pass in 0..n {
        charge(work, 1)?;
        if !state.sweep(
            graph,
            |current| {
                if pass + 1 == n {
                    affected[current] = true;
                    last = Some(current);
                }
            },
            work,
        )? {
            break;
        }
    }
    let witness = last
        .map(|v| cycle(&graph.graph, &state.parent, v, work))
        .transpose()?;
    if witness.is_some() {
        charge(work, n)?;
        let mut queue: VecDeque<_> = affected
            .iter()
            .enumerate()
            .filter_map(|(v, &flag)| flag.then_some(v))
            .collect();
        while let Some(from) = queue.pop_front() {
            charge(work, 1)?;
            for arc in &graph.graph.out[from] {
                charge(work, 1)?;
                if !std::mem::replace(&mut affected[arc.vertex], true) {
                    queue.push_back(arc.vertex);
                }
            }
        }
    }
    let result = match (affected[goal], state.distances[goal]) {
        (true, _) => PathResult::Unbounded,
        (false, None) => PathResult::Unreachable,
        (false, Some(cost)) => PathResult::Finite {
            cost: Signed(cost),
            path: path(&graph.graph, &state.parent, start, goal, work)?,
        },
    };
    Ok((result, witness))
}

/// Find one exact weighted path with the chosen explicit algorithm.
pub fn shortest(request: &Request) -> Result<Report, OperationError> {
    shortest_request(request).map_err(|e| e.operation("graph.weighted_path"))
}

fn shortest_request(request: &Request) -> Result<Report, OperationError> {
    const OP: &str = "graph.weighted_path";
    let mut cancelled = || false;
    let mut work = Work::new(request.max_work, MAX_WORK, &mut cancelled)?;
    charge(
        &mut work,
        request.graph.vertices.len() + request.graph.edges.len(),
    )?;
    let graph = Weighted::new(&request.graph, MAX_VERTICES)?;
    let start = graph.graph.vertex(&request.start, OP)?;
    let goal = graph.graph.vertex(&request.goal, OP)?;
    let (result, negative_cycle) = if matches!(request.algorithm, Algorithm::BellmanFord) {
        bellman_path(&graph, start, goal, &mut work)?
    } else {
        let h = heuristic(&graph, &request.algorithm, goal, &mut work)?;
        (heap_path(&graph, start, goal, h, &mut work)?, None)
    };
    Ok(Report {
        result,
        negative_cycle,
        work: work.used(),
    })
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PathQuery {
    pub start: String,
    pub goal: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllPairsRequest {
    pub graph: WeightedDefinition,
    #[serde(default)]
    #[schemars(length(max = 256))]
    pub paths: Vec<PathQuery>,
    pub max_work: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct AllPairsReport {
    pub vertices: Vec<String>,
    pub distances: Vec<Vec<Option<Signed>>>,
    pub unbounded: Vec<Vec<bool>>,
    pub paths: Vec<PathResult>,
    pub work: usize,
}

type Reach = [u64; MAX_MATRIX / 64];

fn negative_pairs(distances: &[Vec<Option<i128>>]) -> Vec<Reach> {
    let mut reachable = vec![[0; MAX_MATRIX / 64]; distances.len()];
    for (i, row) in distances.iter().enumerate() {
        for (j, item) in row.iter().enumerate() {
            if item.is_some() {
                reachable[i][j / 64] |= 1 << (j % 64);
            }
        }
    }
    let negative: Vec<_> = distances
        .iter()
        .enumerate()
        .filter_map(|(i, row)| row[i].is_some_and(|value| value < 0).then_some(i))
        .collect();
    let mut unbounded = vec![[0; MAX_MATRIX / 64]; distances.len()];
    for (i, &row) in reachable.iter().enumerate() {
        for &via in &negative {
            if bit(row, via) {
                for (result, bits) in unbounded[i].iter_mut().zip(reachable[via]) {
                    *result |= bits;
                }
            }
        }
    }
    unbounded
}

fn bit(row: Reach, index: usize) -> bool {
    row[index / 64] & (1 << (index % 64)) != 0
}

struct Floyd {
    distances: Vec<Vec<Option<i128>>>,
    parent: Vec<Vec<Parent>>,
}

impl Floyd {
    fn new(graph: &Weighted) -> Self {
        let n = graph.graph.out.len();
        let mut distances = vec![vec![None; n]; n];
        let mut parent = vec![vec![None; n]; n];
        for (from, arcs) in graph.graph.out.iter().enumerate() {
            distances[from][from] = Some(0);
            for arc in arcs {
                let weight = i128::from(graph.weights[arc.edge]);
                if distances[from][arc.vertex].is_none_or(|old| weight < old) {
                    distances[from][arc.vertex] = Some(weight);
                    parent[from][arc.vertex] = Some((from, arc.edge));
                }
            }
        }
        Self { distances, parent }
    }

    fn relax_paths(&mut self, work: &mut Work<'_>) -> Result<(), OperationError> {
        let n = self.distances.len();
        // Below the minimum weight of any simple walk of n edges, every such
        // cost is already associated with a negative cycle. A floor preserves
        // reachability/negative diagonals without inflating repeated cycles.
        let floor = -(n as i128 + 1) * (1_i128 << 63);
        for via in 0..n {
            for from in 0..n {
                let Some(first) = self.distances[from][via] else {
                    continue;
                };
                for to in 0..n {
                    charge(work, 1)?;
                    let Some(second) = self.distances[via][to] else {
                        continue;
                    };
                    let next = distance(first, second)?.max(floor);
                    if self.distances[from][to].is_none_or(|old| next < old) {
                        self.distances[from][to] = Some(next);
                        self.parent[from][to] = self.parent[via][to];
                    }
                }
            }
        }
        Ok(())
    }
}

/// Compute all vertex-order distances and mark pairs affected by negative cycles.
pub fn all_pairs(request: &AllPairsRequest) -> Result<AllPairsReport, OperationError> {
    all_pairs_request(request).map_err(|e| e.operation("graph.all_pairs_paths"))
}

fn all_pairs_request(request: &AllPairsRequest) -> Result<AllPairsReport, OperationError> {
    const OP: &str = "graph.all_pairs_paths";
    if request.paths.len() > MAX_MATRIX {
        return Err(resource("all-pairs request exceeds 256 path queries"));
    }
    let graph = Weighted::new(&request.graph, MAX_MATRIX)?;
    let endpoints = request
        .paths
        .iter()
        .map(|query| {
            Ok((
                graph.graph.vertex(&query.start, OP)?,
                graph.graph.vertex(&query.goal, OP)?,
            ))
        })
        .collect::<Result<Vec<_>, OperationError>>()?;
    let mut cancelled = || false;
    let mut work = Work::new(request.max_work, MAX_WORK, &mut cancelled)?;
    let mut state = Floyd::new(&graph);
    state.relax_paths(&mut work)?;
    let flags = negative_pairs(&state.distances);
    // Path reconstruction is bounded separately by n * 256 and cannot take
    // relaxation work away from a full 256 by 256 matrix.
    let mut path_cancelled = || false;
    let mut path_work = Work::new(None, MAX_MATRIX * MAX_MATRIX, &mut path_cancelled)?;
    let paths = endpoints
        .into_iter()
        .map(|(from, to)| {
            Ok(match (bit(flags[from], to), state.distances[from][to]) {
                (true, _) => PathResult::Unbounded,
                (false, None) => PathResult::Unreachable,
                (false, Some(cost)) => PathResult::Finite {
                    cost: Signed(cost),
                    path: path(&graph.graph, &state.parent[from], from, to, &mut path_work)?,
                },
            })
        })
        .collect::<Result<Vec<_>, OperationError>>()?;
    let unbounded: Vec<Vec<bool>> = flags
        .iter()
        .map(|row| (0..flags.len()).map(|j| bit(*row, j)).collect())
        .collect();
    let distances = state
        .distances
        .into_iter()
        .zip(&unbounded)
        .map(|(row, mask)| {
            row.into_iter()
                .zip(mask)
                .map(|(value, &negative)| if negative { None } else { value.map(Signed) })
                .collect()
        })
        .collect();
    Ok(AllPairsReport {
        vertices: graph.graph.definition.vertices,
        distances,
        unbounded,
        paths,
        work: work.used(),
    })
}
