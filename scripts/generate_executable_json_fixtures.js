#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const seedPath = path.join(repoRoot, "corpus", "generated-executable-fixtures.seed.json");
const fixtureRoot = path.join(repoRoot, "tests", "fixtures", "descriptors");
const specs = JSON.parse(fs.readFileSync(seedPath, "utf8"));

const pluginSource = `#!/usr/bin/env python3
import json
import re
import sys

PARSER = "__PARSER__"

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
        parts = line.split("\\t", 1)
        if len(parts) == 2:
            rows.append({"name": parts[0], "value": parts[1]})
    return rows

def ls_long(text):
    rows = []
    pattern = re.compile(r"^(?P<mode>[.d][rwx-]{9})\\s+(?P<size>\\S+)\\s+(?P<owner>\\S+)\\s+(?P<day>\\d+)\\s+(?P<month>\\S+)\\s+(?P<time>\\S+)\\s+(?P<path>.+)$")
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
`;

function expectedRows(spec) {
  const lines = spec.input.split(/\n/).filter(Boolean);
  if (spec.parser === "colon-kv") {
    return lines
      .filter((line) => line.includes(": "))
      .map((line) => {
        const [key, ...rest] = line.split(": ");
        return { key, value: rest.join(": ") };
      });
  }
  if (spec.parser === "tab-pairs") {
    return lines
      .filter((line) => line.includes("\t"))
      .map((line) => {
        const [name, value] = line.split("\t", 2);
        return { name, value };
      });
  }
  if (spec.parser === "ls-long") {
    const pattern = /^(?<mode>[.d][rwx-]{9})\s+(?<size>\S+)\s+(?<owner>\S+)\s+(?<day>\d+)\s+(?<month>\S+)\s+(?<time>\S+)\s+(?<path>.+)$/;
    return lines.map((line) => line.match(pattern)?.groups).filter(Boolean);
  }
  throw new Error(`unsupported parser ${spec.parser}`);
}

for (const spec of specs) {
  const dir = path.join(fixtureRoot, spec.fixture);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, "descriptor.toml"),
    `id = "${spec.descriptor_id}"
name = "${spec.name}"
priority = 50

[matches]
required_substrings = ${JSON.stringify(spec.required_substrings)}

[parser]
backend = "executable-json"
executable = "./parser-plugin.py"
`,
  );
  fs.writeFileSync(path.join(dir, "input.txt"), spec.input);
  fs.writeFileSync(path.join(dir, "negative.txt"), spec.negative);
  fs.writeFileSync(path.join(dir, "expected.rows.json"), `${JSON.stringify(expectedRows(spec), null, 2)}\n`);
  fs.writeFileSync(path.join(dir, "parser-plugin.py"), pluginSource.replace("__PARSER__", spec.parser), {
    mode: 0o755,
  });
}

console.error(`generated ${specs.length} executable-json fixture(s)`);
