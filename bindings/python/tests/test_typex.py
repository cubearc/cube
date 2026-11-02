"""Installed native Mk22 contact symbols and independently published checks."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/typex_vectors.json"
    ).read_text()
)


def test_native_typex_source_answers_and_reversed_core():
    for v in VECTORS:
        p = v["settings"]
        result = cube.typex_encode(v["text"], p)
        assert result["text"] == v["expected"]
        assert result["final_state"] == v["expected_final_state"]
        assert cube.typex_decode(result["text"], p)["text"] == v["text"]
        first = cube.typex_encode(v["text"][:9], p)
        second = cube.typex_encode(
            v["text"][9:], {**p, "initial_state": first["final_state"]}
        )
        assert first["text"] + second["text"] == v["expected"]
        assert second["final_state"] == result["final_state"]
        assert cube.typex_encode("", p)["final_state"] == p["initial_state"]
        with pytest.raises(ValueError):
            cube.typex_encode("a", p)
        with pytest.raises(ValueError):
            cube.typex_encode("A", {**p, "reflector": "ABCDEFGHIJKLMNOPQRSTUVWXYZ"})


def test_native_typex_input_and_trace_limits():
    p = {**VECTORS[0]["settings"], "trace": False}
    assert len(cube.typex_encode("A" * 1048576, p)["text"]) == 1048576
    for text, settings in [("A" * 1048577, p), ("A" * 65537, {**p, "trace": True})]:
        with pytest.raises(ValueError) as caught:
            cube.typex_encode(text, settings)
        assert caught.value.code == "resource_limit"
