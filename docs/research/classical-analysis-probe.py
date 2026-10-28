"""Original reproducible research, not an installed solver or operation.

Run from the Cube checkout: python3 docs/research/classical-analysis-probe.py
Uses only Python's standard library and the already-built real Cube CLI.
"""

import collections
import json
import math
import random
import subprocess
import tempfile
import time
import tracemalloc
import zlib
from pathlib import Path

AZ = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"


def enumerate_maps(fixed, work_limit=1 << 20, result_limit=4096):
    """Lexical DFS with all-different propagation; count every attempted value."""
    if len(set(fixed.values())) != len(fixed):
        return {"solutions": 0, "work": 0, "stop": "contradiction"}
    remaining = [c for c in AZ if c not in fixed]
    work = 0
    solutions = 0
    stop = "complete"

    def visit(index, used):
        nonlocal work, solutions, stop
        if index == len(remaining):
            solutions += 1
            return True
        for value in AZ:
            if work == work_limit:
                stop = "work_limit"
                return False
            work += 1
            if value in used:
                continue
            if solutions == result_limit:
                stop = "result_limit"
                return False
            if not visit(index + 1, used | {value}):
                return False
        return True

    visit(0, set(fixed.values()))
    return {"solutions": solutions, "work": work, "stop": stop}


def hill_climb(fixed, seed, steps=4096):
    """Seeded swap heuristic, scoring only exact supplied crib constraints."""
    rng = random.Random(seed)
    key = list(AZ)
    rng.shuffle(key)

    def score(k):
        return sum(k[AZ.index(c)] == v for c, v in fixed.items())

    best = score(key)
    for _ in range(steps):
        a, b = rng.sample(range(26), 2)
        key[a], key[b] = key[b], key[a]
        new = score(key)
        if new >= best:
            best = new
        else:
            key[a], key[b] = key[b], key[a]
    return {
        "satisfied": best,
        "constraints": len(fixed),
        "score_visits": (steps + 1) * len(fixed),
        "seed": seed,
    }


def suggestions(data, budget=1 << 24):
    """Three deliberately small research rules; labels are not probabilities."""
    if len(data) > 1 << 20:
        raise ValueError("input_limit")
    if len(data) > budget:
        return {"complete": False, "rules": [], "work": 0}
    letters = bytes(c for c in data.upper() if 65 <= c <= 90)
    counts = collections.Counter(letters)
    n = len(letters)
    ic = sum(c * (c - 1) for c in counts.values()) / (n * (n - 1)) if n > 1 else None
    rules = []
    if n >= 20 and ic is not None:
        if ic >= 0.055:
            rules += ["high_ic.monoalphabetic", "high_ic.transposition"]
        else:
            rules += ["low_ic.polyalphabetic"]
    if n >= 6 and len(set(letters[i : i + 3] for i in range(n - 2))) < n - 2:
        rules += ["repeat.possible_period"]
    return {"complete": True, "rules": rules, "work": len(data), "letters": n, "ic": ic}


def main():
    started = time.perf_counter()
    tracemalloc.start()
    fixed = dict(zip(AZ[:24], AZ[:24], strict=True))
    assert enumerate_maps(fixed)["solutions"] == math.factorial(2)
    assert enumerate_maps(fixed, work_limit=78)["stop"] == "complete"
    assert enumerate_maps(fixed, work_limit=77)["stop"] == "work_limit"
    assert enumerate_maps(fixed, result_limit=1)["stop"] == "result_limit"

    # Exact dictionary words preserve repeated-symbol equality and bijection.
    def pattern(word):
        seen = {}
        return tuple(seen.setdefault(c, len(seen)) for c in word)

    assert [w for w in ["XYY", "ZWW"] if pattern(w) == pattern("ABB")] == ["XYY", "ZWW"]
    assert [w for w in ["XYZ", "XXX"] if pattern(w) == pattern("ABB")] == []
    assert enumerate_maps(dict(zip(AZ, AZ, strict=True)))["solutions"] == 1
    assert enumerate_maps({"A": "X", "B": "X"})["stop"] == "contradiction"
    assert enumerate_maps(fixed, work_limit=0)["stop"] == "work_limit"
    assert hill_climb(fixed, 7) == hill_climb(fixed, 7)
    substitution = {
        str(n): {
            "dfs": enumerate_maps(dict(zip(AZ[:n], AZ[:n], strict=True))),
            "seeded": hill_climb(dict(zip(AZ[:n], AZ[:n], strict=True)), 7),
        }
        for n in [0, 20, 24, 26]
    }
    plain = b"THE QUICK FOX RETURNS TO THE SAME QUIET GARDEN EVERY EVENING " * 4
    caesar = bytes(65 + (c - 65 + 3) % 26 if 65 <= c <= 90 else c for c in plain)
    poly = bytes(
        65 + (c - 65 + (i % 7)) % 26 if 65 <= c <= 90 else c
        for i, c in enumerate(plain)
    )
    fixtures = {
        "original_plain": plain,
        "caesar": caesar,
        "transposition": plain[::-1],
        "polyalphabetic": poly,
        "short": b"XYZ",
        "unicode": "ação λ雪".encode(),
        "compressed": zlib.compress(plain),
        "hex_encoded": plain.hex().encode(),
        "uniform_random": b"",
        "structured_negative": b"AB" * 128,
    }
    # The random corpus uses a single advancing generator, not a repeated seed.
    rng = random.Random(71)
    fixtures["uniform_random"] = bytes(rng.randrange(256) for _ in range(256))
    comparison = {}
    with tempfile.TemporaryDirectory() as directory:
        for name, data in fixtures.items():
            path = Path(directory) / name
            path.write_bytes(data)
            triage = json.loads(
                subprocess.check_output(
                    ["target/debug/cube", "investigate", "triage", str(path), "--json"]
                )
            )
            comparison[name] = {
                "baseline_rules": [s["rule"] for s in triage["suggestions"]],
                "experimental": suggestions(data),
            }
            assert suggestions(data) == suggestions(data)
    assert suggestions(b"A" * 32, 31)["complete"] is False
    assert suggestions(b"A" * 32, 32)["complete"] is True
    assert (
        suggestions(plain)["rules"]
        == suggestions(caesar)["rules"]
        == suggestions(plain[::-1])["rules"]
    )
    peak = tracemalloc.get_traced_memory()[1]
    print(
        json.dumps(
            {
                "substitution": substitution,
                "family_comparison": comparison,
                "python_peak_bytes": peak,
                "elapsed_seconds": time.perf_counter() - started,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
