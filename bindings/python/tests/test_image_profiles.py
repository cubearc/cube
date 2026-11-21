"""Run every image profile through the installed native extension."""

import base64
import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).resolve().parents[3]
CASES = json.loads(
    (ROOT / "crates/cube-core/src/image/advanced-fixtures.json").read_text(
        encoding="utf-8"
    )
)


def native(value):
    if isinstance(value, dict) and value.keys() == {"base64"}:
        return base64.b64decode(value["base64"])
    if isinstance(value, dict) and "color_model" in value and "samples" in value:
        return cube.Image.from_samples(
            value["width"],
            value["height"],
            value["color_model"],
            native(value["samples"]),
            value["palette"],
            native(value["transparency"]),
        )
    if isinstance(value, list):
        return [native(item) for item in value]
    return value


@pytest.mark.parametrize("operation,case", CASES.items())
def test_direct_image_profiles_use_native_objects(operation, case):
    inputs = {name: native(value) for name, value in case["inputs"].items()}
    actual = cube.execute_image_profile(operation, inputs, case["parameters"])
    expected = case["outputs"]
    assert actual.keys() == expected.keys()
    for name, value in expected.items():
        sample = actual[name]
        if isinstance(sample, cube.Image):
            assert sample.width == value["width"]
            assert sample.height == value["height"]
            assert sample.info()["color_model"] == value["color_model"]
            assert sample.samples == native(value["samples"])
        else:
            assert sample == native(value)


def test_qr_accepts_explicit_segments_and_reports_missing_or_conflicting_input():
    execute = cube.execute_image_profile
    parameters = {"ecc": "L", "version": 1, "mask": 0, "raster": {"scale": 4}}
    result = execute(
        "image.qr.encode",
        {"segments": [{"kind": "byte", "data": [0, 255]}]},
        parameters,
    )
    assert result["modules"] == native(CASES["image.qr.encode"]["outputs"]["modules"])
    assert result["raster"].width == 116
    with pytest.raises(ValueError) as invalid:
        execute("image.qr.encode", {"data": b"", "segments": []}, parameters)
    assert invalid.value.code == "invalid_input"
    assert invalid.value.operation == "image.qr.encode"


@pytest.mark.parametrize(
    "format_,name,text",
    [
        ("data_matrix", "datamatrix", "0123456789"),
        ("aztec", "aztec-full", "Cube\0ÿ"),
        ("pdf417", "pdf417", "This is PDF417"),
    ],
)
def test_read_optical_codes_with_native_image(format_, name, text):
    crop = cube.Image.open(
        ROOT / f"crates/cube-core/tests/fixtures/visual-codes/{name}.png"
    )
    result = cube.execute_image_profile(
        "image.barcode.decode",
        {"crops": [crop]},
        {"format": format_, "orientation": "0"},
    )["report"]["crops"][0]
    assert result["status"] == "decoded"
    assert result["text"] == text
    assert len(result["points"]) == 4


def test_existing_native_exports_share_new_traversals():
    raster = cube.Image.from_samples(2, 2, "gray", b"\x01\x02\x03\x04")
    assert raster.pixels(traversal="hilbert")["data"] == b"\x01\x03\x04\x02"
    assert raster.lsb(traversal="serpentine", incomplete="pad")["data"] == b"\x90"
    grid = cube.text_grid("ABCD", 2, traversal="hilbert")
    assert grid["rows"] == [["A", "D"], ["B", "C"]]


def test_native_source_provenance_survives_profile_invocation(tmp_path):
    path = tmp_path / "view.png"
    cube.Image.from_samples(2, 1, "gray", b"\x00\xff").save_png(path)
    decoded = cube.Image.open(path)
    result = cube.execute_image_profile(
        "image.stereo.separate",
        {"input": decoded},
        {"mode": "side_by_side"},
    )
    assert result["report"]["source"]["representation"] == "native_png8_samples"
    assert result["left"].samples == b"\x00"
    assert result["right"].samples == b"\xff"
