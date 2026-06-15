#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

rows = []
for line in sys.stdin:
    line = line.rstrip("\n")
    parts = line.split(":", 3)
    if len(parts) != 4:
        continue
    path, line_no, column, match = parts
    rows.append({
        "path": path,
        "line": line_no,
        "column": column,
        "match": match,
    })

print(json.dumps({
    "protocol": "golden-magic.executable-json.v1",
    "rows": rows,
}, separators=(",", ":")))
'
