import cube
import pytest


def test_native_fields_keep_types_and_exact_integer_pipeline_answers():
    assert cube.parse_json('{"a/b": [null, 7]}') == {"a/b": [None, 7]}
    assert cube.select_field('{"a/b": [null, 7]}', "/a~1b/1") == 7
    assert cube.select_field('{"a/b": [null, 7]}', "/a~1b/0") is None
    assert cube.json_text('"é\\u0000世界\\r\\n"') == "é\0世界\r\n"
    assert cube.add_integers(cube.maximum([2, 3]), cube.parse_integer("7")) == 10
    image = cube.Image.from_samples(2, 3, "rgb", bytes(18))
    assert cube.image_dimensions(image) == {"dimensions": [2, 3]}


def test_native_composition_refuses_coercion_missing_fields_and_overflow():
    for call in [
        lambda: cube.parse_json('{"x":1,"x":2}'),
        lambda: cube.select_field('{"x":null}', "/missing"),
        lambda: cube.json_text("7"),
        lambda: cube.maximum([]),
        lambda: cube.maximum([1, True]),
        lambda: cube.maximum([1, 2.0]),
        lambda: cube.maximum([1, "2"]),
        lambda: cube.add_integers(True, 1),
        lambda: cube.add_integers(9007199254740991, 1),
        lambda: cube.parse_integer("1e3"),
        lambda: cube.parse_integer(" 7"),
    ]:
        with pytest.raises(ValueError):
            call()
