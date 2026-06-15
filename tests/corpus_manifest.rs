use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    rank: usize,
    repo: String,
    name: String,
    stars: u64,
    language: String,
    description: String,
    cli_evidence: String,
    lifecycle: CorpusLifecycle,
    status: String,
    descriptor_id: Option<String>,
    backend: Option<String>,
    deterministic_cases: usize,
    agentic_runs: usize,
    analysis_notes: String,
    source_query: String,
    source_queries: Vec<String>,
    fetched_at: String,
}

#[derive(Debug, Deserialize)]
struct CorpusLifecycle {
    found: bool,
    analyzed: bool,
    modeled: bool,
    deterministic_tested: bool,
    agentic_tested: bool,
}

fn is_cli_oriented_query(query: &str) -> bool {
    [
        "topic:cli",
        "topic:command-line",
        "topic:terminal",
        "topic:tui",
        "topic:shell",
        "topic:command-line-tool",
    ]
    .iter()
    .any(|needle| query.contains(needle))
}

#[test]
fn seed_corpus_manifest_has_unique_repositories_and_required_fields() {
    let entries: Vec<CorpusEntry> =
        serde_json::from_str(include_str!("../corpus/cli-tools.seed.json"))
            .expect("seed corpus parses");
    assert!(!entries.is_empty(), "seed corpus must not be empty");
    assert!(
        entries.len() >= 400,
        "partitioned seed corpus should not regress to a single search page"
    );

    let mut repos = BTreeSet::new();
    let mut previous_stars = u64::MAX;
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry.rank, index + 1, "ranks must be contiguous");
        assert!(
            entry.repo.starts_with("https://github.com/"),
            "repo must be a GitHub URL: {}",
            entry.repo
        );
        assert!(!entry.name.trim().is_empty(), "name is required");
        assert!(entry.stars > 0, "stars must be captured for {}", entry.repo);
        assert!(
            repos.insert(&entry.repo),
            "repo appears more than once: {}",
            entry.repo
        );
        assert!(
            entry.cli_evidence.contains("cli")
                || entry.cli_evidence.contains("command")
                || entry.cli_evidence.contains("terminal")
                || entry
                    .source_queries
                    .iter()
                    .any(|query| is_cli_oriented_query(query)),
            "entry must preserve CLI-oriented evidence or query context: {}",
            entry.repo
        );
        assert!(
            !entry.cli_evidence.trim().is_empty(),
            "cli evidence is required for {}",
            entry.repo
        );
        assert_lifecycle_is_consistent(entry);
        assert!(
            entry.stars <= previous_stars,
            "corpus must be sorted by descending stars"
        );
        assert!(
            !entry.fetched_at.trim().is_empty(),
            "fetched_at is required for {}",
            entry.repo
        );
        assert!(
            !entry.source_queries.is_empty(),
            "seed corpus must preserve at least one source query for {}",
            entry.repo
        );
        assert!(
            entry
                .source_queries
                .iter()
                .any(|query| query == &entry.source_query),
            "source_query must be one of source_queries for {}",
            entry.repo
        );
        assert!(
            entry
                .source_queries
                .iter()
                .all(|query| is_cli_oriented_query(query)),
            "seed corpus queries must stay CLI/tool oriented for {}: {:?}",
            entry.repo,
            entry.source_queries
        );
        let _ = (&entry.language, &entry.description);
        previous_stars = entry.stars;
    }
}

#[test]
fn seed_corpus_lifecycle_counts_are_explicitly_incomplete() {
    let entries: Vec<CorpusEntry> =
        serde_json::from_str(include_str!("../corpus/cli-tools.seed.json"))
            .expect("seed corpus parses");

    let found = entries.iter().filter(|entry| entry.lifecycle.found).count();
    let analyzed = entries
        .iter()
        .filter(|entry| entry.lifecycle.analyzed)
        .count();
    let modeled = entries
        .iter()
        .filter(|entry| entry.lifecycle.modeled)
        .count();
    let deterministic = entries
        .iter()
        .filter(|entry| entry.lifecycle.deterministic_tested)
        .count();
    let agentic = entries
        .iter()
        .filter(|entry| entry.lifecycle.agentic_tested)
        .count();

    assert_eq!(found, entries.len(), "every seed entry is at least found");
    assert_eq!(analyzed, 0, "seed fetch alone must not claim analysis");
    assert_eq!(modeled, 0, "seed fetch alone must not claim modeling");
    assert_eq!(
        deterministic, 0,
        "seed fetch alone must not claim deterministic tool tests"
    );
    assert_eq!(
        agentic, 0,
        "seed fetch alone must not claim agentic tool tests"
    );
}

#[test]
fn full_corpus_completion_requires_ten_thousand_entries() {
    let entries: Vec<CorpusEntry> =
        serde_json::from_str(include_str!("../corpus/cli-tools.seed.json"))
            .expect("seed corpus parses");

    assert!(
        entries.len() < 10_000,
        "rename this test when corpus/cli-tools.seed.json becomes the full 10k corpus"
    );
}

fn assert_lifecycle_is_consistent(entry: &CorpusEntry) {
    assert!(entry.lifecycle.found, "{} must be marked found", entry.repo);

    let expected_status = if entry.lifecycle.agentic_tested {
        "agentic-tested"
    } else if entry.lifecycle.deterministic_tested {
        "deterministic-tested"
    } else if entry.lifecycle.modeled {
        "modeled"
    } else if entry.lifecycle.analyzed {
        "analyzed"
    } else {
        "found"
    };
    assert_eq!(
        entry.status, expected_status,
        "{} status must match lifecycle",
        entry.repo
    );

    assert!(
        !entry.lifecycle.modeled || entry.lifecycle.analyzed,
        "{} cannot be modeled before analysis",
        entry.repo
    );
    assert!(
        !entry.lifecycle.deterministic_tested || entry.lifecycle.modeled,
        "{} cannot be deterministically tested before modeling",
        entry.repo
    );
    assert!(
        !entry.lifecycle.agentic_tested || entry.lifecycle.deterministic_tested,
        "{} cannot be agentic-tested before deterministic tests",
        entry.repo
    );

    if entry.lifecycle.modeled {
        assert!(
            entry
                .descriptor_id
                .as_deref()
                .is_some_and(|id| !id.is_empty()),
            "{} modeled entries need descriptor_id",
            entry.repo
        );
        assert!(
            entry
                .backend
                .as_deref()
                .is_some_and(|backend| !backend.is_empty()),
            "{} modeled entries need backend",
            entry.repo
        );
    } else {
        assert!(
            entry.descriptor_id.is_none(),
            "{} unmodeled entries must not claim descriptor_id",
            entry.repo
        );
        assert!(
            entry.backend.is_none(),
            "{} unmodeled entries must not claim backend",
            entry.repo
        );
    }

    assert_eq!(
        entry.deterministic_cases > 0,
        entry.lifecycle.deterministic_tested,
        "{} deterministic_cases must match deterministic_tested",
        entry.repo
    );
    assert_eq!(
        entry.agentic_runs > 0,
        entry.lifecycle.agentic_tested,
        "{} agentic_runs must match agentic_tested",
        entry.repo
    );

    if entry.lifecycle.analyzed {
        assert!(
            !entry.analysis_notes.trim().is_empty(),
            "{} analyzed entries need notes",
            entry.repo
        );
    }
}
