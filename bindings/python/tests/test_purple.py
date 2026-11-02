"""Native PURPLE independent reference, continuation and exact bounds."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/purple_vectors.json"
    ).read_text()
)


def test_native_purple_source_answer_and_continuation():
    v = VECTOR
    p = v["settings"]
    r = cube.purple_encode(v["text"], p)
    assert r["text"] == v["expected"]
    assert r["final_state"] == v["expected_final_state"]
    assert cube.purple_decode(r["text"], p)["text"] == v["text"]
    first = cube.purple_encode(v["text"][:9], p)
    second = cube.purple_encode(
        v["text"][9:], {**p, "initial_state": first["final_state"]}
    )
    assert first["text"] + second["text"] == r["text"]
    assert second["final_state"] == r["final_state"]
    assert cube.purple_encode("", p)["final_state"] == p["initial_state"]
    with pytest.raises(ValueError):
        cube.purple_encode("a", p)
    with pytest.raises(ValueError):
        cube.purple_encode("A", {**p, "profile": "PRIVATE_UNKNOWN_PROFILE"})


def test_native_purple_input_trace_and_report_limits():
    p = {**VECTOR["settings"], "trace": False}
    assert len(cube.purple_encode("A" * 1048576, p)["text"]) == 1048576
    for n, trace in [(1048577, False), (65536, True), (65537, True)]:
        with pytest.raises(ValueError) as caught:
            cube.purple_encode("A" * n, {**p, "trace": trace})
        assert caught.value.code == "resource_limit"
