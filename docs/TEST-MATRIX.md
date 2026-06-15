# Test Matrix

Golden Magic has a generated deterministic integration matrix for the explicit 2,000+ test-case target.

Run it with:

```bash
cargo test --test generated_matrix -- --nocapture
```

The harness fails if fewer than 2,000 generated cases execute.

Current generated case budget:

| Surface | Cases |
| --- | ---: |
| delimiter parsing | 900 |
| first-row headers | 350 |
| fallback lines | 300 |
| sections backend | 300 |
| tree-sitter Rust backend | 300 |
| tool-pack loader | 250 |
| total | 2,400 |

This does not complete the 10,000-tool corpus target. It makes the test-count requirement measurable and keeps it from being silently reduced.
