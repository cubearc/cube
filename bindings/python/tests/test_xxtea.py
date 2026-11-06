"""Whole-word XXTEA answers through the installed native Core extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/ciphers/xxtea/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("vector", VECTORS)
def test_independent_raw_word_answers(vector):
    key = bytes.fromhex(vector["key_hex"])
    plain = bytes.fromhex(vector["plaintext_hex"])
    ciphertext = bytes.fromhex(vector["ciphertext_hex"])
    assert cube.xxtea_encrypt_words(plain, key) == ciphertext
    assert cube.xxtea_decrypt_words(ciphertext, key) == plain
    assert cube.xxtea_encrypt_words(b"", key) == b""
    assert cube.xxtea_decrypt_words(b"", key) == b""


def test_exact_key_and_word_boundaries():
    for function in [cube.xxtea_encrypt_words, cube.xxtea_decrypt_words]:
        for data, key, code in [
            (b"", b"SECRET", "invalid_parameter"),
            (bytes(4), bytes(16), "invalid_input"),
            (bytes(7), bytes(16), "invalid_input"),
            (bytes(9), bytes(16), "invalid_input"),
            (bytes(65537), bytes(16), "resource_limit"),
        ]:
            with pytest.raises(ValueError) as caught:
                function(data, key)
            assert caught.value.code == code
            assert "SECRET" not in str(caught.value)
    data = bytes([0x95]) * 65536
    assert (
        cube.xxtea_decrypt_words(cube.xxtea_encrypt_words(data, bytes(16)), bytes(16))
        == data
    )
    assert (
        cube.xxtea_encrypt_words(bytes(16), bytes(16))
        != cube.xxtea_encrypt_words(bytes(8), bytes(16)) * 2
    )
