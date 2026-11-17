"""Raw group schemes run in the installed SDK against independent BC/Python cases."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/group_encryption/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_independent_group_answers_fresh_ciphertexts_and_key_encoding(v):
    encrypt = getattr(cube, v["scheme"] + "_encrypt")
    decrypt = getattr(cube, v["scheme"] + "_decrypt")
    p = json.dumps(v["parameters"])
    private = bytes.fromhex(v["private_key_hex"])
    public = bytes.fromhex(v["public_key_hex"])
    plain = bytes.fromhex(v["plaintext_hex"])
    cipher = bytes.fromhex(v["ciphertext_hex"])
    assert decrypt(cipher, private, p) == plain
    first = encrypt(plain, public, p)
    assert first != encrypt(plain, public, p)
    assert decrypt(first, private, p) == plain
    with pytest.raises(ValueError):
        decrypt(cipher, bytes(len(private)), p)
    with pytest.raises(ValueError):
        encrypt(bytes(256), public, p)
    if v["scheme"] == "cramer_shoup":
        with pytest.raises(ValueError):
            decrypt(cipher[:-1] + bytes([cipher[-1] ^ 1]), private, p)
        wrong = {**v["parameters"], "label_hex": v["parameters"]["label_hex"] + "00"}
        with pytest.raises(ValueError):
            decrypt(cipher, private, json.dumps(wrong))
