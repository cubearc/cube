import cube
import pytest


def repeated_sum(n):
    n = abs(n)
    while n >= 10:
        n = sum(map(int, str(n)))
    return n


def test_decimal_root_uses_exact_signed_magnitude():
    values = list(range(-500, 501)) + [2**127, -(2**127), 2**128 - 1, -(2**128 - 1)]
    for value in values:
        assert cube.digital_root(value) == repeated_sum(value)
        assert cube.digital_root(value, radix=10) == repeated_sum(value)
    assert cube.digital_root(38) == 2
    assert cube.digital_root(999) == 9
    assert cube.digital_root(0) == 0
    assert type(cube.digital_root(38)) is int
    operation = cube.describe_operation("number.digital_root")
    assert operation["reversible"] is False
    assert operation["pipeline"] is None


@pytest.mark.parametrize(
    "value",
    [
        True,
        False,
        38.0,
        "38",
        None,
        2**128,
        -(2**128),
        pytest.param(1 << 16384, id="oversized_positive"),
        pytest.param(-(1 << 16384), id="oversized_negative"),
    ],
)
def test_invalid_transport_preserves_context(value):
    with pytest.raises(ValueError) as exc:
        cube.digital_root(value)
    assert exc.value.operation == "number.digital_root"
    assert exc.value.parameter == "value"
    assert exc.value.code in {"invalid_parameter", "resource_limit"}


def test_nondecimal_radix_is_explicitly_unsupported():
    for radix in [0, 1, 2, 9, 11, 36]:
        with pytest.raises(ValueError) as exc:
            cube.digital_root(38, radix=radix)
        assert exc.value.code == "invalid_parameter"
        assert exc.value.parameter == "radix"
