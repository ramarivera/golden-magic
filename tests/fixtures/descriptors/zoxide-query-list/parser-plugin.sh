#!/usr/bin/env bash
set -euo pipefail

python3 -c '
import json
import sys

rows = [{"path": line.rstrip("\n")} for line in sys.stdin if line.startswith("/")]
print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
'
