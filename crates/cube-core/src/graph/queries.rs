//! Deterministic weak connectivity, BFS, cycle witnesses and Hierholzer trails.
use super::*;
use std::collections::VecDeque;
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Path {
    pub vertices: Vec<String>,
    pub edges: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct QueryReport<T> {
    pub result: T,
    pub work: u64,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
#[derive(Debug, Serialize)]
pub struct EulerResult {
    pub exists: bool,
    pub circuit: bool,
    pub path: Option<Path>,
    pub reason: Option<&'static str>,
}
impl Graph {
    /// Weak components ignore edge direction, preserve vertex input order within
    /// each component, and include isolated vertices. Empty graph yields [].
    pub fn components(
        &self,
        limits: Limits,
    ) -> Result<QueryReport<Vec<Vec<String>>>, OperationError> {
        let mut b = self.budget(limits, "graph.components")?;
        let groups = self.component_indices(&mut b)?;
        Ok(self.finish(
            groups
                .into_iter()
                .map(|g| {
                    g.into_iter()
                        .map(|i| self.definition.vertices[i].clone())
                        .collect()
                })
                .collect(),
            b,
        ))
    }
    fn component_indices(&self, b: &mut Budget) -> Result<Vec<Vec<usize>>, OperationError> {
        let mut seen = vec![false; self.out.len()];
        let mut groups = Vec::new();
        for root in 0..self.out.len() {
            b.charge(1)?;
            if seen[root] {
                continue;
            }
            let mut queue = VecDeque::from([root]);
            seen[root] = true;
            let mut group = Vec::new();
            while let Some(v) = queue.pop_front() {
                b.charge(1)?;
                group.push(v);
                for &next in &self.weak[v] {
                    b.charge(1)?;
                    if !seen[next] {
                        seen[next] = true;
                        queue.push_back(next);
                    }
                }
            }
            b.charge(2 * group.len() as u64 * u64::from(group.len().ilog2() + 1))?;
            group.sort_unstable();
            groups.push(group);
        }
        Ok(groups)
    }
    /// One shortest unweighted path. BFS explores adjacency in edge input order;
    /// equal-length routes retain the first discovery. None means proven no route.
    pub fn shortest_path(
        &self,
        source: &str,
        target: &str,
        limits: Limits,
    ) -> Result<QueryReport<Option<Path>>, OperationError> {
        const OP: &str = "graph.shortest_path";
        let mut b = self.budget(limits, OP)?;
        let start = self.vertex(source, OP)?;
        let goal = self.vertex(target, OP)?;
        let mut parents = vec![None; self.out.len()];
        let mut seen = vec![false; self.out.len()];
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(v) = queue.pop_front() {
            b.charge(1)?;
            if v == goal {
                let mut vertices = vec![v];
                let mut edges = Vec::new();
                let mut p = v;
                while p != start {
                    b.charge(1)?;
                    let (prev, edge) = parents[p].expect("discovered vertex has a parent");
                    vertices.push(prev);
                    edges.push(edge);
                    p = prev;
                }
                vertices.reverse();
                edges.reverse();
                return Ok(self.finish(Some(self.path(vertices, edges)), b));
            }
            for arc in &self.out[v] {
                b.charge(1)?;
                if !seen[arc.vertex] {
                    seen[arc.vertex] = true;
                    parents[arc.vertex] = Some((v, arc.edge));
                    queue.push_back(arc.vertex);
                }
            }
        }
        Ok(self.finish(None, b))
    }
    /// DFS is used only to construct a cycle witness, not exposed as a new graph
    /// traversal API. Parent *edge* identity preserves parallel-edge cycles.
    pub fn cycle(&self, limits: Limits) -> Result<QueryReport<Option<Path>>, OperationError> {
        let mut b = self.budget(limits, "graph.cycle")?;
        let n = self.out.len();
        let mut color = vec![0u8; n];
        let mut parent = vec![None; n];
        let mut parent_edge = vec![None; n];
        for root in 0..n {
            b.charge(1)?;
            if color[root] != 0 {
                continue;
            }
            color[root] = 1;
            let mut stack = vec![(root, 0usize)];
            while let Some((v, index)) = stack.last_mut() {
                b.charge(1)?;
                if *index == self.out[*v].len() {
                    color[*v] = 2;
                    stack.pop();
                    continue;
                }
                let arc = self.out[*v][*index];
                *index += 1;
                if !self.definition.directed && parent_edge[*v] == Some(arc.edge) {
                    continue;
                }
                if color[arc.vertex] == 0 {
                    parent[arc.vertex] = Some(*v);
                    parent_edge[arc.vertex] = Some(arc.edge);
                    color[arc.vertex] = 1;
                    stack.push((arc.vertex, 0));
                } else if color[arc.vertex] == 1 {
                    let mut vertices = vec![*v];
                    let mut edges = Vec::new();
                    let mut p = *v;
                    while p != arc.vertex {
                        b.charge(1)?;
                        edges.push(parent_edge[p].expect("ancestor path has an edge"));
                        p = parent[p].expect("active ancestor is reachable");
                        vertices.push(p);
                    }
                    vertices.reverse();
                    edges.reverse();
                    vertices.push(arc.vertex);
                    edges.push(arc.edge);
                    return Ok(self.finish(Some(self.path(vertices, edges)), b));
                }
            }
        }
        Ok(self.finish(None, b))
    }
    /// Hierholzer over exact edge identities. Weak connectivity of non-isolates
    /// and degree balance determine existence; isolated vertices are ignored when
    /// edges exist. Requested starts must satisfy the corresponding degree rule.
    pub fn euler_trail(
        &self,
        start: Option<&str>,
        limits: Limits,
    ) -> Result<QueryReport<EulerResult>, OperationError> {
        const OP: &str = "graph.euler_trail";
        let mut b = self.budget(limits, OP)?;
        let requested = start.map(|s| self.vertex(s, OP)).transpose()?;
        if self.ends.is_empty() {
            let v = requested.or_else(|| (!self.out.is_empty()).then_some(0));
            let path = self.path(v.into_iter().collect(), Vec::new());
            return Ok(self.finish(
                EulerResult {
                    exists: true,
                    circuit: true,
                    path: Some(path),
                    reason: None,
                },
                b,
            ));
        }
        let n = self.out.len();
        let mut incoming = vec![0usize; n];
        let mut outgoing = vec![0usize; n];
        for &(a, c) in &self.ends {
            b.charge(1)?;
            outgoing[a] += 1;
            incoming[c] += 1;
        }
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        let mut odd = Vec::new();
        for v in 0..n {
            b.charge(1)?;
            if self.definition.directed {
                if outgoing[v] == incoming[v] + 1 {
                    positive.push(v);
                } else if incoming[v] == outgoing[v] + 1 {
                    negative.push(v);
                } else if outgoing[v] != incoming[v] {
                    return Ok(self.no_euler("directed degree imbalance", b));
                }
            } else if (incoming[v] + outgoing[v]) % 2 == 1 {
                odd.push(v);
            }
        }
        if self.definition.directed
            && !((positive.is_empty() && negative.is_empty())
                || (positive.len() == 1 && negative.len() == 1))
        {
            return Ok(self.no_euler("directed degree imbalance", b));
        }
        if !self.definition.directed && !odd.is_empty() && odd.len() != 2 {
            return Ok(self.no_euler("undirected graph has neither zero nor two odd vertices", b));
        }
        let components = self.component_indices(&mut b)?;
        if components
            .iter()
            .filter(|g| g.iter().any(|&v| incoming[v] + outgoing[v] > 0))
            .count()
            != 1
        {
            return Ok(self.no_euler("non-isolated vertices are disconnected", b));
        }
        let required = if self.definition.directed {
            positive.first().copied()
        } else {
            odd.first().copied()
        };
        let default = required.unwrap_or_else(|| {
            (0..n)
                .find(|&v| incoming[v] + outgoing[v] > 0)
                .expect("edges imply non-isolated vertex")
        });
        let root = requested.unwrap_or(default);
        let allowed = if self.definition.directed {
            required.map_or(outgoing[root] > 0, |v| v == root)
        } else {
            if odd.len() == 2 {
                odd.contains(&root)
            } else {
                incoming[root] + outgoing[root] > 0
            }
        };
        if !allowed {
            return Err(error(
                OP,
                ErrorCode::InvalidParameter,
                "start",
                "requested start does not satisfy Euler trail conditions",
            ));
        }
        let mut used = vec![false; self.ends.len()];
        let mut cursor = vec![0; n];
        let mut vertices = vec![root];
        let mut edges = Vec::new();
        let mut reverse_vertices = Vec::new();
        let mut reverse_edges = Vec::new();
        while let Some(&v) = vertices.last() {
            b.charge(1)?;
            while cursor[v] < self.out[v].len() && used[self.out[v][cursor[v]].edge] {
                b.charge(1)?;
                cursor[v] += 1;
            }
            if cursor[v] < self.out[v].len() {
                let arc = self.out[v][cursor[v]];
                cursor[v] += 1;
                used[arc.edge] = true;
                vertices.push(arc.vertex);
                edges.push(arc.edge);
            } else {
                reverse_vertices.push(vertices.pop().expect("nonempty traversal stack"));
                if let Some(e) = edges.pop() {
                    reverse_edges.push(e);
                }
            }
        }
        debug_assert_eq!(reverse_edges.len(), self.ends.len());
        reverse_vertices.reverse();
        reverse_edges.reverse();
        let circuit = reverse_vertices.first() == reverse_vertices.last();
        Ok(self.finish(
            EulerResult {
                exists: true,
                circuit,
                path: Some(self.path(reverse_vertices, reverse_edges)),
                reason: None,
            },
            b,
        ))
    }
    fn no_euler(&self, reason: &'static str, b: Budget) -> QueryReport<EulerResult> {
        self.finish(
            EulerResult {
                exists: false,
                circuit: false,
                path: None,
                reason: Some(reason),
            },
            b,
        )
    }
}
