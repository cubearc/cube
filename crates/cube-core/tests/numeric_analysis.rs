use cube_core::{numbers::analysis, operations};
use serde_json::{Value, json};

fn run(name: &str, request: Value) -> Value {
    analysis::invoke(&format!("number.{name}"), request).unwrap()
}

fn reject(name: &str, request: Value) {
    let op = format!("number.{name}");
    let error = analysis::invoke(&op, request).unwrap_err();
    assert_eq!(error.operation, Some(operations::require(&op).unwrap().id));
}

#[test]
fn shared_known_answers_execute_on_the_lossless_transport() {
    let fixtures = operations::contracts::registered_fixtures().unwrap();
    let mut count = 0;
    for (op, fixture) in fixtures {
        if analysis::OPERATIONS.iter().any(|row| row.id == op) {
            let actual =
                analysis::invoke_transport(&op, fixture.inputs["request"].clone()).unwrap();
            assert_eq!(actual, fixture.outputs["report"], "{op}");
            count += 1;
        }
    }
    assert_eq!(count, 23);
}

#[test]
fn exact_representations_and_grammar() {
    for n in 1..=3999 {
        let numeral = run("roman", json!({"direction":"encode","value":n}));
        let reverse = run(
            "roman",
            json!({"direction":"decode","text":numeral["text"]}),
        );
        assert_eq!(reverse, numeral);
        assert!(numeral["text"].as_str().unwrap().len() <= 15);
    }
    for text in [
        "IIII",
        "IL",
        "MMMM",
        "Ⅳ",
        "ix",
        "",
        "X ",
        "I".repeat(16).as_str(),
    ] {
        reject("roman", json!({"direction":"decode","text":text}));
    }
    assert_eq!(
        run(
            "roman",
            json!({"direction":"decode","text":"ix","ascii_case_insensitive":true})
        )["value"],
        9
    );
    let max = u128::MAX.to_string();
    let min = format!("-{max}");
    for value in ["0", "1", "-1", "2", "-2", &max, &min] {
        let forward = run(
            "balanced_ternary",
            json!({"direction":"encode","value":value}),
        );
        let reverse = run(
            "balanced_ternary",
            json!({"direction":"decode","text":forward["text"]}),
        );
        assert_eq!(forward, reverse);
        assert_eq!(reverse["value"], value);
    }
    reject(
        "balanced_ternary",
        json!({"direction":"decode","text":"001"}),
    );
    assert_eq!(
        run(
            "balanced_ternary",
            json!({"direction":"decode","text":"001","canonicalize":true})
        )["text"],
        "1"
    );
    for width in [1, 7, 32, 64, 127, 128] {
        let value = (u128::MAX >> (128 - width)).to_string();
        let encoded = run(
            "gray",
            json!({"direction":"encode","value":value,"width":width,"binary":true}),
        );
        assert_eq!(encoded["binary"].as_str().unwrap().len(), width);
        assert_eq!(
            run(
                "gray",
                json!({"direction":"decode","value":encoded["value"],"width":width})
            )["value"],
            value
        );
    }
    reject(
        "gray",
        json!({"direction":"encode","value":"256","width":8}),
    );
    reject("gray", json!({"direction":"encode","value":"0","width":0}));
    assert_eq!(
        run("decimal_check", json!({"action":"summarize","value":"999"})),
        json!({"action":"summarize","digit_sum":27,"digital_root":9,"residue9":0})
    );
    for (op, left, right, claimed) in [
        ("add", "38", "61", "108"),
        ("subtract", "-2", "1", "-3"),
        ("multiply", "-12", "9", "-108"),
    ] {
        assert_eq!(
            run(
                "decimal_check",
                json!({"action":"check","operation":op,"left":left,"right":right,"claimed":claimed})
            )["congruent"],
            true
        );
    }
    for value in [
        "",
        "-",
        "1e2",
        " 1",
        "340282366920938463463374607431768211456",
    ] {
        reject("decimal_check", json!({"action":"summarize","value":value}));
    }
    let limit = "0".repeat(4096);
    assert_eq!(
        run("decimal_check", json!({"action":"summarize","value":limit}))["digit_sum"],
        0
    );
    reject(
        "decimal_check",
        json!({"action":"summarize","value":format!("{limit}0")}),
    );
}

