"""Execute the installed native stream reference API with published answers."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/stream/reference/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("vector", VECTORS)
def test_independent_stream_answers(vector):
    parameters = json.dumps(vector["parameters"])
    plain = bytes.fromhex(vector["plaintext_hex"])
    encrypted = bytes.fromhex(vector["ciphertext_hex"])
    assert cube.stream_encrypt(plain, parameters) == encrypted
    assert cube.stream_decrypt(encrypted, parameters) == plain
    for function in (cube.stream_encrypt, cube.stream_decrypt):
        assert function(b"", parameters) == b""
    for field, value in [("key_hex", ""), ("key_hex", "00" * 257), ("nonce_hex", None)]:
        invalid = dict(vector["parameters"], **{field: value})
        for function in (cube.stream_encrypt, cube.stream_decrypt):
            with pytest.raises(ValueError):
                function(b"", json.dumps(invalid))


def test_stream_errors_do_not_echo_private_settings():
    for function in (cube.stream_encrypt, cube.stream_decrypt):
        with pytest.raises(ValueError) as error:
            function(
                b"", '{"profile":"rc4_rfc6229_v1","key_hex":"SECRET","nonce_hex":""}'
            )
        assert "SECRET" not in str(error.value)


def test_rc4_native_execution_and_inverse():
    parameters = json.dumps(
        {"profile": "rc4_rfc6229_v1", "key_hex": "0102030405", "nonce_hex": ""}
    )
    expected = bytes.fromhex("b2396305f03dc027ccc3524a0a1118a8")
    assert cube.stream_encrypt(bytes(16), parameters) == expected
    assert cube.stream_decrypt(expected, parameters) == bytes(16)


@pytest.mark.parametrize("profile", ["seal3_be_l32768_v1", "seal3_le_l32768_v1"])
def test_seal_last_position_and_exhaustion(profile):
    parameters = json.dumps(
        dict(profile=profile, key_hex="00" * 20, nonce_hex="ffffffff")
    )
    for function in (cube.stream_encrypt, cube.stream_decrypt):
        assert len(function(bytes(4096), parameters)) == 4096
        with pytest.raises(ValueError, match="counter exhausted"):
            function(bytes(4097), parameters)
