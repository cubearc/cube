import cube
import pytest


def test_letter_pairs_arm_order_and_typed_composition():
    assert cube.semaphore_encode("A B") == "S,SW / S,W"
    assert cube.semaphore_decode("S,SW / S,W") == "A B"
    assert cube.semaphore_encode("I K") == "SW,NW / N,SW"
    assert (
        cube.semaphore_decode(cube.semaphore_encode("Hello  World")) == "HELLO  WORLD"
    )
    for bad in ["S,S", "SW,S", "NE,N", "S,SW "]:
        with pytest.raises(ValueError):
            cube.semaphore_decode(bad)
    steps = [
        {"operation": "symbol.semaphore.encode", "version": 1},
        {"operation": "symbol.semaphore.decode", "version": 1},
    ]
    assert cube.execute_pipeline(steps, "A B", capture=[0])["output"] == "A B"