#[test]
fn exact_number_theory_and_bounded_recurrences() {
    assert_eq!(
        run("crt", json!({"pairs":[]})),
        json!({"status":"solution","residue":0,"modulus":1})
    );
    let answer = run(
        "crt",
        json!({"pairs":[{"residue":5,"modulus":6},{"residue":10,"modulus":8}]}),
    );
    assert_eq!(answer["status"], "inconsistent");
    assert_eq!(answer["gcd"], 2);
    reject("crt", json!({"pairs":[{"residue":0,"modulus":0}]}));
    reject(
        "crt",
        json!({"pairs":[{"residue":0,"modulus":u64::MAX},{"residue":0,"modulus":2}]}),
    );
    assert_eq!(
        analysis::invoke_transport(
            "number.crt",
            json!({"pairs":[{"residue":"18446744073709551614","modulus":"18446744073709551615"}]})
        )
        .unwrap()["modulus"],
        "18446744073709551615"
    );

    let sieve = run("sieve", json!({"limit":1000000}));
    assert_eq!(sieve["primes"].as_array().unwrap().len(), 78498);
    assert_eq!(
        sieve["primes"],
        run("sieve", json!({"limit":1000000,"method":"atkin"}))["primes"]
    );
    for n in 0..=512 {
        let trial = run("primality", json!({"value":n}));
        assert_eq!(
            trial["prime"],
            run("primality", json!({"value":n,"method":"miller_rabin"}))["prime"]
        );
        assert_eq!(
            trial["prime"],
            sieve["primes"]
                .as_array()
                .unwrap()
                .binary_search_by_key(&u64::from(n as u32), |x| x.as_u64().unwrap())
                .is_ok()
        );
    }
    for n in [341, 561, 3215031751_u32, u32::MAX] {
        assert_eq!(
            run("primality", json!({"value":n,"method":"miller_rabin"}))["prime"],
            false
        );
    }
    reject("sieve", json!({"limit":1000001}));
    reject("sieve", json!({"limit":10,"max_results":3}));
    assert_eq!(
        run("factor", json!({"method":"trial_division","value":1}))["factors"],
        json!([])
    );
    reject("factor", json!({"method":"trial_division","value":0}));
    let factor = run(
        "factor",
        json!({"method":"pollard_rho","value":8051,"x0":2,"c":1,"steps":100}),
    );
    assert!(matches!(factor["factor"].as_u64(), Some(83 | 97)));
    let inconclusive = run(
        "factor",
        json!({"method":"pollard_rho","value":101,"x0":2,"c":1,"steps":64}),
    );
    assert_eq!(inconclusive["factor"], Value::Null);
    assert_eq!(inconclusive["complete"], false);

    assert_eq!(
        run(
            "recurrence",
            json!({"sequence":"arithmetic","first":1,"difference":-3,"count":3})
        )["values"],
        json!([1, -2, -5])
    );
    assert_eq!(
        run(
            "recurrence",
            json!({"sequence":"geometric","first":2,"ratio":-2,"count":4})
        )["values"],
        json!([2, -4, 8, -16])
    );
    assert_eq!(
        run("recurrence", json!({"sequence":"lucas","count":5}))["values"],
        json!([2, 1, 3, 4, 7])
    );
    assert_eq!(
        run("recurrence", json!({"sequence":"fibonacci","count":94}))["values"][93],
        12200160415121876738_u64
    );
    reject("recurrence", json!({"sequence":"fibonacci","count":95}));
    reject(
        "recurrence",
        json!({"sequence":"arithmetic","first":i64::MAX,"difference":1,"count":2}),
    );
    assert_eq!(
        run(
            "recurrence",
            json!({"sequence":"collatz","start":3,"steps":7,"retain":8})
        )["values"],
        json!([3, 10, 5, 16, 8, 4, 2, 1])
    );
    assert_eq!(
        run(
            "recurrence",
            json!({"sequence":"collatz","start":u64::MAX,"steps":1,"retain":0})
        )["stop_reason"],
        "overflow"
    );
    assert_eq!(
        run(
            "recurrence",
            json!({"sequence":"collatz","start":3,"steps":1,"retain":1})
        )["complete"],
        false
    );
}

