"""Run the formal operation contracts with the native extension."""

import json
from pathlib import Path

import cube
import pytest
from test_numeric_analysis import native_values

OPERATIONS = {
    "graph.traversal",
    "graph.weighted_path",
    "graph.all_pairs_paths",
    "graph.disjoint_sets",
    "graph.bipartite_matching",
    "logic.exact_cover",
    "logic.dpll",
    "logic.cdcl",
    "esolang.lambda",
    "esolang.ski",
    "esolang.cellular_automaton",
    "number.lfsr",
    "number.prng",
}
EXACT_FIELDS = {
    "graph.weighted_path": {"cost"},
    "graph.all_pairs_paths": {"cost", "distances"},
    "number.lfsr": {"final_state"},
}
FIXTURES = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)
CASES = [
    row
    for row in json.loads(FIXTURES.read_text(encoding="utf-8"))
    if row["operation"] in OPERATIONS
]


def test_formal_fixtures_cover_each_operation():
    assert {case["operation"] for case in CASES} == OPERATIONS


@pytest.mark.parametrize("case", CASES, ids=[row["operation"] for row in CASES])
def test_native_formal_contracts(case):
    operation = case["operation"]
    function = operation.replace(".", "_")
    assert function in cube.__all__
    assert getattr(cube, function)(case["inputs"]["request"]) == native_values(
        case["outputs"]["report"], EXACT_FIELDS.get(operation, set())
    )


def test_native_full_width_distance_and_stream_state():
    weight = 2**63 - 1
    graph = {
        "vertices": ["a", "b", "c"],
        "directed": True,
        "edges": [
            {"id": "ab", "from": "a", "to": "b", "weight": weight},
            {"id": "bc", "from": "b", "to": "c", "weight": weight},
        ],
    }
    assert (
        cube.graph_weighted_path(
            {
                "graph": graph,
                "start": "a",
                "goal": "c",
                "algorithm": {"method": "dijkstra"},
            }
        )["result"]["cost"]
        == 2 * weight
    )
    assert cube.graph_all_pairs_paths({"graph": graph})["distances"][0][2] == 2 * weight
    state = 2**64 - 1
    assert (
        cube.number_lfsr(
            {
                "width": 64,
                "state": state,
                "tap_mask": 1,
                "count": 8,
                "packing": "lsb_first",
                "tail": "reject",
            }
        )["final_state"]
        == state
    )


def test_native_proof_and_cancellation():
    formula = {
        "variables": [1, 2],
        "clauses": [[1, 2], [-1, 2], [1, -2], [-1, -2]],
    }
    request = {"action": "solve", "formula": formula, "proof": True}
    proof = cube.logic_cdcl(request)["result"]["proof"]
    assert cube.logic_cdcl(
        {"action": "verify_proof", "formula": formula, "clauses": proof}
    )["result"] == {"status": "verified"}
    token = cube.CancellationToken()
    token.cancel()
    assert cube.logic_cdcl(request, cancel=token)["result"] == {
        "status": "incomplete",
        "reason": "cancelled",
    }
    with pytest.raises(ValueError):
        cube.logic_dpll({"formula": {"variables": [1], "clauses": [[0]]}})
