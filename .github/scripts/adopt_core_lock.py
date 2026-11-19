"""Verify the exact Core lock identities added to the reviewed Play CI baseline."""

import subprocess
import sys
from pathlib import Path

import tomllib


def sourced_packages(path):
    with path.open("rb") as file:
        packages = tomllib.load(file)["package"]
    return {
        (entry["name"], entry["version"], entry["source"], entry.get("checksum"))
        for entry in packages
        if "source" in entry
    }


def main():
    play = Path(sys.argv[1]).resolve()
    core = Path(sys.argv[2]).resolve()
    existing = sourced_packages(play / "Cargo.lock")
    reviewed = sourced_packages(core / "Cargo.lock")
    # A cold runner may still need the index entries for Play-only dependencies.
    # Accept only versions and checksums from the existing Play and exact Core locks.
    command = ["cargo", "update", "--package", "cube-core"]
    cached = subprocess.run([*command, "--offline"], cwd=play, check=False)
    if cached.returncode:
        subprocess.run(command, cwd=play, check=True)
    resolved = sourced_packages(play / "Cargo.lock")
    unexpected = (resolved - existing) - reviewed
    replaced = existing - resolved
    if unexpected or replaced:
        raise SystemExit(
            f"Unexpected Play dependency lock changes. Added: {sorted(unexpected)}. "
            f"Removed or replaced: {sorted(replaced)}."
        )
    print(
        f"Verified {len(resolved - existing)} added, pinned Core dependency identities."
    )


if __name__ == "__main__":
    main()
