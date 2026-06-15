#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

rows = []
for line in sys.stdin:
    line = line.rstrip("\n")
    if line.startswith("HTTP/"):
        rows.append({"name": "status", "value": line})
    elif ":" in line:
        name, value = line.split(":", 1)
        rows.append({"name": name, "value": value.strip()})

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
