use cube_core::{
    errors::ErrorCode,
    graph::{self as g, Definition, Edge, Graph, Limits, Path},
};
use std::collections::BTreeSet;
fn definition(directed: bool, vertices: usize, edges: &[(usize, usize)]) -> Definition {
    Definition {
        directed,
        vertices: (0..vertices).map(|i| i.to_string()).collect(),
        edges: edges
            .iter()
            .enumerate()
            .map(|(i, (a, b))| Edge {
                id: format!("e{i}"),
                from: a.to_string(),
                to: b.to_string(),
            })
            .collect(),
    }
}
fn graph(directed: bool, vertices: usize, edges: &[(usize, usize)]) -> Graph {
    Graph::new(definition(directed, vertices, edges), Limits::default()).unwrap()
}
fn validate(d: &Definition, p: &Path, all: bool) {
    assert_eq!(p.vertices.len(), p.edges.len() + 1);
    let mut used = BTreeSet::new();
    for (i, id) in p.edges.iter().enumerate() {
        assert!(used.insert(id));
        let e = d.edges.iter().find(|e| &e.id == id).unwrap();
        let (a, b) = (&p.vertices[i], &p.vertices[i + 1]);
        assert!((&e.from == a && &e.to == b) || (!d.directed && &e.from == b && &e.to == a));
    }
    if all {
        assert_eq!(used.len(), d.edges.len());
    }
}
fn oracle(d: &Definition, current: &str, used: u64) -> bool {
    if used.count_ones() as usize == d.edges.len() {
        return true;
    }
    d.edges.iter().enumerate().any(|(i, e)| {
        if used & (1 << i) != 0 {
            return false;
        }
        if e.from == current {
            oracle(d, &e.to, used | (1 << i))
        } else if !d.directed && e.to == current {
            oracle(d, &e.from, used | (1 << i))
        } else {
            false
        }
    })
}
#[test]
fn tiny_directed_graphs_match_independent_floyd_and_edge_permutation_oracles() {
    let possible = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
    for mask in 0..64 {
        let edges: Vec<_> = possible
            .iter()
            .enumerate()
            .filter_map(|(i, e)| (mask & (1 << i) != 0).then_some(*e))
            .collect();
        let d = definition(true, 3, &edges);
        let graph = Graph::new(d.clone(), Limits::default()).unwrap();
        let mut distance = [[99; 3]; 3];
        let mut reach = [[false; 3]; 3];
        for (i, row) in distance.iter_mut().enumerate() {
            row[i] = 0;
        }
        for &(a, b) in &edges {
            distance[a][b] = 1;
            reach[a][b] = true;
        }
        for k in 0..3 {
            for i in 0..3 {
                for j in 0..3 {
                    distance[i][j] = distance[i][j].min(distance[i][k] + distance[k][j]);
                    reach[i][j] |= reach[i][k] && reach[k][j];
                }
            }
        }
        for (i, row) in distance.iter().enumerate() {
            for (j, &expected) in row.iter().enumerate() {
                let r = graph
                    .shortest_path(&i.to_string(), &j.to_string(), Limits::default())
                    .unwrap();
                if expected == 99 {
                    assert!(r.result.is_none());
                } else {
                    let p = r.result.unwrap();
                    validate(&d, &p, false);
                    assert_eq!(p.edges.len(), expected);
                }
            }
        }
        let cycle = graph.cycle(Limits::default()).unwrap();
        assert_eq!(cycle.result.is_some(), (0..3).any(|i| reach[i][i]));
        if let Some(p) = cycle.result {
            validate(&d, &p, false);
            assert_eq!(p.vertices.first(), p.vertices.last());
        }
        let expected = d.vertices.iter().any(|s| oracle(&d, s, 0));
        let r = graph.euler_trail(None, Limits::default()).unwrap();
        assert_eq!(r.result.exists, expected, "{edges:?}");
        if let Some(p) = r.result.path {
            validate(&d, &p, true);
        }
    }
}
#[test]
fn loops_parallel_edges_ties_isolates_and_start_rules() {
    for directed in [false, true] {
        let d = definition(directed, 2, &[(0, 0), (0, 1), (1, 0)]);
        let g = Graph::new(d.clone(), Limits::default()).unwrap();
        let c = g.cycle(Limits::default()).unwrap().result.unwrap();
        assert_eq!(c.vertices, ["0", "0"]);
        assert_eq!(c.edges, ["e0"]);
        let e = g.euler_trail(None, Limits::default()).unwrap().result;
        assert!(e.circuit);
        validate(&d, &e.path.unwrap(), true);
    }
    let d = definition(false, 2, &[(0, 1), (0, 1)]);
    let g = Graph::new(d.clone(), Limits::default()).unwrap();
    assert_eq!(
        g.cycle(Limits::default())
            .unwrap()
            .result
            .unwrap()
            .edges
            .len(),
        2
    );
    validate(
        &d,
        &g.euler_trail(Some("1"), Limits::default())
            .unwrap()
            .result
            .path
            .unwrap(),
        true,
    );
    let g = graph(true, 4, &[(0, 2), (0, 1), (2, 3), (1, 3)]);
    assert_eq!(
        g.shortest_path("0", "3", Limits::default())
            .unwrap()
            .result
            .unwrap()
            .edges,
        ["e0", "e2"]
    );
    assert!(
        g.shortest_path("3", "0", Limits::default())
            .unwrap()
            .result
            .is_none()
    );
    assert_eq!(
        g.components(Limits::default()).unwrap().result,
        vec![vec!["0", "1", "2", "3"]]
    );
    let g = graph(false, 4, &[(0, 1)]);
    assert_eq!(
        g.components(Limits::default()).unwrap().result,
        vec![vec!["0", "1"], vec!["2"], vec!["3"]]
    );
    assert!(g.euler_trail(Some("2"), Limits::default()).is_err());
    let g = graph(true, 2, &[(1, 0)]);
    assert_eq!(
        g.euler_trail(None, Limits::default())
            .unwrap()
            .result
            .path
            .unwrap()
            .vertices,
        ["1", "0"]
    );
    assert!(g.euler_trail(Some("0"), Limits::default()).is_err());
    assert!(
        !graph(false, 4, &[(0, 1), (0, 2), (0, 3)])
            .euler_trail(None, Limits::default())
            .unwrap()
            .result
            .exists
    );
    assert!(
        !graph(false, 4, &[(0, 1), (2, 3)])
            .euler_trail(None, Limits::default())
            .unwrap()
            .result
            .exists
    );
    for directed in [true, false] {
        let g = graph(directed, 0, &[]);
        assert!(g.components(Limits::default()).unwrap().result.is_empty());
        assert!(
            g.euler_trail(None, Limits::default())
                .unwrap()
                .result
                .path
                .unwrap()
                .vertices
                .is_empty()
        );
        assert!(g.euler_trail(Some("bad"), Limits::default()).is_err());
        let g = graph(directed, 3, &[]);
        assert_eq!(
            g.euler_trail(Some("2"), Limits::default())
                .unwrap()
                .result
                .path
                .unwrap()
                .vertices,
            ["2"]
        );
        assert_eq!(
            g.euler_trail(None, Limits::default())
                .unwrap()
                .result
                .path
                .unwrap()
                .vertices,
            ["0"]
        );
    }
}
#[test]
fn graph_schemas_ids_limits_and_exact_ceiling_boundaries() {
    for s in [
        r#"{"vertices":[],"vertices":[],"directed":true,"edges":[]}"#,
        r#"{"vertices":[],"directed":1,"edges":[]}"#,
        r#"{"vertices":["a"],"directed":false,"edges":[{"id":"e","from":"a","from":"a","to":"a"}]}"#,
        r#"{"vertices":[],"directed":true,"edges":[],"extra":{}}"#,
    ] {
        assert!(g::parse(s).is_err());
    }
    let mut d = definition(false, 2, &[(0, 1)]);
    d.vertices[1] = "0".into();
    assert!(Graph::new(d, Limits::default()).is_err());
    let mut d = definition(false, 2, &[(0, 1), (0, 1)]);
    d.edges[1].id = "e0".into();
    assert!(Graph::new(d, Limits::default()).is_err());
    let mut d = definition(false, 2, &[(0, 1)]);
    d.edges[0].to = "missing".into();
    assert!(Graph::new(d, Limits::default()).is_err());
    let d = definition(false, 3, &[(0, 1), (1, 2)]);
    let graph = Graph::new(d.clone(), Limits::default()).unwrap();
    let r = graph.euler_trail(None, Limits::default()).unwrap();
    let l = Limits {
        max_work: r.work,
        max_memory_bytes: r.memory_bytes_upper_bound,
        max_report_bytes: r.report_bytes_upper_bound,
    };
    assert!(graph.euler_trail(None, l).is_ok());
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    for l in [
        Limits {
            max_work: l.max_work - 1,
            ..l
        },
        Limits {
            max_memory_bytes: l.max_memory_bytes - 1,
            ..l
        },
        Limits {
            max_report_bytes: l.max_report_bytes - 1,
            ..l
        },
        Limits {
            max_work: g::MAX_WORK + 1,
            ..l
        },
        Limits {
            max_memory_bytes: usize::MAX,
            ..l
        },
        Limits {
            max_report_bytes: usize::MAX,
            ..l
        },
    ] {
        assert!(graph.euler_trail(None, l).is_err());
    }
    assert!(
        graph
            .shortest_path("unknown", "0", Limits::default())
            .is_err()
    );
    assert!(
        Graph::new(
            d,
            Limits {
                max_work: 0,
                ..Limits::default()
            }
        )
        .is_err()
    );
    let d = definition(false, g::MAX_VERTICES, &[]);
    assert_eq!(
        Graph::new(d.clone(), Limits::default())
            .unwrap()
            .components(Limits::default())
            .unwrap()
            .result
            .len(),
        g::MAX_VERTICES
    );
    let mut over = d;
    over.vertices.push("extra".into());
    assert!(Graph::new(over, Limits::default()).is_err());
    let edges = vec![(0, 0); g::MAX_EDGES];
    let d = definition(false, 1, &edges);
    let doc = serde_json::to_string(&d).unwrap();
    assert!(g::parse(&doc).is_ok());
    let graph = Graph::new(d, Limits::default()).unwrap();
    let r = graph.euler_trail(None, Limits::default()).unwrap();
    assert_eq!(r.result.path.as_ref().unwrap().edges.len(), g::MAX_EDGES);
    assert!(serde_json::to_vec(&r).unwrap().len() <= r.report_bytes_upper_bound);
    let over = definition(false, 1, &vec![(0, 0); g::MAX_EDGES + 1]);
    assert!(Graph::new(over.clone(), Limits::default()).is_err());
    assert!(g::parse(&serde_json::to_string(&over).unwrap()).is_err());
    assert_eq!(
        g::parse(&" ".repeat(g::MAX_INPUT + 1)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
