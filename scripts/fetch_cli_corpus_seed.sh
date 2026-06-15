#!/usr/bin/env bash
set -euo pipefail

per_query_limit="${1:-100}"
queries_file="${GOLDEN_MAGIC_CORPUS_QUERIES:-corpus/cli-corpus.queries.txt}"
out="${GOLDEN_MAGIC_CORPUS_OUT:-corpus/cli-tools.seed.json}"
fetch_sleep_seconds="${GOLDEN_MAGIC_CORPUS_FETCH_SLEEP_SECONDS:-2}"
fetch_retries="${GOLDEN_MAGIC_CORPUS_FETCH_RETRIES:-2}"
cache_dir="${GOLDEN_MAGIC_CORPUS_CACHE_DIR:-corpus/.cache/github-search}"
cache_refresh="${GOLDEN_MAGIC_CORPUS_CACHE_REFRESH:-0}"
fetched_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
tmpdir="$(mktemp -d)"
existing="$tmpdir/existing.json"

cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$(dirname "$out")"
mkdir -p "$cache_dir"
if [[ -f "$out" ]]; then
  cp "$out" "$existing"
else
  printf '[]\n' > "$existing"
fi

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
  cache_key="$(printf '%s\n%s\n' "$per_query_limit" "$query" | shasum -a 256 | awk '{print $1}')"
  cache_raw="$cache_dir/$cache_key.raw.json"
  cache_meta="$cache_dir/$cache_key.query.txt"
  read -r -a query_terms <<< "$query"

  if [[ "$cache_refresh" != "1" && -f "$cache_raw" ]]; then
    echo "using cached partition $query_count: $query" >&2
    cp "$cache_raw" "$query_out.raw"
  else
    echo "fetching partition $query_count: $query" >&2
    attempt=0
    until gh search repos "${query_terms[@]}" \
        --sort stars \
        --order desc \
        --limit "$per_query_limit" \
        --json fullName,url,stargazersCount,description,language > "$query_out.raw"; do
      attempt=$((attempt + 1))
      if [[ "$attempt" -gt "$fetch_retries" ]]; then
        echo "failed partition after $fetch_retries retries: $query" >&2
        exit 1
      fi
      sleep_for=$((fetch_sleep_seconds * attempt))
      echo "retrying partition $query_count after ${sleep_for}s: $query" >&2
      sleep "$sleep_for"
    done
    cp "$query_out.raw" "$cache_raw"
    printf 'limit=%s\nquery=%s\n' "$per_query_limit" "$query" > "$cache_meta"
  fi

  jq --arg query "$query" --arg fetched_at "$fetched_at" '
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
      ' "$query_out.raw" > "$query_out"
done < "$queries_file"

if [[ "$query_count" -eq 0 ]]; then
  echo "no usable corpus queries found in $queries_file" >&2
  exit 1
fi

jq -s --slurpfile existing "$existing" '
  flatten as $fresh
  | ($existing[0] // []) as $existing_entries
  | ($existing_entries
      | map({
          key: .repo,
          value: {
            lifecycle,
            status,
            descriptor_id,
            backend,
            deterministic_cases,
            agentic_runs,
            analysis_notes
          }
        })
      | from_entries) as $existing_by_repo
  | $fresh
  | group_by(.repo)
  | map(
      sort_by(.stars) | reverse | .[0] as $best
      | ($existing_by_repo[$best.repo] // {}) as $previous
      | $best + {
          source_query: ((map(.source_query) | unique)[0]),
          source_queries: (map(.source_query) | unique)
        }
      | . + $previous
    )
  | sort_by(.stars) | reverse
  | to_entries
  | map(.value + {rank: (.key + 1)})
' "$tmpdir"/query-*.json > "$out"

echo "wrote $(jq length "$out") corpus entries from $query_count partitions to $out"
