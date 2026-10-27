"""Complete native contracts preserve legacy discovery and exact wire semantics."""

import cube
import pytest


def test_complete_contract_discovery():
    operations = cube.operations(include_contracts=True)
    legacy = cube.operations()
    assert [op["id"] for op in operations] == [op["id"] for op in legacy]
    assert all(operation["contract"]["outputs"] for operation in operations)
    image = cube.describe_operation("image.histogram", include_contracts=True)
    assert image["contract"]["outputs"][0]["semantic_type"] == "image.histogram"
    assert "contract" not in cube.describe_operation("image.histogram")
    assert image["contract"]["policy"]["automatic_capture"] is False


def test_contract_discovery_rejects_short_budget():
    with pytest.raises(ValueError):
        cube.operations(include_contracts=True, max_output_bytes=1)
