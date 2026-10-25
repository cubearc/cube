from collections.abc import Iterator

import cube
import pytest


def test_native_incremental_combinations_preserve_order_duplicates_and_controls():
    values = cube.string_combinations([["a", "b"], ["1", "2"]], separator="-")
    assert isinstance(values, Iterator)
    assert values.plan()["total_results"] == 4
    assert next(values) == "a-1"
    assert len(values) == 3
    assert list(values) == ["a-2", "b-1", "b-2"]
    assert list(cube.string_combinations([])) == [""]
    assert list(cube.string_combinations([["a"], []])) == []
    assert list(cube.string_combinations([["", ""], ["🧊\n"]])) == ["🧊\n", "🧊\n"]
    assert (
        cube.hex_encode(next(cube.string_combinations([["A"], ["B"]])).encode())
        == "4142"
    )
    with pytest.raises(ValueError):
        cube.string_combinations([["a", "b"], ["1", "2"]], max_results=3)
    with pytest.raises(ValueError):
        cube.string_combinations([["x" * 65537]])
    with pytest.raises(ValueError):
        cube.string_combinations([[""] * 256] * 16, max_results=10000)
