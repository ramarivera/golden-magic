#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import re
import sys

pattern = re.compile(r"^(?P<path>.*?):(?P<line>\d+):(?P<column>\d+): (?P<severity>\w+): (?P<message>.*?) \[(?P<code>SC\d+)\]$")
rows = []
for line in sys.stdin:
    match = pattern.match(line.rstrip("\n"))
    if match:
        rows.append(match.groupdict())

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