#[test]
fn combinatorial_prefixes_ranks_and_cycles_match_small_oracles() {
    for n in 0..=6u32 {
        let tokens: Vec<_> = (0..n).rev().collect();
        let rows = run(
            "permutation_list",
            json!({"mode":"multiset","tokens":tokens,"max_results":10000}),
        );
        for (i, permutation) in rows["permutations"].as_array().unwrap().iter().enumerate() {
            let rank = i.to_string();
            assert_eq!(
                run("lehmer", json!({"action":"rank","permutation":permutation}))["rank"],
                rank
            );
            assert_eq!(
                run("lehmer", json!({"action":"unrank","size":n,"rank":rank}))["permutation"],
                *permutation
            );
        }
        let swaps = run(
            "permutation_list",
            json!({"mode":"johnson_trotter","tokens":tokens,"max_results":10000}),
        );
        let adjacent = swaps["permutations"].as_array().unwrap();
        assert_eq!(
            adjacent.len(),
            rows["permutations"].as_array().unwrap().len()
        );
        for pair in adjacent.windows(2) {
            let differences: Vec<_> = pair[0]
                .as_array()
                .unwrap()
                .iter()
                .zip(pair[1].as_array().unwrap())
                .enumerate()
                .filter_map(|(i, (a, b))| (a != b).then_some(i))
                .collect();
            assert_eq!(differences.len(), 2);
            assert_eq!(differences[0] + 1, differences[1]);
        }
    }
    reject(
        "permutation_list",
        json!({"mode":"johnson_trotter","tokens":[1,1]}),
    );
    assert_eq!(
        run(
            "permutation_list",
            json!({"mode":"multiset","tokens":[1,2],"max_work":0})
        )["stop_reason"],
        "work_limit"
    );
    let maximum = u128::MAX.to_string();
    let coded = run("factoradic", json!({"direction":"encode","value":maximum}));
    assert_eq!(
        run(
            "factoradic",
            json!({"direction":"decode","digits":coded["digits"]})
        )["value"],
        maximum
    );
    reject("factoradic", json!({"direction":"decode","digits":[0,0]}));
    reject("lehmer", json!({"action":"from_digits","digits":[2,0]}));
    let words = ["BAA", "ABA", "AAB"];
    for (i, expected) in words.iter().enumerate() {
        assert_eq!(
            run(
                "lexicographic_rank",
                json!({"action":"unrank","alphabet":"BA","multiset":"ABA","rank":i.to_string()})
            )["text"],
            *expected
        );
    }

    for (n, expected) in [1, 1, 2, 3, 5, 7, 11, 15, 22, 30, 42]
        .into_iter()
        .enumerate()
    {
        let all = run("partitions", json!({"action":"enumerate","n":n}));
        let rows = all["partitions"].as_array().unwrap();
        assert_eq!(rows.len(), expected);
        assert_eq!(all["count"], expected.to_string());
        for maximum in 0..=n {
            for parts in 0..=n {
                let exhaustive = rows
                    .iter()
                    .filter(|row| {
                        let row = row.as_array().unwrap();
                        row.len() == parts
                            && row
                                .first()
                                .is_none_or(|item| item.as_u64().unwrap() <= maximum as u64)
                    })
                    .count();
                assert_eq!(
                    run(
                        "partitions",
                        json!({"action":"count","n":n,"max_part":maximum,"part_count":parts})
                    )["count"],
                    exhaustive.to_string()
                );
            }
        }
    }
    reject("partitions", json!({"action":"count","n":257}));
    reject("partitions", json!({"action":"enumerate","n":65}));
    assert_eq!(
        run(
            "partitions",
            json!({"action":"enumerate","n":64,"max_results":2})
        )["stop_reason"],
        "result_limit"
    );

    for alphabet in ["é", "01", "aβǰ"] {
        for order in 1..=5 {
            let result = run(
                "de_bruijn",
                json!({"alphabet":alphabet,"order":order,"linearized":true}),
            );
            let sequence: Vec<_> = result["sequence"].as_str().unwrap().chars().collect();
            let size = alphabet.chars().count().pow(order as u32);
            assert_eq!(sequence.len(), size + order - 1);
            let words: std::collections::HashSet<_> = sequence.windows(order).collect();
            assert_eq!(words.len(), size);
        }
    }
    reject("de_bruijn", json!({"alphabet":"aa","order":2}));
    reject("de_bruijn", json!({"alphabet":"01","order":21}));
}

