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


def test_exact_clock_convention_is_separate_and_reports_uncertainty():
    assert cube.clock_directions("3:00", decode=True)["decoded"] == "J"
    assert cube.clock_directions("12:45")["hour"]["ambiguous"] is True
    r = cube.clock_directions(hour_millidegrees=-45000, minute_millidegrees=0)
    assert r["hour"]["directions"] == ["NW"]
    with pytest.raises(ValueError):
        cube.clock_directions("12:45", decode=True)
    with pytest.raises(ValueError):
        cube.clock_directions("3:00", hour_millidegrees=0, minute_millidegrees=0)
    with pytest.raises((TypeError, OverflowError)):
        cube.clock_directions(hour_millidegrees=float("nan"), minute_millidegrees=0)
