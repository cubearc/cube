use cube_core::{
    errors::ErrorCode,
    operations::{contracts, formal},
};
use serde_json::{Value, json};

fn run(id: &str, request: Value) -> Value {
    formal::invoke_with_cancel(id, request, &mut || false).unwrap()
}

#[test]
fn declared_formal_fixtures_execute_through_named_ports() {
    let registered = contracts::registered_fixtures().unwrap();
    for operation in formal::OPERATIONS {
        let (_, fixture) = registered
            .iter()
            .find(|(id, _)| id == operation.id)
            .unwrap();
        assert_eq!(
            formal::invoke_transport(operation.id, fixture.inputs["request"].clone()).unwrap(),
            fixture.outputs["report"],
            "{}",
            operation.id
        );
    }
}

#[test]
fn ordered_traversal_and_cycle_evidence() {
    let graph = json!({"vertices":["b","a","c","d"],"directed":true,"edges":[
        {"id":"a-c","from":"a","to":"c"},{"id":"a-c2","from":"a","to":"c"},
        {"id":"c-b","from":"c","to":"b"},{"id":"b-c","from":"b","to":"c"}]});
    let traverse = |mode| run("graph.traversal", json!({"graph":graph,"mode":mode}));
    let dfs = run(
        "graph.traversal",
        json!({"graph":graph,"mode":"depth_first","start":"a"}),
    );
    assert_eq!(dfs["result"]["finish"], json!(["b", "c", "a", "d"]));
    assert_eq!(dfs["result"]["discovery"][1]["parent_edge"], "a-c");
    assert_eq!(
        traverse("strong_components")["result"]["components"],
        json!([["b", "c"], ["a"], ["d"]])
    );
    let topological = traverse("topological");
    assert!(topological["result"]["order"].is_null());
    assert_eq!(
        topological["result"]["cycle"]["edges"],
        json!(["b-c", "c-b"])
    );
    let count = 4096;
    let vertices: Vec<_> = (0..count).map(|i| format!("v{i}")).collect();
    let edges: Vec<_> = (1..count)
        .map(|i| json!({"id":i.to_string(),"from":vertices[i-1],"to":vertices[i]}))
        .collect();
    let chain = json!({"vertices":vertices,"edges":edges,"directed":true});
    assert_eq!(
        run(
            "graph.traversal",
            json!({"graph":chain,"mode":"topological"})
        )["result"]["order"],
        json!(vertices)
    );
    assert_eq!(
        run(
            "graph.traversal",
            json!({"graph":chain,"mode":"depth_first"})
        )["result"]["finish"][0],
        "v4095"
    );
}

#[test]
fn weighted_exact_costs_and_finite_routes() {
    let max = i64::MAX.to_string();
    let graph = json!({"vertices":["a","b","c"],"directed":true,"edges":[
        {"id":"ab","from":"a","to":"b","weight":max},{"id":"bc","from":"b","to":"c","weight":max}]});
    for algorithm in [
        json!({"method":"dijkstra"}),
        json!({"method":"bellman_ford"}),
        json!({"method":"a_star","heuristic":[0,5,0]}),
    ] {
        let report = formal::invoke_transport(
            "graph.weighted_path",
            json!({"graph":graph,"start":"a","goal":"c","algorithm":algorithm}),
        )
        .unwrap();
        assert_eq!(report["result"]["cost"], "18446744073709551614");
        assert_eq!(report["result"]["path"]["edges"], json!(["ab", "bc"]));
    }
    let graph = json!({"vertices":["a","b","c","d","e"],"directed":true,"edges":[
        {"id":"ab","from":"a","to":"b","weight":-3},{"id":"ab2","from":"a","to":"b","weight":2},
        {"id":"bc","from":"b","to":"c","weight":4},{"id":"ed","from":"e","to":"d","weight":-2},
        {"id":"de","from":"d","to":"e","weight":1},{"id":"ec","from":"e","to":"c","weight":7}]});
    let vertices = ["a", "b", "c", "d", "e"];
    let paths: Vec<_> = vertices
        .iter()
        .flat_map(|start| {
            vertices
                .iter()
                .map(move |goal| json!({"start":start,"goal":goal}))
        })
        .collect();
    let floyd = run(
        "graph.all_pairs_paths",
        json!({"graph":graph,"paths":paths}),
    );
    for (i, start) in vertices.iter().enumerate() {
        for (j, goal) in vertices.iter().enumerate() {
            let one = run(
                "graph.weighted_path",
                json!({"graph":graph,"start":start,"goal":goal,"algorithm":{"method":"bellman_ford"}}),
            );
            assert_eq!(
                floyd["paths"][i * vertices.len() + j]["status"],
                one["result"]["status"]
            );
            assert_eq!(floyd["distances"][i][j], one["result"]["cost"]);
            assert_eq!(
                floyd["unbounded"][i][j],
                one["result"]["status"] == "unbounded"
            );
        }
    }
    assert_eq!(floyd["distances"][0][2], "1");
    assert_eq!(floyd["unbounded"][3][2], true);
    assert_eq!(floyd["unbounded"][0][2], false);
}

