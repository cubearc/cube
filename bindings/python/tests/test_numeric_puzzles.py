"""Actual installed native transports for all eight bounded numeric operations."""

import json
from pathlib import Path

import cube
import pytest

FAMILIES = ("prime_number", "prime_multiplication", "modulo", "triliteral")
P = {
    "profile": "prime_multiset_v1",
    "index": {"alphabet": "AB", "first_index": 1, "max_prime": 3},
    "max_factor_checks": 13,
}


def test_native_numeric_known_answers_and_discovery():
    rows = json.loads(
        (
            Path(__file__).parents[3]
            / "crates/cube-core/src/operations/contracts/conformance.json"
        ).read_text()
    )
    cases = [r for r in rows if r["operation"].split(".")[1] in FAMILIES]
    assert len(cases) == 8
    for r in cases:
        _, family, direction = r["operation"].split(".")
        assert (
            getattr(cube, f"{family}_{direction}")(
                **r["inputs"], parameters=r["parameters"]
            )
            == r["outputs"]
        )


def test_products_keep_counts_and_exact_decimal_values():
    encoded = cube.prime_multiplication_encode(["BA", "AB", ""], P)
    assert encoded["groups"][0] == encoded["groups"][1]
    assert encoded["groups"][2]["product"] == "1"
    result = cube.prime_multiplication_decode(["6", "6", "1", "8"], P)
    assert result["factor_checks"] == 13
    assert result["groups"][0]["counts"] == [
        {"symbol": "A", "count": 1},
        {"symbol": "B", "count": 1},
    ]
    with pytest.raises(ValueError) as failure:
        cube.prime_multiplication_decode(
            ["6", "6", "1", "8"], dict(P, max_factor_checks=12)
        )
    assert failure.value.code == "resource_limit"
    assert failure.value.operation == "cipher.prime_multiplication.decode"


@pytest.mark.parametrize(
    "bad", ["0", "01", "5", "6.0", "340282366920938463463374607431768211456"]
)
def test_products_reject_unassigned_factors_and_noncanonical_values(bad):
    with pytest.raises(ValueError) as failure:
        cube.prime_multiplication_decode([bad], P)
    assert failure.value.code == "invalid_input"


def test_numeric_typed_transport_uses_input_error_context():
    with pytest.raises(ValueError) as failure:
        cube.prime_multiplication_decode([6], P)
    assert failure.value.code == "invalid_input"
    assert failure.value.parameter == "products"
    assert failure.value.operation == "cipher.prime_multiplication.decode"
    p = {
        "profile": "indexed_primes_v1",
        "index": {"alphabet": "A", "first_index": 1, "max_prime": 2},
    }
    with pytest.raises(ValueError) as failure:
        cube.prime_number_decode(["02"], p)
    assert failure.value.code == "invalid_input"
    assert failure.value.operation == "cipher.prime_number.decode"
