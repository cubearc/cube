"""Import the pinned Apache Commons Codec 1.15 Beider–Morse rule resources."""

import argparse
import json
import re
import zipfile
from collections import defaultdict
from functools import cache
from hashlib import sha256
from pathlib import Path

SOURCE_DIGEST = "7019940b2298d333edb946e2db3d10f1caacbbd52bb64e85832cfd0017e049cc"
RESOURCE = "org/apache/commons/codec/language/bm/"
KINDS = ("gen", "ash", "sep")
QUOTED = re.compile(r'"([^"]*)"')


def statements(text):
    in_comment = False
    for line in text.splitlines():
        if line.startswith("/*"):
            in_comment = True
        if in_comment:
            if line.endswith("*/"):
                in_comment = False
            continue
        text = line.partition("//")[0].strip()
        if text:
            yield text


def read_rules(source):
    content = source.read_bytes()
    if sha256(content).hexdigest() != SOURCE_DIGEST:
        raise ValueError("Expected the pinned Apache Commons Codec 1.15 source JAR")
    with zipfile.ZipFile(source) as jar:
        raw = {
            Path(name).stem: jar.read(name).decode("utf-8")
            for name in jar.namelist()
            if name.startswith(RESOURCE) and name.endswith(".txt")
        }

    @cache
    def table(name):
        indexed = defaultdict(list)
        for line in statements(raw[name]):
            if line.startswith("#include "):
                # The reference replaces existing first-character buckets.
                indexed.update(
                    {key: list(value) for key, value in table(line[9:]).items()}
                )
                continue
            values = QUOTED.findall(line)
            if len(values) != 4 or not values[0]:
                raise ValueError((name, line))
            indexed[values[0][0]].append(values)
        return dict(indexed)

    languages = {kind: list(statements(raw[f"{kind}_languages"])) for kind in KINDS}
    guess = {}
    tables = {}
    for kind, keys in languages.items():
        guess[kind] = []
        for line in statements(raw[f"{kind}_lang"]):
            pattern, names, accept = line.split()
            guess[kind].append((pattern, names.split("+"), accept == "true"))
        for mode in ("rules", "exact", "approx"):
            selected = keys + (["common"] if mode != "rules" else [])
            for key in selected:
                name = f"{kind}_{mode}_{key}"
                tables[name] = table(name)
    return {"languages": languages, "guess": guess, "tables": tables}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    options = parser.parse_args()
    data = json.dumps(
        read_rules(options.source),
        sort_keys=True,
        ensure_ascii=False,
        separators=(",", ":"),
    )
    options.output.write_text(data + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
