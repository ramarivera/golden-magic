#!/usr/bin/env python3
import json
import re
import sys

PARSER = "tab-pairs"

def colon_kv(text):
    rows = []
    for line in text.splitlines():
        if ": " not in line:
            continue
        key, value = line.split(": ", 1)
        rows.append({"key": key, "value": value})
    return rows

def tab_pairs(text):
    rows = []
    for line in text.splitlines():
        parts = line.split("\t", 1)
        if len(parts) == 2:
            rows.append({"name": parts[0], "value": parts[1]})
    return rows

def ls_long(text):
    rows = []
    pattern = re.compile(r"^(?P<mode>[.d][rwx-]{9})\s+(?P<size>\S+)\s+(?P<owner>\S+)\s+(?P<day>\d+)\s+(?P<month>\S+)\s+(?P<time>\S+)\s+(?P<path>.+)$")
    for line in text.splitlines():
        match = pattern.match(line)
        if match:
            rows.append(match.groupdict())
    return rows

text = sys.stdin.read()
if PARSER == "colon-kv":
    rows = colon_kv(text)
elif PARSER == "tab-pairs":
    rows = tab_pairs(text)
elif PARSER == "ls-long":
    rows = ls_long(text)
else:
    raise SystemExit(f"unsupported generated parser: {PARSER}")

print(json.dumps({"protocol": "golden-magic.executable-json.v1", "rows": rows}, separators=(",", ":")))
