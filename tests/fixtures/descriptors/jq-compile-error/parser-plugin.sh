#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import re
import sys

rows = []
detail = re.compile(r"^jq: error: (?P<message>.*) at (?P<location>.*), line (?P<line>\d+):$")
summary = re.compile(r"^jq: (?P<count>\d+) (?P<message>compile error)$")
for line in sys.stdin:
    line = line.rstrip("\n")
    if match := detail.match(line):
        row = {"kind": "error"}
        row.update(match.groupdict())
        rows.append(row)
    elif match := summary.match(line):
        row = {"kind": "summary"}
        row.update(match.groupdict())
        rows.append(row)

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
