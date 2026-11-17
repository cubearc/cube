"""Independent authenticated ciphertext/tag answers through the installed wheel."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).parents[3] / "crates/cube-core/src/ciphers/modern_aead"
VECTORS = json.loads((ROOT / "vectors.json").read_text())
SIV_VECTORS = json.loads((ROOT / "siv_vectors.json").read_text())


@pytest.mark.parametrize("v", VECTORS)
def test_independent_aead_answers_and_forged_tags(v):
    p = json.dumps(v["parameters"])
    plain, aad, ciphertext, tag = (
        bytes.fromhex(v[name])
        for name in ("plaintext_hex", "aad_hex", "ciphertext_hex", "tag_hex")
    )
    assert cube.aead_encrypt(plain, aad, p) == (ciphertext, tag)
    assert cube.aead_decrypt(ciphertext, aad, tag, p) == plain
    bad = bytes([tag[0] ^ 1]) + tag[1:]
    with pytest.raises(ValueError, match="authentication failed"):
        cube.aead_decrypt(ciphertext, aad, bad, p)


@pytest.mark.parametrize("v", SIV_VECTORS)
def test_independent_nonce_free_siv_answers(v):
    p = json.dumps(v["parameters"])
    plain, ciphertext, tag = (
        bytes.fromhex(v[name])
        for name in ("plaintext_hex", "ciphertext_hex", "tag_hex")
    )
    components = [bytes.fromhex(x) for x in v["aad_components_hex"]]
    assert cube.aes_siv_encrypt(plain, components, p) == (ciphertext, tag)
    assert cube.aes_siv_decrypt(ciphertext, components, tag, p) == plain
    with pytest.raises(ValueError, match="authentication failed"):
        cube.aes_siv_decrypt(ciphertext, components + [b""], tag, p)


def test_exact_private_fields_and_no_nonce_default():
    for v in VECTORS:
        good = v["parameters"]
        for name in good:
            p = dict(good)
            del p[name]
            with pytest.raises(ValueError):
                cube.aead_encrypt(b"", b"", json.dumps(p))
    with pytest.raises(ValueError) as error:
        cube.aead_encrypt(b"", b"", '{"key_hex":"SECRET"}')
    assert "SECRET" not in str(error.value)
    p = json.dumps({"profile": "aes128_siv_cmac_nonce_free", "key_hex": "42" * 32})
    _, none_tag = cube.aes_siv_encrypt(b"", [], p)
    _, empty_tag = cube.aes_siv_encrypt(b"", [b""], p)
    assert none_tag != empty_tag
    with pytest.raises(ValueError):
        cube.aes_siv_encrypt(b"", [b""] * 127, p)
