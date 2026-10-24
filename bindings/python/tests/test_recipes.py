import json

import cube
import pytest


def definition():
    return {
        "format_version": 1,
        "engine_api": 1,
        "input": {"source": "slot", "name": "payload", "type": "bytes"},
        "output": {"type": "bytes"},
        "steps": [
            {"operation": "encoding.hex.encode", "version": 1},
            {"operation": "encoding.hex.decode", "version": 1},
        ],
    }


def test_native_recipe_contracts_and_exact_binary_results():
    document = json.dumps(definition())
    info = cube.validate_recipe(document)
    assert info["plan"]["engine_api"] == 1
    assert info["input_slot"] == "payload"
    result = cube.execute_recipe(document, {"payload": b"\x00\xff"}, capture=[0])
    assert result["output"] == b"\x00\xff"
    assert result["intermediates"] == {0: "00ff"}
    assert cube.check_recipe(document, {"payload": b"\x00\xff"}, b"\x00\xff")["passed"]
    with pytest.raises(ValueError, match="exact typed output"):
        cube.check_recipe(document, {"payload": b"\x00\xff"}, b"\xff\x00")
    for bindings in (
        {},
        {"other": b""},
        {"payload": "00ff"},
        {"payload": b"", "extra": b""},
    ):
        with pytest.raises(ValueError):
            cube.execute_recipe(document, bindings)
    token = cube.CancellationToken()
    token.cancel()
    with pytest.raises(ValueError) as error:
        cube.execute_recipe(document, {"payload": b""}, cancellation=token)
    assert error.value.code == "cancelled"


def test_native_recipe_validation_is_strict_and_precedes_execution():
    for field in ("format_version", "engine_api"):
        invalid = definition()
        invalid[field] = 2
        with pytest.raises(ValueError) as error:
            cube.validate_recipe(json.dumps(invalid))
        assert error.value.parameter == field
    invalid = definition()
    invalid["steps"][1]["version"] = 99
    with pytest.raises(ValueError) as error:
        cube.execute_recipe(json.dumps(invalid), {"payload": b"\xff"})
    assert error.value.step == 1
    for document in ('{"x":1,"x":2}', "[" * 18 + "0" + "]" * 18):
        with pytest.raises(ValueError):
            cube.validate_recipe(document)
    invalid = definition()
    invalid["shell"] = "echo unsafe"
    with pytest.raises(ValueError):
        cube.validate_recipe(json.dumps(invalid))


def test_inline_identity_and_structured_values_have_no_implicit_bindings():
    for value, expected in (
        ({"type": "bytes", "base64": "AP8="}, b"\x00\xff"),
        ({"type": "text", "data": "$HOME/secret\n🧊"}, "$HOME/secret\n🧊"),
        (
            {"type": "byte_range", "offset": 1, "length": 2},
            {"type": "byte_range", "offset": 1, "length": 2},
        ),
    ):
        inline = definition()
        inline["input"] = {"source": "inline", "value": value}
        inline["output"] = {"type": value["type"]}
        inline["steps"] = []
        document = json.dumps(inline)
        assert cube.execute_recipe(document, {})["output"] == expected
        assert cube.check_recipe(document, {}, expected)["passed"]
        assert "AP8=" not in json.dumps(cube.validate_recipe(document))
        with pytest.raises(ValueError):
            cube.execute_recipe(document, {"payload": b""})
