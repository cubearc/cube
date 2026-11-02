"""Actual native five-bit switch network with explicit fixed cam facts."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/t52ab_vectors.json"
    ).read_text()
)


def test_native_t52ab_primary_switch_answer_and_continuation():
    p = VECTOR["settings"]
    result = cube.t52ab_encode(VECTOR["codes"], p)
    assert result["codes"] == VECTOR["expected"]
    assert result["trace"][0]["permutation"] == [4, 1, 2, 3, 0]
    assert cube.t52ab_decode(result["codes"], p)["codes"] == VECTOR["codes"]
    first = cube.t52ab_encode(VECTOR["codes"][:3], p)
    second = cube.t52ab_encode(
        VECTOR["codes"][3:], {**p, "initial_state": first["final_state"]}
    )
    assert first["codes"] + second["codes"] == result["codes"]
    assert second["final_state"] == result["final_state"]
    assert cube.t52ab_encode([], p)["final_state"] == p["initial_state"]
    for codes in [[32], [-1], [True], ["PRIVATE_CODE"]]:
        with pytest.raises(ValueError) as caught:
            cube.t52ab_encode(codes, p)
        assert caught.value.code == "invalid_input"
        assert "PRIVATE_CODE" not in str(caught.value)
    with pytest.raises(ValueError):
        cube.t52ab_encode([0], {**p, "kt": True})


def test_native_t52ab_code_and_trace_boundaries():
    p = {**VECTOR["settings"], "trace": False}
    assert len(cube.t52ab_encode([31] * 1048576, p)["codes"]) == 1048576
    for codes, settings in [([0] * 1048577, p), ([0] * 65537, {**p, "trace": True})]:
        with pytest.raises(ValueError) as caught:
            cube.t52ab_encode(codes, settings)
        assert caught.value.code == "resource_limit"
