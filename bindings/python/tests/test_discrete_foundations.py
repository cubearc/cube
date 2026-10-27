import collections
import itertools
import math

import cube
import pytest


def test_exact_builtin_integer_counts_and_pascal_oracle():
    for n in range(129):
        for k in range(n + 1):
            assert cube.binomial(n, k) == math.comb(n, k)
    for n in [0, 1, 5, 20, 34, 35, 128]:
        for k in range(n + 1):
            expected = math.perm(n, k)
            if expected < 2**128:
                assert cube.permutations(n, k) == expected
            else:
                with pytest.raises(ValueError) as exc:
                    cube.permutations(n, k)
                assert exc.value.code == "resource_limit"
    for groups in [[], [0], [2, 1, 1], [20, 20, 20], [0] * 128]:
        expected = math.factorial(sum(groups))
        for n in groups:
            expected //= math.factorial(n)
        assert cube.multinomial(groups) == expected
    rows = cube.pascal_triangle(129)
    assert rows == [[math.comb(n, k) for k in range(n + 1)] for n in range(129)]
    assert cube.pascal_triangle(0) == []
    for call in [
        lambda: cube.binomial(1, 2),
        lambda: cube.multinomial([128, 1]),
        lambda: cube.binomial(5, 2, limits={"max_work": 0}),
        lambda: cube.binomial(0, 0, limits={"max_report_bytes": 127}),
        lambda: cube.pascal_triangle(130),
    ]:
        with pytest.raises(ValueError):
            call()


@pytest.mark.parametrize("value", [True, 1.5, -1, 2**64, "1", None])
def test_count_and_budget_integer_types_do_not_coerce(value):
    for call in [
        lambda: cube.binomial(value, 0),
        lambda: cube.permutations(1, value),
        lambda: cube.multinomial([value]),
        lambda: cube.pascal_triangle(value),
        lambda: cube.binomial(0, 0, limits={"max_work": value}),
    ]:
        with pytest.raises(ValueError):
            call()


def test_native_multisets_share_scalar_preparation_and_complete_counts():
    for source, requested in [
        ("listen", "silent"),
        ("aab", "ab"),
        ("ab", "aab"),
        ("ab", "aa"),
        ("", ""),
        ("\0é雪🧊", "雪é\0"),
    ]:
        r = cube.multiset_compare(source, requested)
        a, b = collections.Counter(source), collections.Counter(requested)
        assert r["source"] == [{"scalar": c, "count": n} for c, n in sorted(a.items())]
        assert r["requested"] == [
            {"scalar": c, "count": n} for c, n in sorted(b.items())
        ]
        assert r["residual"] == [
            {"scalar": c, "count": n} for c, n in sorted((a - b).items())
        ]
        assert r["deficits"] == [
            {"scalar": c, "count": n} for c, n in sorted((b - a).items())
        ]
        assert r["equal"] == (a == b)
        assert r == cube.multiset_subtract(source, requested)
    assert cube.multiset_compare("é", "e\u0301", profile={"normalization": "NFC"})[
        "equal"
    ]
    assert cube.multiset_compare(
        "Aa!", "aa", profile={"case": "ascii_lower", "filter": "ascii_letters"}
    )["equal"]
    r = cube.multiset_compare("aab", "ab")
    exact = {
        "max_work": r["work"],
        "max_memory_bytes": r["memory_bytes_upper_bound"],
        "max_report_bytes": r["report_bytes_upper_bound"],
    }
    assert cube.multiset_compare("aab", "ab", limits=exact) == r
    for name in exact:
        with pytest.raises(ValueError):
            cube.multiset_compare("aab", "ab", limits={**exact, name: exact[name] - 1})


def var(name):
    return {"op": "var", "name": name}


def model(op):
    return {
        "variables": ["A", "B"],
        "formula": {"op": op, "left": var("A"), "right": var("B")},
        "other": var("B"),
    }


