//! Explicit bounded multigraphs. Ordered vertex/edge IDs resolve traversal ties;
//! undirected loops have degree two and retain one edge identity. No weights or
//! implicit vertices. BFS and Hierholzer are shared concrete algorithms.
mod metadata;
mod queries;
use crate::{
    errors::{ErrorCode, OperationError},
    pipeline::json,
};
pub use metadata::OPERATIONS;
pub use queries::{EulerResult, Path, QueryReport};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_VERTICES: usize = 4096;
pub const MAX_EDGES: usize = 32768;
pub const MAX_INPUT: usize = 2 * 1024 * 1024;
pub const MAX_MEMORY: usize = 32 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
pub const MAX_WORK: u64 = 4 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    #[serde(deserialize_with = "id")]
    pub id: String,
    #[serde(deserialize_with = "id")]
    pub from: String,
    #[serde(deserialize_with = "id")]
    pub to: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "vertex_ids")]
    pub vertices: Vec<String>,
    pub directed: bool,
    #[serde(deserialize_with = "edge_list")]
    pub edges: Vec<Edge>,
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Limits {
    pub max_work: u64,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Arc {
    vertex: usize,
    edge: usize,
}
#[derive(Debug)]
pub struct Graph {
    definition: Definition,
    indices: BTreeMap<String, usize>,
    ends: Vec<(usize, usize)>,
    out: Vec<Vec<Arc>>,
    weak: Vec<Vec<usize>>,
    memory: usize,
    report: usize,
    base_work: u64,
}
fn error(op: &'static str, code: ErrorCode, name: &'static str, msg: &str) -> OperationError {
    OperationError::new(code, msg).operation(op).parameter(name)
}
fn invalid(op: &'static str, name: &'static str, msg: &str) -> OperationError {
    error(op, ErrorCode::InvalidInput, name, msg)
}
struct Budget {
    op: &'static str,
    work: u64,
    max: u64,
}
impl Budget {
    fn charge(&mut self, n: u64) -> Result<(), OperationError> {
        self.work = self
            .work
            .checked_add(n)
            .filter(|w| *w <= self.max)
            .ok_or_else(|| {
                error(
                    self.op,
                    ErrorCode::ResourceLimit,
                    "max_work",
                    "graph visit budget exhausted; no impossibility claim produced",
                )
            })?;
        Ok(())
    }
}
fn id<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    use serde::de::Error;
    let value = String::deserialize(d)?;
    if value.is_empty() || value.len() > 64 {
        return Err(D::Error::custom("graph IDs require 1..64 UTF-8 bytes"));
    }
    Ok(value)
}
struct Id(String);
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        id(d).map(Self)
    }
}
fn bounded_list<'de, D, T, const N: usize>(d: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct List<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for List<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded graph array")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> Result<Self::Value, A::Error> {
            use serde::de::Error;
            let mut v = Vec::new();
            while let Some(item) = a.next_element()? {
                if v.len() == N {
                    return Err(A::Error::custom("graph JSON budget exceeded"));
                }
                v.push(item);
            }
            Ok(v)
        }
    }
    d.deserialize_seq(List::<T, N>(std::marker::PhantomData))
}
fn vertex_ids<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    bounded_list::<D, Id, MAX_VERTICES>(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
fn edge_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Edge>, D::Error> {
    bounded_list::<D, Edge, MAX_EDGES>(d)
}
/// Typed bounded JSON decoding avoids a second untyped object tree. Derive rejects
/// duplicate/unknown fields at every object; array visitors cap before append and
/// IDs are checked before retention. Accepted schema depth is fixed (four levels).
pub fn parse(document: &str) -> Result<Definition, OperationError> {
    const OP: &str = "graph.validate";
    if document.len() > MAX_INPUT {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "document",
            "graph JSON exceeds 2 MiB",
        ));
    }
    serde_json::from_str(document).map_err(|e| {
        let code = if e.to_string().starts_with("graph JSON budget exceeded") {
            ErrorCode::ResourceLimit
        } else {
            ErrorCode::InvalidInput
        };
        error(
            OP,
            code,
            "graph",
            "invalid graph JSON: check fields, types, unique keys, ID lengths and array ceilings",
        )
        .caused_by(e)
    })
}
impl Graph {
    /// Index a caller-owned definition once; source/IDs are not inferred. Preflight
    /// covers both adjacency lists, lookup trees, traversal state and report IDs.
    pub fn new(definition: Definition, limits: Limits) -> Result<Self, OperationError> {
        const OP: &str = "graph.validate";
        Self::validate_limits(limits, OP)?;
        let v = definition.vertices.len();
        let e = definition.edges.len();
        if v > MAX_VERTICES || e > MAX_EDGES {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "graph",
                "graph exceeds 4096 vertices or 32768 edges",
            ));
        }
        let valid_id = |s: &str| !s.is_empty() && s.len() <= 64;
        if definition.vertices.iter().any(|s| !valid_id(s))
            || definition
                .edges
                .iter()
                .any(|e| !valid_id(&e.id) || !valid_id(&e.from) || !valid_id(&e.to))
        {
            return Err(invalid(
                OP,
                "graph",
                "all IDs must be nonempty UTF-8 strings <=64 bytes",
            ));
        }
        let vertex_bytes: usize = definition
            .vertices
            .iter()
            .map(|s| json::string_bytes(s) + 1)
            .sum();
        let edge_bytes: usize = definition
            .edges
            .iter()
            .map(|e| {
                22 + json::string_bytes(&e.id)
                    + json::string_bytes(&e.from)
                    + json::string_bytes(&e.to)
            })
            .sum();
        let input = 128 + vertex_bytes + edge_bytes;
        if input > MAX_INPUT {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "graph",
                "conservative compact graph transport bound exceeds 2 MiB",
            ));
        }
        let max_vertex = definition
            .vertices
            .iter()
            .map(|s| json::string_bytes(s) + 1)
            .max()
            .unwrap_or(0);
        let report = 4096
            + vertex_bytes
            + v * 8
            + (e + 1) * max_vertex
            + definition
                .edges
                .iter()
                .map(|e| json::string_bytes(&e.id) + 1)
                .sum::<usize>();
        let memory = 8192 + input * 2 + v * 512 + e * 384 + report * 2;
        if report > limits.max_report_bytes {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "max_report_bytes",
                "conservative complete graph report exceeds budget",
            ));
        }
        if memory > limits.max_memory_bytes {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "max_memory_bytes",
                "conservative graph allocation bound exceeds budget",
            ));
        }
        let base_work = v as u64 + 4 * e as u64;
        if base_work > limits.max_work {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "max_work",
                "graph indexing exceeds work budget",
            ));
        }
        let mut indices = BTreeMap::new();
        for (i, s) in definition.vertices.iter().enumerate() {
            if indices.insert(s.clone(), i).is_some() {
                return Err(invalid(OP, "vertices", "duplicate vertex ID"));
            }
        }
        let mut edge_ids = BTreeSet::new();
        let mut ends = Vec::with_capacity(e);
        let mut out = vec![Vec::new(); v];
        let mut weak = vec![Vec::new(); v];
        for (i, edge) in definition.edges.iter().enumerate() {
            if !edge_ids.insert(edge.id.as_str()) {
                return Err(invalid(OP, "edges", "duplicate edge ID"));
            }
            let (Some(&a), Some(&b)) = (indices.get(&edge.from), indices.get(&edge.to)) else {
                return Err(invalid(OP, "edges", "edge endpoint is undeclared"));
            };
            ends.push((a, b));
            out[a].push(Arc { vertex: b, edge: i });
            if !definition.directed {
                out[b].push(Arc { vertex: a, edge: i });
            }
            weak[a].push(b);
            weak[b].push(a);
        }
        Ok(Self {
            definition,
            indices,
            ends,
            out,
            weak,
            memory,
            report,
            base_work,
        })
    }
    fn validate_limits(limits: Limits, op: &'static str) -> Result<(), OperationError> {
        if limits.max_work > MAX_WORK
            || limits.max_memory_bytes > MAX_MEMORY
            || limits.max_report_bytes > MAX_REPORT
        {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "limits",
                "graph query limits exceed hard ceilings",
            ));
        }
        Ok(())
    }
    fn budget(&self, limits: Limits, op: &'static str) -> Result<Budget, OperationError> {
        Self::validate_limits(limits, op)?;
        if self.memory > limits.max_memory_bytes || self.report > limits.max_report_bytes {
            return Err(error(
                op,
                ErrorCode::ResourceLimit,
                "limits",
                "graph query allocation/report bound exceeds budget",
            ));
        }
        let mut b = Budget {
            op,
            work: 0,
            max: limits.max_work,
        };
        b.charge(self.base_work)?;
        Ok(b)
    }
    fn vertex(&self, id: &str, op: &'static str) -> Result<usize, OperationError> {
        self.indices.get(id).copied().ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidParameter,
                "vertex",
                "requested vertex does not exist",
            )
        })
    }
    fn path(&self, vertices: Vec<usize>, edges: Vec<usize>) -> Path {
        Path {
            vertices: vertices
                .into_iter()
                .map(|i| self.definition.vertices[i].clone())
                .collect(),
            edges: edges
                .into_iter()
                .map(|i| self.definition.edges[i].id.clone())
                .collect(),
        }
    }
    fn finish<T>(&self, result: T, b: Budget) -> QueryReport<T> {
        QueryReport {
            result,
            work: b.work,
            memory_bytes_upper_bound: self.memory,
            report_bytes_upper_bound: self.report,
        }
    }
}
