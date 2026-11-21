"""Exercise the installed SDK against exact wire fixtures and native errors."""

import base64
import json
from pathlib import Path

import cube
import pytest

CATALOG = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)


def cases():
    rows = json.loads(CATALOG.read_text(encoding="utf-8"))
    return [
        row
        for row in rows
        if row["operation"].split(".")[1]
        in {
            "packbits",
            "lz77",
            "lz78",
            "lz4",
            "bzip2",
            "bwt",
            "mtf",
            "huffman",
            "lzw",
            "entropy",
            "bmp",
            "webp",
            "tiff",
            "executable",
            "mpeg_audio",
            "ogg",
            "zip",
        }
        and (
            row["operation"].startswith("binary.")
            or row["operation"].startswith("file.")
        )
    ]


@pytest.mark.parametrize("row", cases(), ids=lambda row: row["operation"])
def test_installed_native_format_fixture(row):
    name = row["operation"]
    _, profile, action = name.split(".")
    native = {
        key: base64.b64decode(value["base64"])
        if isinstance(value, dict) and "base64" in value
        else value
        for key, value in row["inputs"].items()
    }
    expected = {
        key: base64.b64decode(value["base64"])
        if isinstance(value, dict) and "base64" in value
        else value
        for key, value in row["outputs"].items()
    }
    options = row["parameters"]
    if profile == "packbits":
        native["data"] = native.pop("input")
    if profile == "zip":
        assert cube.validate_zip(native["input"], **options) == expected["output"]
    elif action == "inspect":
        assert getattr(cube, f"inspect_{profile}")(native["data"]) == expected["report"]
    else:
        method = getattr(cube, f"{profile}_{action}")
        if profile == "huffman" and action == "construct":
            assert method(native["frequencies"]) == expected["codes"]
        elif profile == "huffman" and action == "encode":
            assert method(native["data"], native["frequencies"]) == (
                expected["output"],
                expected["bit_count"],
                expected["codes"],
            )
        elif profile == "huffman":
            assert (
                method(native["data"], native["lengths"], **options)
                == expected["output"]
            )
        elif profile in {"lz77", "lz78"} and action == "encode":
            assert method(native["data"], **options) == expected["tokens"]
        elif profile in {"lz77", "lz78"}:
            assert method(native["tokens"], **options) == expected["output"]
        elif profile == "bwt" and action == "encode":
            assert method(native["data"]) == (
                expected["output"],
                expected["primary_index"],
            )
        elif profile in {"lz4", "lzw"}:
            assert method(native["data"], options) == expected["output"]
        elif profile == "entropy":
            assert (
                method(native["data"], native["frequencies"], options)
                == expected["output"]
            )
        else:
            assert method(native["data"], **options) == expected["output"]


def test_native_error_codes_and_lossless_bytes():
    with pytest.raises(ValueError) as error:
        cube.packbits_decode(bytes([3, 1, 2]))
    assert error.value.code == "invalid_input"
    assert error.value.operation == "binary.packbits.decode"
    with pytest.raises(ValueError) as error:
        cube.packbits_encode(bytes((1 << 20) + 1))
    assert error.value.code == "resource_limit"
    payload = bytes(range(256))
    assert (
        cube.lzw_decode(
            cube.lzw_encode(payload, {"profile": "tiff"}), {"profile": "tiff"}
        )
        == payload
    )
    assert cube.bzip2_decode(cube.bzip2_encode(payload)) == payload
