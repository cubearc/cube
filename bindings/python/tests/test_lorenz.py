"""Executable native five-bit profiles with explicit contact phase and histories."""

import json
from pathlib import Path

import cube
import pytest

VECTOR = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/mechanical/lorenz_vectors.json"
    ).read_text()
)


def test_lorenz_native_source_pins_and_continuation():
    settings = VECTOR["settings"]
    encoded = cube.lorenz_encode(VECTOR["codes"], settings)
    assert encoded["codes"] == VECTOR["expected"]
    assert cube.lorenz_decode(encoded["codes"], settings)["codes"] == VECTOR["codes"]
    for profile in ["lorenz_sz40_v1", "lorenz_sz42a_v1", "lorenz_sz42b_v1"]:
        for kt in [False, True]:
            if profile == "lorenz_sz40_v1" and kt:
                continue
            p = {**settings, "profile": profile, "plaintext_limitation": kt}
            whole = cube.lorenz_encode(list(range(32)), p)
            decoded = cube.lorenz_decode(whole["codes"], p)
            assert decoded["codes"] == list(range(32))
            assert decoded["final_state"] == whole["final_state"]
            first = cube.lorenz_encode(list(range(9)), p)
            second = cube.lorenz_encode(
                list(range(9, 32)), {**p, "initial_state": first["final_state"]}
            )
            assert first["codes"] + second["codes"] == whole["codes"]
            assert second["final_state"] == whole["final_state"]
    for codes in [[32], [-1], [True], ["SECRET"]]:
        with pytest.raises(ValueError) as caught:
            cube.lorenz_encode(codes, settings)
        assert caught.value.code == "invalid_input"
        assert "SECRET" not in str(caught.value)
    with pytest.raises(ValueError) as caught:
        cube.lorenz_encode([], {**settings, "plaintext_limitation": True})
    assert caught.value.code == "invalid_parameter"


def test_lorenz_native_transport_and_core_limits_are_independent():
    p = {**VECTOR["settings"], "trace": False}
    assert len(cube.lorenz_encode([0] * 1048576, p)["codes"]) == 1048576
    with pytest.raises(ValueError) as caught:
        cube.lorenz_encode([0] * 1048577, p)
    assert caught.value.code == "resource_limit"
    with pytest.raises(ValueError) as caught:
        cube.lorenz_encode([0] * 65537, {**p, "trace": True})
    assert caught.value.code == "resource_limit"
