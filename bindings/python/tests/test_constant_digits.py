import cube
import pytest


def test_native_constant_digits_limits_selection_and_override():
    for name, integer, prefix in [
        ("pi", "3", "1415926535897932"),
        ("e", "2", "7182818284590452"),
        ("phi", "1", "6180339887498948"),
        ("sqrt2", "1", "4142135623730950"),
    ]:
        result = cube.constant_digits(name)
        assert result["digits"] == prefix
        assert result["integer_part"] == integer
        assert result["max_digits"] == 256
        assert result["fractional_zero_based"] is True
        assert len(result["prefix_sha256"]) == 64
        assert cube.constant_digits(name, indices=[2, 0, 2])["digits"] == (
            prefix[2] + prefix[0] + prefix[2]
        )
        assert cube.constant_digits(name, indices=[])["digits"] == ""
        with pytest.raises(ValueError, match="max_digits"):
            cube.constant_digits(name, start=256, count=1)
        assert (
            len(
                cube.constant_digits(name, start=256, count=1, max_digits=512)["digits"]
            )
            == 1
        )
    for kwargs in [
        {"max_digits": 4097},
        {"max_digits": 0},
        {"start": 0, "indices": []},
        {"count": 0, "indices": []},
        {"indices": [256]},
        {"indices": [0] * 1025},
    ]:
        with pytest.raises(ValueError):
            cube.constant_digits("pi", **kwargs)
    for kwargs in [{"count": 1.5}, {"max_digits": float("nan")}, {"indices": [1.5]}]:
        with pytest.raises((TypeError, OverflowError)):
            cube.constant_digits("pi", **kwargs)
    with pytest.raises((OverflowError, ValueError)):
        cube.constant_digits("pi", indices=[-1])
    with pytest.raises(ValueError):
        cube.constant_digits("PI")
