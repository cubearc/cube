import json

import cube
import pytest

WIF = "L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd"
HEX = "ff778740f88ddcf102aeb81daee289c044c4a4571c4b6f287400f4b8e0b843f8"


def test_safe_native_wif_and_scalar_metadata():
    report = cube.validate_wif(WIF, expected_network="mainnet")
    assert report["valid"] and report["compressed"] and report["scalar_validated"]
    assert report["version"] == 128 and report["network_family"] == "mainnet"
    assert WIF not in json.dumps(report) and HEX not in json.dumps(report)
    assert cube.validate_private_scalar(bytes.fromhex(HEX))["valid"]
    assert cube.validate_private_scalar(HEX, format="hex")["byte_width"] == 32
    for bad in [WIF + "\n", WIF[:-1], "0OIl", "1" * 129]:
        with pytest.raises(ValueError) as error:
            cube.validate_wif(bad)
        assert WIF not in str(error.value) and HEX not in str(error.value)
    with pytest.raises(ValueError):
        cube.validate_wif(WIF, expected_network="testnet")
    for invalid in [bytes(32), bytes([255]) * 32, bytes(31), bytes(33)]:
        with pytest.raises(ValueError):
            cube.validate_private_scalar(invalid)
    assert cube.describe_operation("blockchain.wif.validate")["pipeline"] is None
    assert cube.describe_operation("blockchain.scalar.validate")["pipeline"] is None

    for call in [
        lambda: cube.validate_wif(WIF + "\ud800"),
        lambda: cube.validate_private_scalar(HEX + "\ud800", format="hex"),
    ]:
        with pytest.raises(ValueError) as error:
            call()
        assert WIF not in repr(error.value.args)
        assert HEX not in repr(error.value.args)
        assert not hasattr(error.value, "object")
