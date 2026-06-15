#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

lines = [line.rstrip("\n") for line in sys.stdin]
rows = []
index = 0
while index < len(lines):
    language = lines[index].strip()
    index += 1
    if not language:
        continue
    if index >= len(lines):
        break
    extensions = lines[index].strip()
    index += 1
    if extensions.startswith("*."):
        rows.append({"language": language, "extensions": extensions})

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
