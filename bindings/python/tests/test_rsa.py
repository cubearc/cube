"""Actual native RSA profiles against independent ciphertexts and rejection bytes."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/rsa"
INTEROP = json.loads((ROOT / "interoperability.json").read_text())
OAEP = json.loads((ROOT / "wycheproof_oaep.json").read_text())["testGroups"][0]
LEGACY = [
    (g["parameters"], g["key_hex"], v)
    for g in json.loads((ROOT / "implicit_rejection_vectors.json").read_text())[
        "groups"
    ]
    for v in g["cases"]
]


@pytest.mark.parametrize("v", INTEROP)
def test_independent_formats_moduli_and_randomized_encryption(v):
    private = json.dumps(v["private_parameters"])
    public = json.dumps(v["public_parameters"])
    expected = bytes.fromhex(v["plaintext_hex"])
    assert (
        cube.rsa_oaep_decrypt(
            bytes.fromhex(v["ciphertext_hex"]),
            bytes.fromhex(v["private_key_hex"]),
            private,
        )
        == expected
    )
    a = cube.rsa_oaep_encrypt(expected, bytes.fromhex(v["public_key_hex"]), public)
    b = cube.rsa_oaep_encrypt(expected, bytes.fromhex(v["public_key_hex"]), public)
    assert a != b
    assert len(a) == (v["bits"] + 7) // 8
    assert (
        cube.rsa_oaep_decrypt(a, bytes.fromhex(v["private_key_hex"]), private)
        == expected
    )


@pytest.mark.parametrize("v", OAEP["tests"])
def test_wycheproof_oaep_with_explicit_cfrg10_exception(v):
    p = json.dumps(
        {
            "profile": "rsa_oaep_sha256_mgf1_sha256_cfrg10",
            "key_format": "pkcs8_der",
            "label_hex": v["label"],
        }
    )
    data = bytes.fromhex(v["ct"])
    if v["result"] == "valid" or v["tcId"] == 23:
        assert cube.rsa_oaep_decrypt(
            data, bytes.fromhex(OAEP["privateKeyPkcs8"]), p
        ) == bytes.fromhex(v["msg"])
    else:
        with pytest.raises(ValueError, match="RSA operation failed"):
            cube.rsa_oaep_decrypt(data, bytes.fromhex(OAEP["privateKeyPkcs8"]), p)


@pytest.mark.parametrize("v", LEGACY)
def test_cfrg10_implicit_rejection_matches_independent_bytes(v):
    parameters, key, v = v
    p = json.dumps(parameters)
    data = bytes.fromhex(v["ciphertext_hex"])
    if v["expect"] == "public_error":
        with pytest.raises(ValueError, match="RSA operation failed"):
            cube.rsa_pkcs1_v1_5_decrypt(data, bytes.fromhex(key), p)
    else:
        assert cube.rsa_pkcs1_v1_5_decrypt(
            data, bytes.fromhex(key), p
        ) == bytes.fromhex(v["output_hex"])


def test_secret_errors_and_limits_are_redacted():
    p = json.dumps(INTEROP[0]["private_parameters"])
    with pytest.raises(ValueError) as e:
        cube.rsa_oaep_decrypt(b"", b"SECRET", p)
    assert "SECRET" not in str(e.value)
    p = json.dumps(INTEROP[0]["public_parameters"])
    with pytest.raises(ValueError):
        cube.rsa_oaep_encrypt(
            bytes(191), bytes.fromhex(INTEROP[0]["public_key_hex"]), p
        )
