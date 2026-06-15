#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import re
import sys

pattern = re.compile(r"^(?P<mode>[dl-][rwx-]{9})\s+(?P<links>\d+)\s+(?P<owner>\S+)\s+(?P<group>\S+)\s+(?P<size>\d+)\s+(?P<month>\S+)\s+(?P<day>\d+)\s+(?P<time>\S+)\s+(?P<path>.+)$")
rows = []
for line in sys.stdin:
    match = pattern.match(line.rstrip("\n"))
    if match:
        rows.append(match.groupdict())

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
