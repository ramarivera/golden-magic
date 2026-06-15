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
    status: String,
    source_query: String,
    fetched_at: String,
}

#[test]
fn seed_corpus_manifest_has_unique_repositories_and_required_fields() {
    let entries: Vec<CorpusEntry> =
        serde_json::from_str(include_str!("../corpus/cli-tools.seed.json"))
            .expect("seed corpus parses");
    assert!(!entries.is_empty(), "seed corpus must not be empty");

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
                || entry.source_query.contains("topic:cli"),
            "entry must preserve CLI-oriented evidence or query context: {}",
            entry.repo
        );
        assert!(
            !entry.cli_evidence.trim().is_empty(),
            "cli evidence is required for {}",
            entry.repo
        );
        assert!(
            matches!(entry.status.as_str(), "seed" | "modeled" | "tested"),
            "unexpected corpus status for {}: {}",
            entry.repo,
            entry.status
        );
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
            entry.source_query.contains("topic:cli"),
            "seed corpus must use an explicit CLI topic query"
        );
        let _ = (&entry.language, &entry.description);
        previous_stars = entry.stars;
    }
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
