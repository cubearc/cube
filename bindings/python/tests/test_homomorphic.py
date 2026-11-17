"""Verify external ciphertext results through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = (
    Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/homomorphic/vectors"
)


@pytest.mark.parametrize(
    ("family", "profile", "evaluate"),
    [
        ("bfv", "bfv_seal412_degree1024_t17_raw_v1", cube.bfv_add),
        ("bgv", "bgv_seal412_degree1024_t17_ntt_raw_v1", cube.bgv_add),
        ("ckks", "ckks_seal412_degree2048_scale32_ntt_raw_v1", cube.ckks_add),
    ],
)
def test_seal_addition(family, profile, evaluate):
    left = (VECTORS / f"{family}-lhs.bin").read_bytes()
    right = (VECTORS / f"{family}-rhs.bin").read_bytes()
    expected = (VECTORS / f"{family}-sum.bin").read_bytes()
    settings = json.dumps({"profile": profile})
    assert evaluate(left, right, settings) == expected
    with pytest.raises(ValueError):
        evaluate(left, right[:-1], settings)
    with pytest.raises(ValueError):
        evaluate(left, right, "{}")


def test_tfhe_not():
    settings = '{"profile":"tfhe_rs182_boolean_default_lwe805_raw_v1"}'
    ciphertext = (VECTORS / "tfhe-input.bin").read_bytes()
    expected = (VECTORS / "tfhe-not.bin").read_bytes()
    assert cube.tfhe_not(ciphertext, settings) == expected
    assert cube.tfhe_not(expected, settings) == ciphertext
    with pytest.raises(ValueError):
        cube.tfhe_not(b"CBF1" + ciphertext[4:], settings)


@pytest.mark.parametrize(
    ("family", "profile", "evaluate"),
    [
        ("paillier", "paillier_g_n_plus_1_be_v1", cube.paillier_add),
        ("damgard-jurik", "damgard_jurik_s2_g_n_plus_1_be_v1", cube.damgard_jurik_add),
    ],
)
def test_additive_ciphertext_results(family, profile, evaluate):
    vectors = VECTORS.parent.parent / "additive/vectors"
    left, right, modulus, expected = [
        (vectors / f"{family}-{role}.bin").read_bytes()
        for role in ("lhs", "rhs", "modulus", "sum")
    ]
    settings = json.dumps({"profile": profile})
    assert evaluate(left, right, modulus, settings) == expected
    with pytest.raises(ValueError):
        evaluate(bytes(len(left)), right, modulus, settings)


@pytest.mark.parametrize(
    ("family", "profile", "encrypt", "decrypt"),
    [
        (
            "ntru-encrypt",
            "ntru_encrypt_educational_n11_p3_q32",
            cube.ntru_encrypt_encrypt,
            cube.ntru_encrypt_decrypt,
        ),
        (
            "regev",
            "regev_educational_n4_m16_q97",
            cube.regev_encrypt,
            cube.regev_decrypt,
        ),
    ],
)
def test_educational_lattice_round_trip(family, profile, encrypt, decrypt):
    vectors = VECTORS.parent.parent / "lattice_reference"
    vector = {
        name: bytes(data)
        for name, data in json.loads((vectors / f"{family}.json").read_text()).items()
    }
    settings = json.dumps({"profile": profile})
    assert (
        decrypt(vector["ciphertext"], vector["private_key"], settings)
        == vector["plaintext"]
    )
    ciphertext = encrypt(vector["plaintext"], vector["public_key"], settings)
    assert decrypt(ciphertext, vector["private_key"], settings) == vector["plaintext"]
    with pytest.raises(ValueError):
        encrypt(vector["plaintext"], vector["public_key"][:-1], settings)