#[test]
fn floyd_exact_relaxation_threshold() {
    let vertices: Vec<_> = (0..256).map(|i| format!("v{i}")).collect();
    let edges: Vec<_> = (1..256)
        .map(|i| json!({"id":i.to_string(),"from":"v0","to":vertices[i],"weight":1}))
        .collect();
    let graph = json!({"vertices":vertices,"edges":edges,"directed":false});
    let mut request = json!({"graph":graph,"max_work":16777216});
    assert_eq!(
        run("graph.all_pairs_paths", request.clone())["work"],
        16777216
    );
    request["max_work"] = json!(16777215);
    assert_eq!(
        formal::invoke_with_cancel("graph.all_pairs_paths", request, &mut || false)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn disjoint_representatives_use_input_order() {
    let report = run(
        "graph.disjoint_sets",
        json!({"ids":[99,0,7,1,2],"actions":[
        {"action":"union","left":2,"right":1},{"action":"union","left":1,"right":7},
        {"action":"union","left":2,"right":99},{"action":"union","left":99,"right":2},
        {"action":"find","id":7},{"action":"same_component","left":99,"right":0}]}),
    );
    assert_eq!(report["components"], json!([[99, 7, 1, 2], [0]]));
    assert_eq!(
        report["answers"],
        json!([{"action":"find","action_index":4,"representative":99},
        {"action":"same_component","action_index":5,"same":false}])
    );
}

fn brute_matching(mask: u16, row: u32, used: u16) -> usize {
    if row == 3 {
        return 0;
    }
    let mut best = brute_matching(mask, row + 1, used);
    for column in 0..3 {
        if used & (1 << column) == 0 && mask & (1 << (3 * row + column)) != 0 {
            best = best.max(1 + brute_matching(mask, row + 1, used | (1 << column)));
        }
    }
    best
}

#[test]
fn matching_and_cover_agree_with_exhaustive_search() {
    for mask in 0..512 {
        let left = ["L0", "L1", "L2"];
        let right = ["R0", "R1", "R2"];
        let edges: Vec<_> = (0..9)
            .filter(|&i| mask & (1 << i) != 0)
            .map(|i| json!({"id":format!("e{i}"),"from":left[i/3],"to":right[i%3]}))
            .collect();
        let report = run(
            "graph.bipartite_matching",
            json!({"left":left,"right":right,"edges":edges,"minimum_cover":true}),
        );
        let expected = brute_matching(mask, 0, 0);
        assert_eq!(
            report["matches"].as_array().unwrap().len(),
            expected,
            "mask {mask}"
        );
        let cover = &report["minimum_cover"];
        let left = cover["left"].as_array().unwrap();
        let right = cover["right"].as_array().unwrap();
        assert_eq!(left.len() + right.len(), expected);
        for edge in &edges {
            assert!(
                left.contains(&edge["from"]) || right.contains(&edge["to"]),
                "mask {mask}"
            );
        }
    }
}

#[test]
fn dancing_links_knuth_example_and_stops() {
    let mut request = json!({"columns":["1","2","3","4","5","6","7"],"max_solutions":10,"rows":[
        {"id":"A","columns":["1","4","7"]},{"id":"B","columns":["1","4"]},
        {"id":"C","columns":["4","5","7"]},{"id":"D","columns":["3","5","6"]},
        {"id":"E","columns":["2","3","6","7"]},{"id":"F","columns":["2","7"]}]});
    let value = run("logic.exact_cover", request.clone());
    assert_eq!(value["solutions"], json!([["B", "D", "F"]]));
    assert_eq!(value["complete"], true);
    let mut polls = 0;
    let stopped = formal::invoke_with_cancel("logic.exact_cover", request.clone(), &mut || {
        polls += 1;
        polls == 47
    })
    .unwrap();
    assert_eq!(stopped["stop_reason"], "cancelled");
    request["max_report_bytes"] = json!(128);
    assert_eq!(
        run("logic.exact_cover", request)["stop_reason"],
        "report_limit"
    );
}

fn clause_satisfied(clause: &[i32], bits: usize) -> bool {
    clause
        .iter()
        .any(|&lit| (bits & (1 << (lit.unsigned_abs() - 1)) != 0) == (lit > 0))
}

#[test]
fn sat_backends_match_truth_tables_and_verify_learned_proofs() {
    let options = [
        vec![1, 2],
        vec![-1, 2],
        vec![1, -2],
        vec![-1, -2],
        vec![1, 1],
        vec![2, -2],
        vec![2, 3],
        vec![-3],
    ];
    for mask in 0..(1 << options.len()) {
        let clauses: Vec<_> = options
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, clause)| clause.clone())
            .collect();
        let formula = json!({"variables":[1,2,3],"clauses":clauses});
        let dpll = run("logic.dpll", json!({"formula":formula}));
        let cdcl = run(
            "logic.cdcl",
            json!({"action":"solve","formula":formula,"proof":true}),
        );
        let satisfiable = (0..8).any(|bits| clauses.iter().all(|c| clause_satisfied(c, bits)));
        for report in [&dpll, &cdcl] {
            if satisfiable {
                assert_eq!(report["result"]["status"], "sat", "mask {mask}");
                let model = report["result"]["assignment"].as_array().unwrap();
                let bits = model
                    .iter()
                    .enumerate()
                    .filter(|(_, row)| row["value"] == true)
                    .fold(0, |bits, (i, _)| bits | (1 << i));
                assert!(clauses.iter().all(|c| clause_satisfied(c, bits)));
            } else {
                assert_eq!(report["result"]["status"], "unsat", "mask {mask}");
            }
        }
        if !satisfiable {
            let replay = run(
                "logic.cdcl",
                json!({"action":"verify_proof","formula":formula,"clauses":cdcl["result"]["proof"]}),
            );
            assert_eq!(replay["result"]["status"], "verified", "mask {mask}");
        }
    }
}

