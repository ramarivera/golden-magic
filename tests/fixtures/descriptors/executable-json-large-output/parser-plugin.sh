#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
print('{"protocol":"golden-magic.executable-json.v1","rows":[')
print('{"blob":"' + ('x' * (1024 * 1024 + 1)) + '"}')
print(']}')
PY
