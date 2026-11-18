"""Execute sequence contracts through the installed native Python extension."""

import json
from pathlib import Path

import cube
import pytest

FIXTURES = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)
CASES = [
    row
    for row in json.loads(FIXTURES.read_text(encoding="utf-8"))
    if row["operation"].startswith("sequence.")
]


@pytest.mark.parametrize("case", CASES, ids=[row["operation"] for row in CASES])
def test_native_sequence_contract(case):
    operation = case["operation"].replace(".", "_")
    assert operation in cube.__all__
    assert (
        getattr(cube, operation)(case["inputs"]["request"]) == case["outputs"]["report"]
    )


def test_float_requests_and_duplicate_fields_are_checked_natively():
    exact = cube.sequence_dtw({"left": [0.5, 2.0], "right": [0.5, 1.0], "radius": 0})
    assert exact == {"cost": 1, "path": [[0, 0], [1, 1]]}
    with pytest.raises(ValueError):
        cube.sequence_dtw({"left": [float("nan")], "right": [0]})
    with pytest.raises(ValueError):
        cube.sequence_hamming('{"unit":"bytes","left":"","left":"YQ==","right":""}')
