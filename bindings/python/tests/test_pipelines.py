"""Original typed pipeline vectors; validation and execution use the Rust engine."""

import cube
import pytest


def step(operation, parameters=None, version=1):
    return dict(operation=operation, version=version, parameters=parameters or {})


def test_binary_pipeline_capture_and_explicit_utf8():
    steps = [step("encoding.hex.encode"), step("encoding.hex.decode")]
    result = cube.execute_pipeline(steps, b"\x00\xff", capture=[0])
    assert result["output"] == b"\x00\xff"
    assert result["intermediates"] == {0: "00ff"}
    assert cube.execute_pipeline(steps, b"\x00\xff")["intermediates"] == {}
    assert cube.validate_pipeline(steps, "bytes")["output_type"] == "bytes"
    assert len(cube.pipeline_adapters()["adapters"]) == 47
    text = "é 世界🧊\x00\r\n"
    assert cube.utf8_decode(cube.utf8_encode(text)) == text
    assert (
        cube.execute_pipeline(
            [step("text.utf8.decode"), step("text.utf8.encode")], text.encode()
        )["output"]
        == text.encode()
    )
    inert = {"type": "byte_range", "offset": 3, "length": 4}
    assert cube.execute_pipeline([], inert)["output"] == inert


def test_preflight_and_runtime_failures_have_step_context():
    with pytest.raises(ValueError) as error:
        cube.validate_pipeline(
            [step("encoding.hex.decode"), step("file.range.extract")], "text"
        )
    assert error.value.step == 1
    for steps, kind in [
        ([step("cipher.rot13")], "bytes"),
        ([step("cipher.caesar.encode", {"shift": "3"})], "text"),
        ([step("cipher.vigenere.encode", {"key": "bad key"})], "text"),
        ([step("cipher.rot13", version=2)], "text"),
    ]:
        with pytest.raises(ValueError):
            cube.validate_pipeline(steps, kind)
    with pytest.raises(ValueError) as error:
        cube.execute_pipeline(
            [step("encoding.hex.decode"), step("text.utf8.decode")], "zz"
        )
    assert error.value.step == 0
    assert error.value.operation == "encoding.hex.decode"
    with pytest.raises(TypeError):
        cube.execute_pipeline([], 123)
    with pytest.raises(ValueError):
        cube.utf8_decode(b"\xff")


def test_native_limits_cancellation_and_parameter_bytes():
    assert (
        cube.execute_pipeline(
            [step("binary.xor", {"key": {"base64": "/w=="}})], b"\x00\xff"
        )["output"]
        == b"\xff\x00"
    )
    token = cube.CancellationToken()
    assert not token.is_cancelled
    token.cancel()
    assert token.is_cancelled
    with pytest.raises(ValueError) as error:
        cube.execute_pipeline([], b"", cancellation=token)
    assert error.value.code == "cancelled"
    for limits in [
        dict(max_value_bytes=1),
        dict(max_work_bytes=1),
        dict(max_capture_bytes=1),
    ]:
        with pytest.raises(ValueError) as error:
            cube.execute_pipeline(
                [step("encoding.hex.encode")], b"ab", capture=[0], limits=limits
            )
        assert error.value.code == "resource_limit"


def test_forensic_reports_compose_and_keep_native_bytes():
    steps = [step("encoding.hex.decode"), step("file.hex.inspect", {"width": 1})]
    result = cube.execute_pipeline(steps, "00ff", capture=[0, 1])
    assert result["output_type"] == "forensic_report"
    report = result["output"]
    assert report["data"]["rows"][1] == {
        "offset": 1,
        "bytes": b"\xff",
        "hex": "ff",
        "ascii": ".",
    }
    assert result["intermediates"][0] == b"\0\xff"
    assert result["intermediates"][1] == report
    assert cube.execute_pipeline([], report)["output"] == report
    signatures = cube.execute_pipeline([step("file.signature.detect")], b"PK\x03\x04")
    assert signatures["output"]["data"]["matches"][0]["format"] == "zip"
    strings = cube.execute_pipeline(
        [step("file.strings.extract", {"max_results": 1})], b"\0hello\0world\0"
    )["output"]["data"]
    assert strings["strings"][0] == {"offset": 1, "length": 5, "text": "hello"}
    assert strings["complete"] is False
    for bad in [
        [step("file.hex.inspect"), step("hash.sha256")],
        [step("file.hex.inspect", {"width": 0})],
    ]:
        with pytest.raises(ValueError):
            cube.validate_pipeline(bad, "bytes")
    with pytest.raises(ValueError) as error:
        cube.execute_pipeline([step("file.hex.inspect", {"width": 1})], b"\0" * 10000)
    assert error.value.code == "resource_limit"
    with pytest.raises(TypeError):
        cube.execute_pipeline([], {"type": "text", "data": b"text"})
