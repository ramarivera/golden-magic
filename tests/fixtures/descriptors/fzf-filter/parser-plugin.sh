#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

rows = []
for line in sys.stdin:
    line = line.rstrip("\n")
    if ": " not in line:
        continue
    kind, value = line.split(": ", 1)
    rows.append({"kind": kind, "value": value})

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
