#!/usr/bin/env bash
set -euo pipefail

limit="${1:-100}"
query="${GOLDEN_MAGIC_CORPUS_QUERY:-topic:cli stars:>1000}"
out="${GOLDEN_MAGIC_CORPUS_OUT:-corpus/cli-tools.seed.json}"
fetched_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

mkdir -p "$(dirname "$out")"

gh search repos "$query" \
  --sort stars \
  --order desc \
  --limit "$limit" \
  --json fullName,url,stargazersCount,description,language \
  | jq --arg query "$query" --arg fetched_at "$fetched_at" '
      to_entries
      | map({
          rank: (.key + 1),
          repo: .value.url,
          name: .value.fullName,
          stars: .value.stargazersCount,
          language: (.value.language // ""),
          description: (.value.description // ""),
          cli_evidence: (.value.description // .value.fullName),
          status: "seed",
          source_query: $query,
          fetched_at: $fetched_at
        })
    ' > "$out"

echo "wrote $(jq length "$out") corpus entries to $out"
