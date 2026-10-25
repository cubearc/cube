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


def test_native_bitcoin_public_derivation_and_digest_composition():
    key = bytes(31) + b"\x01"
    result = cube.derive_bitcoin_address(key, network="mainnet", compressed=True)
    assert result["address"] == "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"
    assert isinstance(result["public_key"], bytes) and len(result["public_key"]) == 33
    assert result["hash160"].hex() == "751e76e8199196d454941c45d1b3a323f1433bd6"
    assert (
        cube.derive_bitcoin_address(key, network="mainnet", compressed=False)["address"]
        == "1EHNa6Q4Jz2uvNExL497mE43ikXhwF6kZm"
    )
    wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
    assert (
        cube.derive_bitcoin_address(
            wif, format="wif", network="mainnet", compressed=True
        )["address"]
        == result["address"]
    )
    for kwargs in [
        {"network": "mainnet", "compressed": False},
        {"network": "testnet", "compressed": True},
    ]:
        with pytest.raises(ValueError) as error:
            cube.derive_bitcoin_address(wif, format="wif", **kwargs)
        assert wif not in str(error.value)
    assert cube.ripemd160(b"") == "9c1185a5c5e9fc54612808977ee8f548b2258d31"
    assert cube.execute_pipeline([{"operation": "hash.ripemd160", "version": 1}], b"")[
        "output"
    ] == cube.ripemd160(b"")
    assert cube.describe_operation("blockchain.bitcoin.derive")["pipeline"] is None


def test_explicit_native_private_export_is_separate_from_safe_validation():
    key = bytes(31) + b"\x01"
    wif = cube.export_private_key(key, output="wif", network="mainnet", compressed=True)
    assert wif == "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
    assert cube.export_private_key(wif, format="wif", output="raw") == key
    assert cube.export_private_key(wif, format="wif", output="hex") == key.hex()
    assert "payload" not in cube.validate_wif(wif)
    for kwargs in [
        {"output": "wif"},
        {"output": "wif", "network": "mainnet"},
        {"output": "hex", "network": "mainnet"},
    ]:
        with pytest.raises(ValueError):
            cube.export_private_key(key, **kwargs)
    with pytest.raises(ValueError) as error:
        cube.export_private_key(
            wif, format="wif", output="wif", network="mainnet", compressed=False
        )
    assert wif not in str(error.value)
    assert cube.describe_operation("blockchain.private.export")["pipeline"] is None


def test_native_ethereum_and_reusable_keccak():
    key = bytes(31) + b"\x01"
    result = cube.derive_ethereum_address(key)
    assert result["address"] == "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf"
    assert (
        isinstance(result["address_bytes"], bytes)
        and len(result["address_bytes"]) == 20
    )
    assert (
        isinstance(result["public_key_xy"], bytes)
        and len(result["public_key_xy"]) == 64
    )
    assert result["address_hex"] == result["address_bytes"].hex()
    assert (
        cube.derive_ethereum_address(key.hex(), format="hex", checksum="lowercase")[
            "address"
        ]
        == result["address"].lower()
    )
    assert (
        cube.keccak256(b"")
        == "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
    )
    assert cube.execute_pipeline(
        [{"operation": "hash.keccak256", "version": 1}], b"abc"
    )["output"] == cube.keccak256(b"abc")
    assert cube.describe_operation("blockchain.ethereum.derive")["pipeline"] is None
    for bad in ["g" * 64, HEX + "\ud800"]:
        with pytest.raises(ValueError) as error:
            cube.derive_ethereum_address(bad, format="hex")
        assert bad not in repr(error.value.args)
        assert not hasattr(error.value, "object")
    with pytest.raises(ValueError):
        cube.derive_ethereum_address(key, checksum="sha3")
