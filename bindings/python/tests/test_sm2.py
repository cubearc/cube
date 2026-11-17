"""Independent OpenSSL SM2/SM3 vectors execute in the installed native SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/sm2/interoperability.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
@pytest.mark.parametrize(
    ("profile", "field"),
    [
        ("sm2_sm3_c1c3c2", "c1c3c2_hex"),
        ("sm2_sm3_c1c2c3", "c1c2c3_hex"),
        ("sm2_sm3_der", "der_hex"),
    ],
)
def test_openssl_answers_fresh_ciphertexts_and_tampering(v, profile, field):
    p = json.dumps({"profile": profile})
    private = bytes.fromhex(v["private_key_hex"])
    public = bytes.fromhex(v["public_key_hex"])
    plain = bytes.fromhex(v["plaintext_hex"])
    cipher = bytes.fromhex(v[field])
    assert cube.sm2_decrypt(cipher, private, p) == plain
    first = cube.sm2_encrypt(plain, public, p)
    assert first != cube.sm2_encrypt(plain, public, p)
    assert cube.sm2_decrypt(first, private, p) == plain
    with pytest.raises(ValueError):
        cube.sm2_decrypt(cipher[:-1] + bytes([cipher[-1] ^ 1]), private, p)
    with pytest.raises(ValueError):
        cube.sm2_encrypt(b"", public, p)
    with pytest.raises(ValueError):
        cube.sm2_decrypt(cipher, bytes(32), p)