#[test]
fn cdcl_caps_and_cancellation_precede_any_proof_claim() {
    let formula = json!({"variables":[1,2],"clauses":[[1,2],[-1,2],[1,-2],[-1,-2]]});
    let request = json!({"action":"solve","formula":formula,"proof":true});
    let full = run("logic.cdcl", request.clone());
    assert_eq!(full["result"]["status"], "unsat");
    assert!(full["learned_literals"].as_u64().unwrap() > 0);
    for (key, reason) in [
        ("max_work", "work_limit"),
        ("max_conflicts", "conflict_limit"),
        ("max_learned_literals", "learned_literal_limit"),
    ] {
        let mut capped = request.clone();
        capped[key] = json!(0);
        assert_eq!(
            run("logic.cdcl", capped)["result"],
            json!({"status":"incomplete","reason":reason})
        );
    }
    let mut checks = 0;
    let stopped = formal::invoke_with_cancel("logic.cdcl", request, &mut || {
        checks += 1;
        checks == 4
    })
    .unwrap();
    assert_eq!(stopped["work"], 3);
    assert_eq!(
        stopped["result"],
        json!({"status":"incomplete","reason":"cancelled"})
    );
    assert_eq!(
        run(
            "logic.cdcl",
            json!({"action":"verify_proof","formula":formula,"clauses":[[]]})
        )["result"],
        json!({"status":"invalid_proof","clause_index":0})
    );
}

