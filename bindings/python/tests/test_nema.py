"""Native Model45 source answer and exact lower-row window continuation."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/nema_vectors.json"
    ).read_text()
)


def test_native_nema_complete_source_answer_and_state():
    p = VECTOR["settings"]
    result = cube.nema_encode(VECTOR["text"], p)
    assert result["text"] == VECTOR["expected"]
    assert result["final_state"] == VECTOR["expected_final_state"]
    assert cube.nema_decode(result["text"], p)["text"] == VECTOR["text"]
    first = cube.nema_encode(VECTOR["text"][:9], p)
    second = cube.nema_encode(
        VECTOR["text"][9:], {**p, "initial_state": first["final_state"]}
    )
    assert first["text"] + second["text"] == result["text"]
    assert second["final_state"] == result["final_state"]
    training = {
        **p,
        "profile": "nema_model45_training_v1",
        "drive_wheels": [16, 19, 20, 21],
    }
    encoded = cube.nema_encode("ABCDEFGHIJKLMNOPQRSTUVWXYZ", training)
    assert (
        cube.nema_decode(encoded["text"], training)["text"]
        == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    )
    assert cube.nema_encode("", training)["final_state"] == training["initial_state"]
    for text in ["a", "A A", "秘密"]:
        with pytest.raises(ValueError):
            cube.nema_encode(text, p)
    with pytest.raises(ValueError):
        cube.nema_encode("A", {**training, "rotors": ["A", "B", "C", "F"]})


def test_native_nema_input_and_trace_limits():
    p = {**VECTOR["settings"], "trace": False}
    assert len(cube.nema_encode("A" * 1048576, p)["text"]) == 1048576
    for text, settings in [("A" * 1048577, p), ("A" * 65537, {**p, "trace": True})]:
        with pytest.raises(ValueError) as caught:
            cube.nema_encode(text, settings)
        assert caught.value.code == "resource_limit"