def test_native_boolean_exact_tables_counterexamples_and_complete_assignments():
    operators = {
        "and": lambda a, b: a and b,
        "or": lambda a, b: a or b,
        "xor": lambda a, b: a != b,
        "implies": lambda a, b: not a or b,
        "iff": lambda a, b: a == b,
    }
    for name, operation in operators.items():
        r = cube.boolean_analyze(model(name))
        expected = [
            operation(a, b) for a, b in itertools.product([False, True], repeat=2)
        ]
        assert [row["value"] for row in r["rows"]] == expected
        assert r["true_count"] == sum(expected)
        assert r["listing_complete"]
        for row in r["rows"]:
            e = cube.boolean_evaluate(model(name), row["assignment"])
            assert e["value"] == row["value"]
            assert e["other_value"] == row["other_value"]
    r = cube.boolean_analyze(model("implies"), limits={"max_rows": 0})
    assert r["true_count"] == 3 and r["rows"] == [] and not r["listing_complete"]
    assert not r["relationship"]["implies"]
    witness = r["relationship"]["implication_counterexample"]
    e = cube.boolean_evaluate(model("implies"), witness)
    assert e["value"] and not e["other_value"]
    for bad in [
        [],
        [{"name": "A", "value": True}] * 2,
        [{"name": "A", "value": 1}, {"name": "B", "value": False}],
    ]:
        with pytest.raises(ValueError):
            cube.boolean_evaluate(model("and"), bad)
    with pytest.raises(ValueError) as exc:
        cube.boolean_analyze(model("and"), limits={"max_work": 11})
    assert exc.value.code == "resource_limit"
    with pytest.raises(ValueError):
        cube.boolean_analyze(
            '{"variables":[],"variables":[],"formula":{"op":"const","value":true}}'
        )


def test_native_graph_edge_identity_orientations_and_unreachable_results():
    g = {
        "vertices": ["a", "b", "z"],
        "directed": False,
        "edges": [
            {"id": "x", "from": "a", "to": "b"},
            {"id": "y", "from": "a", "to": "b"},
        ],
    }
    assert cube.graph_components(g)["result"] == [["a", "b"], ["z"]]
    assert cube.graph_shortest_path(g, "a", "b")["result"] == {
        "vertices": ["a", "b"],
        "edges": ["x"],
    }
    assert cube.graph_shortest_path(g, "z", "b")["result"] is None
    assert cube.graph_cycle(g)["result"] == {
        "vertices": ["a", "b", "a"],
        "edges": ["x", "y"],
    }
    e = cube.graph_euler_trail(g, start="b")
    assert e["result"]["circuit"]
    assert e["result"]["path"]["edges"] == ["x", "y"]
    directed = {**g, "directed": True, "edges": g["edges"][:1]}
    assert cube.graph_shortest_path(directed, "b", "a")["result"] is None
    assert cube.graph_cycle(directed)["result"] is None
    assert cube.graph_euler_trail(directed)["result"]["path"]["vertices"] == ["a", "b"]
    for call in [
        lambda: cube.graph_euler_trail(g, start="z"),
        lambda: cube.graph_components(g, limits={"max_work": 0}),
        lambda: cube.graph_components({**g, "vertices": ["a", "a"]}),
        lambda: cube.graph_components({**g, "directed": 1}),
    ]:
        with pytest.raises(ValueError):
            call()


def test_discrete_metadata_and_hostile_json_transport_stay_bounded():
    operations = [
        "number.permutations",
        "number.binomial",
        "number.multinomial",
        "number.pascal",
        "text.multiset.compare",
        "text.multiset.subtract",
        "logic.boolean.evaluate",
        "logic.boolean.analyze",
        "graph.components",
        "graph.shortest_path",
        "graph.cycle",
        "graph.euler_trail",
    ]
    for name in operations:
        assert cube.describe_operation(name)["pipeline"] is None
    bad = []
    bad.append(bad)
    with pytest.raises(ValueError):
        cube.boolean_analyze({"variables": [], "formula": bad})
    for value in [1.0, {1: "bad"}, {"a": object()}, " " * (1024**2 + 1)]:
        with pytest.raises(ValueError):
            cube.boolean_analyze(value)
    with pytest.raises(ValueError):
        cube.graph_components(
            '{"vertices":[],"directed":true,"edges":[],"directed":false}'
        )


@pytest.mark.parametrize("value", [1.0, 1, "true", None])
def test_model_boolean_type_errors_agree_for_objects_and_raw_json(value):
    import json

    data = {"variables": [], "formula": {"op": "const", "value": value}}
    for source in [data, json.dumps(data)]:
        with pytest.raises(ValueError) as exc:
            cube.boolean_analyze(source)
        assert exc.value.code == "invalid_input"
