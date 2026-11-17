"""Independent NIST byte profiles through the installed native SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/ciphers/aes_modes/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("vector", VECTORS)
def test_nist_vectors(vector):
    plain = bytes.fromhex(vector["plaintext_hex"])
    ciphertext = bytes.fromhex(vector["ciphertext_hex"])
    settings = json.dumps(vector["parameters"])
    assert cube.aes_mode_encrypt(plain, settings) == ciphertext
    assert cube.aes_mode_decrypt(ciphertext, settings) == plain


def test_padding_counter_and_private_settings():
    p = {"mode": "cbc", "padding": "pkcs7", "key_hex": "42" * 16, "iv_hex": "24" * 16}
    for size in (0, 1, 15, 16, 17, 32):
        data = b"\xff" * size
        cipher = cube.aes_mode_encrypt(data, json.dumps(p))
        assert len(cipher) == (size // 16 + 1) * 16
        assert cube.aes_mode_decrypt(cipher, json.dumps(p)) == data
    for name in p:
        invalid = dict(p)
        del invalid[name]
        with pytest.raises(ValueError):
            cube.aes_mode_encrypt(b"", json.dumps(invalid))
    with pytest.raises(ValueError, match="invalid AES mode ciphertext or padding"):
        cube.aes_mode_decrypt(b"", json.dumps(p))
    p = {"mode": "ctr128_be", "key_hex": "00" * 16, "iv_hex": "ff" * 16}
    assert len(cube.aes_mode_encrypt(bytes(16), json.dumps(p))) == 16
    with pytest.raises(ValueError):
        cube.aes_mode_encrypt(bytes(17), json.dumps(p))
    with pytest.raises(ValueError) as error:
        cube.aes_mode_encrypt(b"", '{"key_hex":"SECRET"}')
    assert "SECRET" not in str(error.value)
