#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import re
import sys

rows = []
for line in sys.stdin.read().splitlines():
    match = re.match(r"^\s{4}(?P<recipe>.+?)\s{2,}(?P<comment># .+)$", line)
    if match:
        rows.append(match.groupdict())

print(json.dumps({
    "protocol": "golden-magic.executable-json.v1",
    "rows": rows,
}))
'
