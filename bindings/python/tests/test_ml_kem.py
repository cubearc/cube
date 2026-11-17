"""Test FIPS 203 through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/ml_kem/vectors.json"
    ).read_text()
)["profiles"]


@pytest.mark.parametrize("vector", VECTORS, ids=lambda v: v["profile"])
def test_acvp_and_fresh_key_exchange(vector):
    parameters = json.dumps({"profile": vector["profile"]})
    for answer in vector["decapsulation"]:
        assert cube.ml_kem_decapsulate(
            bytes.fromhex(answer["c"]), bytes.fromhex(answer["dk"]), parameters
        ) == bytes.fromhex(answer["k"])

    public, private = cube.ml_kem_keygen(parameters)
    assert len(public) * 2 == len(vector["keygen"]["ek"])
    assert len(private) * 2 == len(vector["keygen"]["dk"])
    assert cube.ml_kem_keygen(parameters)[0] != public
    ciphertext, shared = cube.ml_kem_encapsulate(public, parameters)
    assert cube.ml_kem_decapsulate(ciphertext, private, parameters) == shared

    with pytest.raises(ValueError):
        cube.ml_kem_decapsulate(ciphertext[:-1], private, parameters)
    with pytest.raises(ValueError):
        cube.ml_kem_decapsulate(ciphertext, private[:-1], parameters)
    with pytest.raises(ValueError):
        cube.ml_kem_encapsulate(public, '{"profile":"kyber768"}')


@pytest.mark.parametrize("index", range(3))
def test_historical_kyber_vectors(index):
    path = (
        Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/kyber/vectors.json"
    )
    vector = json.loads(path.read_text())["profiles"][index]
    profile = json.dumps({"profile": vector["profile"]})
    key = bytes.fromhex(vector["dk"])
    ciphertext = bytes.fromhex(vector["ciphertext"])
    assert cube.kyber_decapsulate(ciphertext, key, profile) == bytes.fromhex(
        vector["shared_secret"]
    )
    assert cube.kyber_decapsulate(
        bytes.fromhex(vector["modified_ciphertext"]), key, profile
    ) == bytes.fromhex(vector["rejection_secret"])
    with pytest.raises(ValueError):
        cube.kyber_decapsulate(ciphertext[:-1], key, profile)


@pytest.mark.parametrize(
    ("operation", "name", "profile"),
    [
        (cube.frodo_kem_decapsulate, "frodo640", "frodokem640_shake_salted"),
        (cube.hqc_decapsulate, "hqc128", "hqc128_20260210_submission"),
        (
            cube.classic_mceliece_decapsulate,
            "mceliece460896f",
            "mceliece460896f_round4",
        ),
    ],
)
def test_alternate_kems_match_external_answers(operation, name, profile):
    directory = (
        Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/alternate_kem"
    )
    vector = json.loads((directory / f"{name}.json").read_text())
    ciphertext = bytes.fromhex(vector["ct"])
    private_key = bytes.fromhex(vector["sk"])
    parameters = json.dumps({"profile": profile})
    assert operation(ciphertext, private_key, parameters) == bytes.fromhex(vector["ss"])
    with pytest.raises(ValueError):
        operation(ciphertext[:-1], private_key, parameters)
    with pytest.raises(ValueError):
        operation(ciphertext, private_key, '{"profile":"other"}')
