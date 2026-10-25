import cube
import pytest


def test_factors_scalars_layout_and_explicit_color_rendering():
    assert [
        (p["width"], p["height"]) for p in cube.grid_factors("ABCD")["candidates"]
    ] == [(1, 4), (2, 2), (4, 1)]
    g = cube.text_grid("a e\u0301🧊", 2, drop_whitespace=True)
    assert g["rows"] == [["a", "e"], ["\u0301", "🧊"]]
    assert g["removed_scalars"] == 1
    assert cube.text_grid("ABCD", 2, traversal="column")["rows"] == [
        ["A", "C"],
        ["B", "D"],
    ]
    g = cube.text_grid("01", 2, colors={"0": [0, 0, 0, 255], "1": [255] * 4})
    assert cube.Image.decode(g["png"]).samples == bytes(
        [0, 0, 0, 255, 255, 255, 255, 255]
    )
    for text, width in [("", 1), ("ABCD", 3), ("\t", 0)]:
        with pytest.raises(ValueError):
            cube.text_grid(text, width)
    with pytest.raises(ValueError):
        cube.text_grid("01", 2, colors={"0": [0] * 4})
