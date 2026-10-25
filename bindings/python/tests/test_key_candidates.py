import json

import cube
import pytest

WIF = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
HEX = "0" * 63 + "1"


def test_ordered_incremental_safe_candidate_validation():
    entries = [
        {"format": "wif", "value": WIF},
        {"format": "hex", "value": HEX},
        {"format": "wif", "value": WIF[:-1]},
        {"format": "private-label", "value": "secret-material"},
        {"format": "hex", "value": HEX},
    ]
    batch = cube.validate_key_candidates(entries)
    assert isinstance(batch, cube.CandidateValidation)
    assert iter(batch) is batch and len(batch) == 5
    assert not batch.progress()["complete"]
    first = next(batch)
    assert first["index"] == 0 and first["valid"]
    assert first["wif"] == cube.validate_wif(WIF)
    assert len(batch) == 4 and batch.progress()["processed"] == 1
    assert not batch.progress()["complete"]
    results = [first, *batch]
    assert [r["index"] for r in results] == list(range(5))
    assert [r["valid"] for r in results] == [True, True, False, False, True]
    assert results[3]["format"] == "unsupported"
    assert results[3]["error"]["code"] == "unsupported_format"
    assert batch.progress() == dict(
        total=5, processed=5, valid=3, invalid=2, public_derived=0, complete=True
    )
    assert len(batch) == 0 and list(batch) == []
    report = json.dumps(dict(results=results, progress=batch.progress()), indent=2)
    assert len(report.encode()) <= batch.plan()["max_output_bytes"]
    for secret in [WIF, WIF[:-1], HEX, "private-label", "secret-material"]:
        assert secret not in report
    assert all("public_address" not in row for row in results)
    assert cube.validate_key_candidates([]).progress()["complete"]
    assert cube.describe_operation("blockchain.candidates.validate")["pipeline"] is None


def test_explicit_public_addresses_reuse_single_entry_intent():
    entries = [{"format": "wif", "value": WIF}, {"format": "hex", "value": HEX}]
    batch = cube.validate_key_candidates(
        entries, derive="bitcoin", network="mainnet", compressed=True
    )
    for result in batch:
        assert result["public_address"]["kind"] == "bitcoin"
        assert (
            result["public_address"]["value"]["address"]
            == "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"
        )
    assert batch.progress()["public_derived"] == 2
    results = list(
        cube.validate_key_candidates(
            entries, derive="bitcoin", network="testnet", compressed=True
        )
    )
    assert results[0]["valid"] and "address_error" in results[0]
    assert "public_address" not in results[0] and "public_address" in results[1]
    results = list(cube.validate_key_candidates(entries, derive="ethereum"))
    assert results[0]["valid"] and "address_error" in results[0]
    assert (
        results[1]["public_address"]["value"]["address"]
        == cube.derive_ethereum_address(HEX, format="hex")["address"]
    )
    for kwargs in [
        {"network": "mainnet"},
        {"derive": "bitcoin"},
        {"derive": "ethereum", "compressed": True},
        {"checksum": "eip55"},
    ]:
        with pytest.raises(ValueError):
            cube.validate_key_candidates(entries, **kwargs)


def test_preflight_candidate_limits_and_sanitized_transport_errors():
    entry = {"format": "hex", "value": HEX}
    assert len(cube.validate_key_candidates([entry] * 256)) == 256
    for entries in [
        [entry] * 257,
        [{"format": "hex", "value": "a" * 129}],
        [{"format": "wif", "value": "a" * 128}] * 256,
        [{"format": "hex", "value": HEX + "\ud800"}],
        [{"format": "hex", "value": "private-material", "extra": 0}],
        [{"format": "hex", "value": 1}],
        [1],
    ]:
        with pytest.raises(ValueError) as error:
            cube.validate_key_candidates(entries)
        assert "private-material" not in repr(error.value.args)
        assert not hasattr(error.value, "object")
    assert len(cube.validate_key_candidates([entry] * 32, derive="ethereum")) == 32
    with pytest.raises(ValueError):
        cube.validate_key_candidates([entry] * 33, derive="ethereum")
