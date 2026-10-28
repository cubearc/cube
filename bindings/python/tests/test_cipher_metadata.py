"""Opt-in profile facts use native discovery and preserve strict legacy views."""

import json

import cube
import pytest
from test_discovery import SDK_BINDINGS


def test_optional_profiles_preserve_legacy_descriptors_and_real_availability():
    legacy = cube.operations()
    extended = cube.operations(include_metadata=True)
    adapters = {a["operation"]: a for a in cube.pipeline_adapters()["adapters"]}
    assert len(legacy) == len(extended) == 305
    annotated = 0
    for old, new in zip(legacy, extended, strict=True):
        assert "metadata" not in old
        assert cube.describe_operation(new["id"], include_metadata=True) == new
        assert new in cube.operations(new["category"], include_metadata=True)
        profile = new.get("metadata")
        assert {k: v for k, v in new.items() if k != "metadata"} == old
        if profile is None:
            continue
        annotated += 1
        assert profile["schema_version"] == 1
        assert profile["profile"]["variant"] == old["description"]
        assert profile["python_api"] == SDK_BINDINGS[new["id"]]
        assert profile["rust_api"].startswith("cube_core::")
        security = profile["security"]
        if security["private_input"] or security["private_output"]:
            assert new["pipeline"] is None
        assert new["pipeline"] == (
            {"version": adapters[new["id"]]["version"]}
            if new["id"] in adapters
            else None
        )
        for alias in profile["aliases"]:
            with pytest.raises(ValueError, match="unknown operation ID"):
                cube.describe_operation(alias["name"], include_metadata=True)
    assert annotated == 189
    assert "metadata" not in cube.describe_operation(
        "file.info.inspect", include_metadata=True
    )


def test_profile_facts_distinguish_legacy_security_variants_and_private_material():
    def metadata(operation):
        return cube.describe_operation(operation, include_metadata=True)["metadata"]

    assert metadata("hash.md5")["security"]["classification"] == "legacy_digest"
    assert metadata("hash.crc32")["security"]["classification"] == "checksum"
    assert metadata("hash.sha256")["profile"]["block_bits"] == 512
    assert metadata("hash.sha256")["profile"]["output_bits"] == 256
    assert metadata("hash.keccak256")["profile"]["block_bits"] == 1088
    private = metadata("blockchain.private.export")
    assert private["security"]["private_output"] is True
    with pytest.raises(ValueError, match="unknown operation ID"):
        cube.describe_operation("cipher.aes.encrypt", include_metadata=True)


def test_discovery_output_bounds_are_native_exact_and_query_errors_are_typed():
    for function, args in [
        (cube.operations, ()),
        (cube.describe_operation, ("hash.sha256",)),
    ]:
        for include_metadata in (False, True):
            value = function(*args, include_metadata=include_metadata)
            # serde's compact UTF-8 JSON includes one terminating newline.
            size = (
                len(
                    json.dumps(
                        value, ensure_ascii=False, separators=(",", ":")
                    ).encode()
                )
                + 1
            )
            assert (
                function(
                    *args, include_metadata=include_metadata, max_output_bytes=size
                )
                == value
            )
            with pytest.raises(ValueError) as error:
                function(
                    *args, include_metadata=include_metadata, max_output_bytes=size - 1
                )
            assert error.value.code == "resource_limit"
            assert error.value.parameter == "max_output_bytes"
    for limit in (0, 1048577, -1, True, 1.5, "100", 2**128):
        with pytest.raises((ValueError, TypeError, OverflowError)):
            cube.operations(include_metadata=True, max_output_bytes=limit)
    for function, parameter in [
        (cube.operations, "category"),
        (cube.describe_operation, "id"),
    ]:
        with pytest.raises(ValueError) as error:
            function("é" * 129, include_metadata=True)
        assert error.value.code == "resource_limit"
        assert error.value.parameter == parameter
        with pytest.raises(ValueError) as error:
            function("missing", include_metadata=True)
        assert error.value.code == "invalid_parameter"
    assert cube.operations(max_output_bytes=None) == cube.operations()