fn fraction(n: &str, d: &str) -> Value {
    json!({"numerator":n,"denominator":d})
}

#[test]
fn reduced_rationals_matrix_elimination_and_polynomial_evaluation() {
    let empty = json!({"rows":0,"cols":0,"cells":[]});
    let a = json!({"rows":2,"cols":2,"cells":[1,2,3,4]});
    assert_eq!(
        run("matrix", json!({"action":"determinant","matrix":empty}))["value"],
        fraction("1", "1")
    );
    assert_eq!(
        run("matrix", json!({"action":"inverse","matrix":empty}))["matrix"],
        empty
    );
    let inverse = run("matrix", json!({"action":"inverse","matrix":a}));
    let product = run(
        "matrix",
        json!({"action":"multiply","left":a,"right":inverse["matrix"]}),
    );
    assert_eq!(
        product["matrix"]["cells"],
        json!([
            fraction("1", "1"),
            fraction("0", "1"),
            fraction("0", "1"),
            fraction("1", "1")
        ])
    );
    assert_eq!(
        run("matrix", json!({"action":"solve","matrix":a,"rhs":[3,7]}))["solution"],
        json!([fraction("1", "1"), fraction("1", "1")])
    );
    let singular = json!({"rows":2,"cols":2,"cells":[1,2,2,4]});
    assert_eq!(
        run("matrix", json!({"action":"inverse","matrix":singular}))["singular"],
        true
    );
    assert_eq!(
        run(
            "matrix",
            json!({"action":"solve","matrix":singular,"rhs":[1,2]})
        )["status"],
        "underdetermined"
    );
    assert_eq!(
        run(
            "matrix",
            json!({"action":"solve","matrix":singular,"rhs":[1,3]})
        )["status"],
        "inconsistent"
    );
    assert_eq!(
        run("matrix", json!({"action":"reduce","matrix":singular}))["rank"],
        1
    );
    assert_eq!(
        run("matrix", json!({"action":"transpose","matrix":singular}))["matrix"]["cells"][3],
        fraction("4", "1")
    );
    reject(
        "matrix",
        json!({"action":"determinant","matrix":{"rows":0,"cols":1,"cells":[]}}),
    );
    reject(
        "matrix",
        json!({"action":"determinant","matrix":{"rows":1,"cols":1,"cells":[fraction("1","0")]}}),
    );
    let points = json!([{"x":0,"y":1},{"x":1,"y":2},{"x":2,"y":5}]);
    for method in ["newton", "lagrange"] {
        let r = run(
            "interpolation",
            json!({"action":"interpolate","method":method,"samples":points,"points":[-1,3,fraction("1","2")],"coefficients":true}),
        );
        assert_eq!(
            r["values"],
            json!([fraction("2", "1"), fraction("10", "1"), fraction("5", "4")])
        );
        assert_eq!(
            r["coefficients"],
            json!([fraction("1", "1"), fraction("0", "1"), fraction("1", "1")])
        );
    }
    assert_eq!(
        run(
            "interpolation",
            json!({"action":"differences","values":[1,2,5]})
        )["rows"][2],
        json!([fraction("2", "1")])
    );
    reject(
        "interpolation",
        json!({"action":"interpolate","method":"newton","samples":[{"x":1,"y":1},{"x":fraction("2","2"),"y":1}],"points":[1]}),
    );
}

