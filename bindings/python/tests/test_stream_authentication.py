"""Installed native Grain128a ciphertext/tag answers and verification failures."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/stream/reference/authentication/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("vector", VECTORS)
def test_independent_authenticated_answers(vector):
    parameters = json.dumps(vector["parameters"])
    plain = bytes.fromhex(vector["plaintext_hex"])
    ciphertext = bytes.fromhex(vector["ciphertext_hex"])
    tag = bytes.fromhex(vector["tag_hex"])
    assert cube.stream_auth_encrypt(plain, parameters) == (ciphertext, tag)
    assert cube.stream_auth_decrypt(ciphertext, tag, parameters) == plain
    forged = bytes([tag[0] ^ 128]) + tag[1:]
    with pytest.raises(ValueError, match="authentication failed") as error:
        cube.stream_auth_decrypt(ciphertext, forged, parameters)
    assert error.value.code == "invalid_input"


def test_authentication_settings_and_padding_fail_without_private_excerpts():
    good = dict(VECTORS[4]["parameters"])
    for field in good:
        missing = dict(good)
        del missing[field]
        with pytest.raises(ValueError):
            cube.stream_auth_encrypt(b"", json.dumps(missing))
    for field, value in [
        ("key_hex", "SECRET"),
        ("nonce_hex", None),
        ("data_bits", 524289),
        ("tag_bits", 0),
        ("tag_bits", 33),
        ("tag_bits", 1.5),
        ("associated_data", "SECRET"),
    ]:
        with pytest.raises(ValueError) as error:
            cube.stream_auth_encrypt(b"", json.dumps(dict(good, **{field: value})))
        assert "SECRET" not in str(error.value)
    settings = json.dumps(good)
    with pytest.raises(ValueError):
        cube.stream_auth_encrypt(bytes.fromhex("123456789e81"), settings)
    cipher = bytes.fromhex(VECTORS[4]["ciphertext_hex"])
    tag = bytes.fromhex(VECTORS[4]["tag_hex"])
    with pytest.raises(ValueError, match="authentication failed"):
        cube.stream_auth_decrypt(bytes([cipher[0] ^ 128]) + cipher[1:], tag, settings)
    for invalid in (b"", bytes(5)):
        with pytest.raises(ValueError, match="authentication failed"):
            cube.stream_auth_decrypt(cipher, invalid, settings)