#[test]
fn term_reduction_avoids_capture_and_distinguishes_strategies() {
    let lambda = |term| run("esolang.lambda", json!({"term":term}));
    assert_eq!(lambda(r"(\x.\y.x) y")["term"], r"(\v0.y)");
    assert_eq!(
        run("esolang.lambda", json!({"term":r"\x.f x","eta":true}))["term"],
        "f"
    );
    assert_eq!(
        run("esolang.lambda", json!({"term":r"\x.x x","eta":true}))["steps"],
        0
    );
    let term = r"(\x.answer) ((\s.s s) (\s.s s))";
    assert_eq!(lambda(term)["term"], "answer");
    let applicative = run(
        "esolang.lambda",
        json!({"term":term,"strategy":"applicative_order","max_reductions":5,"trace":2}),
    );
    assert_eq!(applicative["normal_form"], false);
    assert_eq!(applicative["steps"], 5);
    assert_eq!(applicative["omitted_steps"], 3);
    assert_eq!(run("esolang.ski", json!({"term":"S K K I"}))["term"], "I");
    assert_eq!(
        run(
            "esolang.ski",
            json!({"term":"K winner ignored","free_symbols":true})
        )["term"],
        "winner"
    );
}

#[test]
fn automata_boundary_and_stream_continuation() {
    let ca = |initial, boundary, generations| {
        run(
            "esolang.cellular_automaton",
            json!({"initial":initial,"boundary":boundary,"generations":generations}),
        )
    };
    let blinker = json!({"kind":"life","rows":3,"cols":3,"cells":"000111000"});
    assert_eq!(
        ca(blinker.clone(), "finite_dead", 1)["final_grid"]["cells"],
        "010010010"
    );
    assert_eq!(
        ca(blinker, "finite_dead", 2)["final_grid"]["cells"],
        "000111000"
    );
    assert_eq!(
        ca(
            json!({"kind":"elementary","rule":128,"cells":"1"}),
            "toroidal",
            1
        )["final_grid"]["cells"],
        "1"
    );
    assert_eq!(
        ca(
            json!({"kind":"elementary","rule":128,"cells":"1"}),
            "finite_dead",
            1
        )["final_grid"]["cells"],
        "0"
    );
    let lfsr = run(
        "number.lfsr",
        json!({"width":3,"state":1,"tap_mask":3,"count":5,"packing":"lsb_first","tail":"pad_zero"}),
    );
    assert_eq!(lfsr["hex"], "09");
    assert_eq!(lfsr["valid_bits"], 5);
    let mt = |count| {
        run(
            "number.prng",
            json!({"source":{"profile":"mt19937ar_2002","seed":5489},"count":count}),
        )
    };
    let whole = mt(627);
    assert_eq!(
        &whole["values"].as_array().unwrap()[..5],
        &[
            json!(3499211612_u32),
            json!(581869302),
            json!(3890346734_u32),
            json!(3586334585_u32),
            json!(545404204)
        ]
    );
    let start = mt(624);
    assert_eq!(start["final_state"]["state"]["cursor"], 624);
    let continued = run(
        "number.prng",
        json!({"source":start["final_state"],"count":3}),
    );
    assert_eq!(
        continued["values"].as_array().unwrap(),
        &whole["values"].as_array().unwrap()[624..]
    );
    assert_eq!(continued["final_state"], whole["final_state"]);
}

