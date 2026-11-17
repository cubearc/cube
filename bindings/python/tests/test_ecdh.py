"""Independent raw ECDH answers execute through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/ecdh/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_independent_shared_secrets(v):
    assert cube.ecdh_agree(
        bytes.fromhex(v["peer_hex"]), json.dumps(v["parameters"])
    ) == bytes.fromhex(v["shared_hex"])


def test_rejects_weak_points_bad_lengths_and_redacts_private_settings():
    for v in VECTORS:
        parameters = json.dumps(v["parameters"])
        for length in [0, 1, 31, 32, 33, 65, 66]:
            with pytest.raises(ValueError):
                cube.ecdh_agree(bytes(length), parameters)
    with pytest.raises(ValueError) as error:
        cube.ecdh_agree(
            bytes(32), '{"profile":"x25519_rfc7748","private_key_hex":"SECRET"}'
        )
    assert "SECRET" not in str(error.value)
