#!/usr/bin/env bash
set -euo pipefail

per_query_limit="${1:-100}"
queries_file="${GOLDEN_MAGIC_CORPUS_QUERIES:-corpus/cli-corpus.queries.txt}"
out="${GOLDEN_MAGIC_CORPUS_OUT:-corpus/cli-tools.seed.json}"
fetched_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
tmpdir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$(dirname "$out")"

if [[ ! -f "$queries_file" ]]; then
  echo "missing corpus query file: $queries_file" >&2
  exit 1
fi

query_count=0
while IFS= read -r raw_query || [[ -n "$raw_query" ]]; do
  query="${raw_query%%#*}"
  query="${query#"${query%%[![:space:]]*}"}"
  query="${query%"${query##*[![:space:]]}"}"
  [[ -z "$query" ]] && continue

  query_count=$((query_count + 1))
  query_out="$tmpdir/query-$query_count.json"
  echo "fetching partition $query_count: $query" >&2

  gh search repos "$query" \
    --sort stars \
    --order desc \
    --limit "$per_query_limit" \
    --json fullName,url,stargazersCount,description,language \
    | jq --arg query "$query" --arg fetched_at "$fetched_at" '
        to_entries
        | map({
            repo: .value.url,
            name: .value.fullName,
            stars: .value.stargazersCount,
            language: (.value.language // ""),
            description: (.value.description // ""),
            cli_evidence: (if ((.value.description // "") | length) > 0 then .value.description else .value.fullName end),
            lifecycle: {
              found: true,
              analyzed: false,
              modeled: false,
              deterministic_tested: false,
              agentic_tested: false
            },
            status: "found",
            descriptor_id: null,
            backend: null,
            deterministic_cases: 0,
            agentic_runs: 0,
            analysis_notes: "",
            source_query: $query,
            source_queries: [$query],
            fetched_at: $fetched_at
          })
      ' > "$query_out"
done < "$queries_file"

if [[ "$query_count" -eq 0 ]]; then
  echo "no usable corpus queries found in $queries_file" >&2
  exit 1
fi

jq -s '
  flatten
  | group_by(.repo)
  | map(
      sort_by(.stars) | reverse | .[0] as $best
      | $best + {
          source_query: ((map(.source_query) | unique)[0]),
          source_queries: (map(.source_query) | unique)
        }
    )
  | sort_by(.stars) | reverse
  | to_entries
  | map(.value + {rank: (.key + 1)})
' "$tmpdir"/query-*.json > "$out"

echo "wrote $(jq length "$out") corpus entries from $query_count partitions to $out"
