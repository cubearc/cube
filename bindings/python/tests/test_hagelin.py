"""Actual native Hagelin source answer, continuation and profile validation."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/hagelin_vectors.json"
    ).read_text()
)


def test_native_hagelin_manual_answer_and_continuation():
    p = VECTOR["settings"]
    result = cube.hagelin_encode(VECTOR["text"], p)
    assert result["text"] == VECTOR["expected"]
    assert cube.hagelin_decode(result["text"], p)["text"] == VECTOR["text"]
    first = cube.hagelin_encode(VECTOR["text"][:9], p)
    second = cube.hagelin_encode(
        VECTOR["text"][9:], {**p, "initial_state": first["final_state"]}
    )
    assert first["text"] + second["text"] == result["text"]
    assert second["final_state"] == result["final_state"]
    for text in ["a", "A A", "秘密"]:
        with pytest.raises(ValueError):
            cube.hagelin_encode(text, p)
    with pytest.raises(ValueError):
        cube.hagelin_encode("A", {**p, "slide": 0})


def test_every_native_hagelin_profile_executes_its_distinct_advancement():
    cases = [
        ("c36_fixed25_bouch_v1", [25, 23, 21, 19, 17], [3] * 5),
        ("c38_standard27_v1", [26, 25, 23, 21, 19, 17], [3] * 6),
        ("m209_standard_v1", [26, 25, 23, 21, 19, 17], [3] * 6),
        ("c52_c2_27_v1", [26, 25, 46, 42, 38, 34], [3, 3, 6, 6, 6, 6]),
        ("cx52_m27_late_v1", [47] * 6, [3, 0, 0, 0, 0, 0]),
    ]
    for profile, lengths, expected_state in cases:
        p = {
            "profile": profile,
            "pins": ["0" * n for n in lengths],
            "initial_state": [0] * len(lengths),
        }
        if not profile.startswith("c36_"):
            p["lugs"] = [[False] * 6 for _ in range(27)]
        if profile != "m209_standard_v1":
            p["slide"] = 0
        expected = "ZYX" if profile == "m209_standard_v1" else "AZY"
        result = cube.hagelin_encode("ABC", p)
        assert result["text"] == expected
        assert result["final_state"] == expected_state
        assert cube.hagelin_decode(expected, p)["text"] == "ABC"
        assert cube.hagelin_encode("", p)["final_state"] == p["initial_state"]
        assert cube.hagelin_encode("A" * 1048576, p)["text"]
        with pytest.raises(ValueError) as caught:
            cube.hagelin_encode("A" * 1048577, p)
        assert caught.value.code == "resource_limit"
