import math
import random

import cube
import pytest


def test_exact_modular_oracles_and_full_width_bezout():
    rng = random.Random(180128)
    maximum = 2**64 - 1
    pairs = [(0, 0), (0, maximum), (maximum, 0), (maximum, maximum - 1)]
    pairs += [(12200160415121876738, 7540113804746346429)]
    pairs += [(rng.getrandbits(64), rng.getrandbits(64)) for _ in range(200)]
    for a, b in pairs:
        expected = math.gcd(a, b)
        assert cube.gcd(a, b) == expected
        result = cube.extended_gcd(a, b)
        assert result == cube.extended_gcd(a, b)
        assert all(type(value) is int for value in result.values())
        assert result["gcd"] == expected
        assert a * result["x"] + b * result["y"] == expected
        modulus = max(b, 2)
        assert cube.mod_pow(a, maximum, modulus) == pow(a, maximum, modulus)
        if math.gcd(a, modulus) == 1:
            assert cube.mod_inverse(a, modulus) == pow(a, -1, modulus)
        else:
            with pytest.raises(ValueError) as exc:
                cube.mod_inverse(a, modulus)
            assert exc.value.code == "invalid_input"
    assert cube.extended_gcd(240, 46) == {"gcd": 2, "x": -9, "y": 47}
    assert cube.extended_gcd(0, 0) == {"gcd": 0, "x": 0, "y": 0}
    assert cube.mod_pow(4, 13, 497) == 445
    assert cube.mod_pow(0, 0, 1) == 0
    assert cube.mod_pow(0, 0, 7) == 1


@pytest.mark.parametrize("value", [-1, 2**64, True, 1.0, "1", None])
def test_modular_integer_transport_rejects_coercion_with_context(value):
    calls = [
        ("number.gcd", "a", lambda: cube.gcd(value, 1)),
        ("number.extended_gcd", "b", lambda: cube.extended_gcd(1, value)),
        ("number.mod_inverse", "a", lambda: cube.mod_inverse(value, 3)),
        ("number.mod_pow", "exponent", lambda: cube.mod_pow(2, value, 3)),
        ("number.mod_pow", "modulus", lambda: cube.mod_pow(2, 3, value)),
    ]
    for operation, parameter, call in calls:
        with pytest.raises(ValueError) as exc:
            call()
        assert exc.value.operation == operation
        assert exc.value.parameter == parameter
        assert exc.value.code in {"invalid_parameter", "resource_limit"}


def test_modulus_errors_and_direct_discovery():
    for call in [
        lambda: cube.mod_inverse(1, 0),
        lambda: cube.mod_inverse(1, 1),
        lambda: cube.mod_pow(1, 2, 0),
    ]:
        with pytest.raises(ValueError) as exc:
            call()
        assert exc.value.code == "invalid_parameter"
        assert exc.value.parameter == "modulus"
    for name in ["gcd", "extended_gcd", "mod_inverse", "mod_pow"]:
        assert cube.describe_operation(f"number.{name}")["pipeline"] is None
