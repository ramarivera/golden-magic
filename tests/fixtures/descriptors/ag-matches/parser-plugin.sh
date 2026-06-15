#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

rows = []
for line in sys.stdin:
    parts = line.rstrip("\n").split(":", 2)
    if len(parts) == 3 and parts[1].isdigit():
        rows.append({"path": parts[0], "line": parts[1], "match": parts[2]})

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
