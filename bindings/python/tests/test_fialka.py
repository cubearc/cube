"""Native Fialka independent reference, continuation and exact bounds."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/fialka_vectors.json"
    ).read_text()
)


def test_native_fialka_source_answer_and_continuation():
    v = VECTOR
    p = v["settings"]
    r = cube.fialka_encode(v["codes"], p)
    assert r["codes"] == v["expected"]
    assert r["final_state"] == v["expected_final_state"]
    assert cube.fialka_decode(r["codes"], p)["codes"] == v["codes"]
    first = cube.fialka_encode(v["codes"][:9], p)
    second = cube.fialka_encode(
        v["codes"][9:], {**p, "initial_state": first["final_state"]}
    )
    assert first["codes"] + second["codes"] == r["codes"]
    assert second["final_state"] == r["final_state"]
    assert cube.fialka_encode([], p)["final_state"] == p["initial_state"]
    with pytest.raises(ValueError):
        cube.fialka_encode([30], p)
    with pytest.raises(ValueError):
        cube.fialka_encode([0], {**p, "profile": "PRIVATE_UNKNOWN_PROFILE"})


def test_native_fialka_input_trace_and_report_limits():
    p = {**VECTOR["settings"], "trace": False}
    assert len(cube.fialka_encode([0] * 1048576, p)["codes"]) == 1048576
    for n, trace in [(1048577, False), (65536, True), (65537, True)]:
        with pytest.raises(ValueError) as caught:
            cube.fialka_encode([0] * n, {**p, "trace": trace})
        assert caught.value.code == "resource_limit"