#[test]
fn cdcl_pigeonhole_checks_proof_and_exact_caps() {
    // Four pigeons cannot each occupy one of three holes exclusively.
    let id = |pigeon: i32, hole: i32| 3 * pigeon + hole + 1;
    let mut clauses: Vec<Vec<i32>> = (0..4)
        .map(|pigeon| (0..3).map(|hole| id(pigeon, hole)).collect())
        .collect();
    for hole in 0..3 {
        for left in 0..4 {
            for right in left + 1..4 {
                clauses.push(vec![-id(left, hole), -id(right, hole)]);
            }
        }
    }
    let formula = json!({"variables":(1..=12).collect::<Vec<_>>(),"clauses":clauses});
    let request = json!({"action":"solve","formula":formula,"proof":true});
    let full = run("logic.cdcl", request.clone());
    assert_eq!(full["result"]["status"], "unsat");
    let count = full["conflicts"].as_u64().unwrap();
    assert!(count > 1);
    let proof = &full["result"]["proof"];
    assert_eq!(
        run(
            "logic.cdcl",
            json!({"action":"verify_proof","formula":formula,"clauses":proof})
        )["result"]["status"],
        "verified"
    );
    for (limit, field, reason) in [
        ("max_work", "work", "work_limit"),
        ("max_conflicts", "conflicts", "conflict_limit"),
        (
            "max_learned_literals",
            "learned_literals",
            "learned_literal_limit",
        ),
    ] {
        let mut capped = request.clone();
        let measured = full[field].as_u64().unwrap();
        capped[limit] = json!(measured);
        assert_eq!(run("logic.cdcl", capped.clone()), full);
        capped[limit] = json!(measured - 1);
        assert_eq!(
            run("logic.cdcl", capped)["result"],
            json!({"status":"incomplete","reason":reason})
        );
    }
    let mut probes = 0;
    // Use an observed point after normalization, with learning already active.
    let cancelled = formal::invoke_with_cancel("logic.cdcl", request, &mut || {
        probes += 1;
        probes == 500
    })
    .unwrap();
    assert!(cancelled["learned_literals"].as_u64().unwrap() > 0);
    assert_eq!(
        cancelled["result"],
        json!({"status":"incomplete","reason":"cancelled"})
    );
}

#[test]
fn explicit_limits_and_malformed_requests_stay_typed() {
    let cases = [
        ("esolang.ski", json!({"term":"Z"}), ErrorCode::InvalidInput),
        (
            "esolang.lambda",
            json!({"term":"(\\x.x"}),
            ErrorCode::InvalidInput,
        ),
        (
            "logic.dpll",
            json!({"formula":{"variables":[1],"clauses":[[0]]}}),
            ErrorCode::InvalidInput,
        ),
        (
            "logic.exact_cover",
            json!({"columns":["A","A"],"rows":[],"max_solutions":1}),
            ErrorCode::InvalidInput,
        ),
        (
            "graph.disjoint_sets",
            json!({"ids":[1],"actions":[{"action":"find","id":2}]}),
            ErrorCode::InvalidInput,
        ),
        (
            "graph.traversal",
            json!({"graph":{"vertices":[],"edges":[],"directed":false},"mode":"topological"}),
            ErrorCode::InvalidParameter,
        ),
        (
            "number.prng",
            json!({"source":{"profile":"xorshift32","seed":0},"count":0}),
            ErrorCode::InvalidInput,
        ),
        (
            "number.lfsr",
            json!({"width":64,"state":1,"tap_mask":1,"count":7,"packing":"msb_first","tail":"reject"}),
            ErrorCode::InvalidParameter,
        ),
        (
            "esolang.cellular_automaton",
            json!({"initial":{"kind":"elementary","rule":30,"cells":"00100"},"generations":1,"max_work":4}),
            ErrorCode::ResourceLimit,
        ),
    ];
    for (operation, request, expected) in cases {
        let error = formal::invoke_with_cancel(operation, request, &mut || false).unwrap_err();
        assert_eq!(error.code, expected, "{operation}");
        assert_eq!(error.operation, Some(operation));
    }
}