#[test]
fn explicit_statistical_evidence_and_letter_variants() {
    for (method, alpha) in [
        ("shannon", None),
        ("renyi", Some(1.)),
        ("renyi", Some(2.)),
        ("miller_madow", None),
    ] {
        let mut request = json!({"method":method,"counts":[5,0,5]});
        if let Some(alpha) = alpha {
            request["alpha"] = json!(alpha);
        }
        let observed = run("entropy", request);
        let expected = 1.
            + if method == "miller_madow" {
                1. / (20. * std::f64::consts::LN_2)
            } else {
                0.
            };
        assert!((observed["bits"].as_f64().unwrap() - expected).abs() < 1e-12);
    }
    reject(
        "entropy",
        json!({"method":"renyi","counts":[1],"alpha":1.0000001}),
    );
    for (alpha, expected) in [
        (1. - 2_f64.powi(-19), 1.4563648236155238_f64),
        (1. + 2_f64.powi(-19), 1.4563627626754359_f64),
    ] {
        let estimate = run(
            "entropy",
            json!({"method":"renyi","counts":[1,7,12,31,91],"alpha":alpha}),
        );
        assert!((estimate["bits"].as_f64().unwrap() - expected).abs() < 1e-12);
    }
    reject("entropy", json!({"method":"shannon","counts":[0]}));
    assert_eq!(
        run(
            "correlation",
            json!({"method":"pearson","x":[4,4,4],"y":[1,2,3]})
        )["value"],
        Value::Null
    );
    assert!(
        run(
            "correlation",
            json!({"method":"spearman","x":[2,2,1],"y":[3,3,5]})
        )["value"]
            .as_f64()
            .unwrap()
            < -0.9999999999
    );
    let r = run(
        "correlation",
        json!({"method":"lag","x":[1,2],"y":[3,4,5],"lags":[0,1,2],"normalize":"population_covariance"}),
    );
    assert_eq!(r["lags"][2]["value"], Value::Null);
    reject("correlation", json!({"method":"lag","x":[3],"lags":[1]}));
    assert_eq!(
        run(
            "runs",
            json!({"input":"threshold","values":[-1.,0.,1.,1.,-1.],"threshold":0.,"tie":"drop"})
        )["runs"],
        3
    );
    assert_eq!(
        run("runs", json!({"input":"binary","values":[]}))["undefined_reason"],
        "empty"
    );
    reject("runs", json!({"input":"binary","values":[2]}));
    reject("benford", json!({"values":["10e2"]}));
    assert_eq!(
        run("benford", json!({"values":["0","-000","+000"]}))["zeros"],
        3
    );
    reject("markov", json!({"states":["x"],"sequence":["y"]}));
    reject("markov", json!({"states":["x","x"],"sequence":[]}));

    for (preset, text, expected) in [
        ("ordinal", "AZ", 27),
        ("reverse_ordinal", "AB", 51),
        ("reduction", "JZ", 9),
        ("pythagorean", "JZ", 9),
        ("full_reduction", "JZ", 9),
        ("reverse_reduction", "ZY", 3),
        ("chaldean_cheiro", "CHEIRO", 23),
        ("hebrew_ordinary", "שלום", 376),
        ("hebrew_final_500", "שלום", 936),
        ("abjad_eastern", "سضغ", 1860),
        ("abjad_maghrebi", "سضغ", 1290),
    ] {
        assert_eq!(
            run(
                "letter_values",
                json!({"action":"evaluate","mapping":{"preset":preset},"text":text})
            )["sum"],
            expected
        );
        let report = analysis::invoke_transport(
            "number.letter_values",
            json!({"action":"table","preset":preset}),
        )
        .unwrap();
        assert_eq!(report["table_sha256"].as_str().unwrap().len(), 64);
    }
    let r = run(
        "letter_values",
        json!({"action":"evaluate","mapping":{"preset":"ordinal"},"text":"a🧊b","case":"ascii_upper","unmapped":"skip","aggregate":"positional_weighted_sum","weights":[4,500,6]}),
    );
    assert_eq!(r["value"], "16");
    assert_eq!(r["values"][2]["byte_offset"], 5);
    assert_eq!(
        run(
            "letter_values",
            json!({"action":"evaluate","mapping":{"table":[["x",0],["y",1]]},"text":"x","aggregate":"product"})
        )["value"],
        "0"
    );
    assert_eq!(
        run(
            "letter_values",
            json!({"action":"evaluate","mapping":{"preset":"ordinal"},"text":"","aggregate":"product"})
        )["value"],
        "1"
    );
    reject(
        "letter_values",
        json!({"action":"evaluate","mapping":{"table":[["x",1],["x",2]]},"text":"x"}),
    );
    reject(
        "letter_values",
        json!({"action":"evaluate","mapping":{"preset":"hebrew_ordinary"},"text":"שָׁלוֹם"}),
    );
    reject(
        "letter_values",
        json!({"action":"evaluate","mapping":{"preset":"ordinal"},"text":"Z","max_sum":25}),
    );
}
