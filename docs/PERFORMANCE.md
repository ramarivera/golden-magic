# Golden Magic Performance

Golden Magic includes a Criterion benchmark suite for parser hot paths.

Run:

```bash
cargo bench --bench parser
```

Fast local smoke run:

```bash
cargo bench --bench parser -- --sample-size 10
```

## Current Benchmarks

Initial local baseline from 2026-06-14 on Ramiro's machine, using `--sample-size 10`:

| Benchmark | Time |
| --- | ---: |
| `parse medium rectangular tsv` (1,000 × 8) | ~0.98 ms |
| `parse large rectangular tsv` (10,000 × 8) | ~9.66 ms |
| `parse first-row headers` (1,000 × 2) | ~0.38 ms |

2026-06-15 audit run, using `cargo bench --bench parser -- --sample-size 10`:

| Benchmark | Time | Criterion comparison |
| --- | ---: | --- |
| `parse medium rectangular tsv` (1,000 × 8) | ~1.02 ms | small regression against local untracked baseline |
| `parse large rectangular tsv` (10,000 × 8) | ~9.57 ms | improved against local untracked baseline |
| `parse first-row headers` (1,000 × 2) | ~0.37 ms | improved against local untracked baseline |

Criterion baselines live under untracked `target/criterion`, so local "regressed" output is advisory unless paired with a checked-in baseline policy or a hard test budget. Do not update benchmark baselines just to silence output; record the observed tradeoff or optimize first.

## Hard Gates

`tests/performance_gate.rs` enforces conservative regression budgets during `cargo test`:

| Gate | Budget |
| --- | ---: |
| large rectangular TSV parse (10,000 × 8) | 250 ms |
| first-row header parse (1,000 × 2) | 100 ms |

These budgets are intentionally much looser than the Criterion baselines. They are tripwires for accidental regressions, not microbenchmark claims.

Any regression beyond the budget should require either an optimization or a documented tradeoff.

## Future Gates

Additional budgets still needed:

- interactive CLI startup latency
- descriptor registry selection under large registries
- Nushell adapter overhead
- full fixture harness execution
