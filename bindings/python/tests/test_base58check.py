import cube
import pytest


def test_native_base58check_retains_typed_payloads_and_pipeline_availability():
    data = bytes.fromhex("a31c06bd463e3923bc1aadbde48b16976c080717")
    text = "1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo"
    assert cube.base58check_encode(data, version=0) == text
    decoded = cube.base58check_decode(text)
    assert decoded["version"] == 0
    assert decoded["payload"] == data
    assert decoded["checksum_verified"] is True
    for payload in [b"", bytes(32), bytes(range(256)), bytes([255]) * 1024]:
        assert (
            cube.base58check_decode(cube.base58check_encode(payload, version=239))[
                "payload"
            ]
            == payload
        )
    for invalid in ["", "0OIl", text + "\n", text[:-1] + "1", "1" * 1422]:
        with pytest.raises(ValueError):
            cube.base58check_decode(invalid)
    with pytest.raises(ValueError):
        cube.base58check_encode(bytes(1025), version=0)
    assert cube.describe_operation("encoding.base58check.encode")["pipeline"] == {
        "version": 1
    }
    assert cube.describe_operation("encoding.base58check.decode")["pipeline"] is None
    assert cube.base58_decode("") == b""

    steps = [
        {
            "operation": "encoding.base58check.encode",
            "version": 1,
            "parameters": {"version": 0},
        }
    ]
    composed = cube.execute_pipeline(steps, data, capture=[0])
    assert composed["output"] == text
    assert composed["intermediates"][0] == text
    with pytest.raises(ValueError):
        cube.execute_pipeline(steps, bytes(1025))
    with pytest.raises(ValueError):
        cube.validate_pipeline(
            [{"operation": "encoding.base58check.decode", "version": 1}], "text"
        )
