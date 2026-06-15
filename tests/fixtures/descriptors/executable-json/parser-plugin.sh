#!/usr/bin/env bash
set -euo pipefail

rows=()
while IFS= read -r line; do
  [[ "$line" == plugin-row:* ]] || continue
  payload="${line#plugin-row: }"
  name="${payload%% *}"
  status="${payload#* }"
  rows+=("{\"name\":\"$name\",\"status\":\"$status\"}")
done

printf '{"protocol":"golden-magic.executable-json.v1","rows":['
separator=''
for row in "${rows[@]}"; do
  printf '%s%s' "$separator" "$row"
  separator=','
done
printf ']}\n'
