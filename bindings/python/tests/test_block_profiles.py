"""Independent known answers through the real installed native SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/block_profiles/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("vector", VECTORS)
def test_independent_answers_empty_and_multiple_blocks(vector):
    plain = bytes.fromhex(vector["plaintext_hex"])
    encrypted = bytes.fromhex(vector["ciphertext_hex"])
    key = bytes.fromhex(vector["key_hex"])
    options = (
        {"effective_key_bits": vector["effective_key_bits"]}
        if "effective_key_bits" in vector
        else {}
    )
    if "tweak_hex" in vector:
        options["tweak"] = bytes.fromhex(vector["tweak_hex"])
    for function, data, expected in [
        (cube.block_encrypt_blocks, plain, encrypted),
        (cube.block_decrypt_blocks, encrypted, plain),
    ]:
        assert function(data, key, vector["profile"], **options) == expected
        assert function(data * 3, key, vector["profile"], **options) == expected * 3
        assert function(b"", key, vector["profile"], **options) == b""


def test_block_errors_do_not_echo_private_material():
    for function in (cube.block_encrypt_blocks, cube.block_decrypt_blocks):
        for data, key, profile, options, code in [
            (b"", b"SECRET", "des_fips46_v1", {}, "invalid_parameter"),
            (b"SECRET", bytes(16), "idea_v1", {}, "invalid_input"),
            (b"", bytes(16), "unknown", {}, "invalid_parameter"),
            (b"", bytes(1), "rc2_rfc2268_v1", {}, "invalid_parameter"),
            (
                b"",
                bytes(1),
                "rc2_rfc2268_v1",
                {"effective_key_bits": 0},
                "invalid_parameter",
            ),
            (
                b"",
                bytes(1),
                "rc2_rfc2268_v1",
                {"effective_key_bits": 1025},
                "invalid_parameter",
            ),
            (
                b"",
                bytes(1),
                "rc2_rfc2268_v1",
                {"effective_key_bits": True},
                "invalid_parameter",
            ),
            (
                b"",
                bytes(16),
                "idea_v1",
                {"effective_key_bits": 64},
                "invalid_parameter",
            ),
            (bytes(1048577), bytes(16), "idea_v1", {}, "resource_limit"),
        ]:
            with pytest.raises(ValueError) as caught:
                function(data, key, profile, **options)
            assert caught.value.code == code
            assert "SECRET" not in str(caught.value)


def test_exact_native_byte_limit():
    data = bytes(1048576)
    encrypted = cube.block_encrypt_blocks(data, bytes(16), "idea_v1")
    assert cube.block_decrypt_blocks(encrypted, bytes(16), "idea_v1") == data


def test_craft_requires_exact_tweak_and_other_profiles_reject_it():
    for function in (cube.block_encrypt_blocks, cube.block_decrypt_blocks):
        for profile, options in [
            ("craft_2020_v1", {}),
            ("craft_2020_v1", {"tweak": b"SECRET"}),
            ("craft_2020_v1", {"tweak": bytes(9)}),
            ("idea_v1", {"tweak": bytes(8)}),
        ]:
            with pytest.raises(ValueError) as caught:
                function(b"", bytes(16), profile, **options)
            assert caught.value.code == "invalid_parameter"
            assert "SECRET" not in str(caught.value)
