use golden_magic::{HeaderMode, ParseOptions, parse, parse_with_options};
use std::time::{Duration, Instant};

const LARGE_TSV_BUDGET: Duration = Duration::from_millis(250);
const HEADER_TSV_BUDGET: Duration = Duration::from_millis(100);

#[test]
fn large_rectangular_tsv_stays_under_interactive_budget() {
    let input = rectangular_tsv(10_000, 8);

    let elapsed = time_once(|| {
        let report = parse(&input);
        assert_eq!(report.rows.len(), 10_000);
        assert_eq!(report.columns.len(), 8);
    });

    assert!(
        elapsed <= LARGE_TSV_BUDGET,
        "large TSV parse exceeded budget: elapsed={elapsed:?}, budget={LARGE_TSV_BUDGET:?}"
    );
}

#[test]
fn first_row_headers_stay_under_interactive_budget() {
    let input = format!("name\tstatus\n{}", rectangular_tsv(1_000, 2));
    let options = ParseOptions::new().header_mode(HeaderMode::FirstRow);

    let elapsed = time_once(|| {
        let report = parse_with_options(&input, &options);
        assert_eq!(report.rows.len(), 1_000);
        assert_eq!(report.columns, vec!["name", "status"]);
    });

    assert!(
        elapsed <= HEADER_TSV_BUDGET,
        "first-row header parse exceeded budget: elapsed={elapsed:?}, budget={HEADER_TSV_BUDGET:?}"
    );
}

fn time_once(work: impl FnOnce()) -> Duration {
    let started = Instant::now();
    work();
    started.elapsed()
}

fn rectangular_tsv(rows: usize, columns: usize) -> String {
    (0..rows)
        .map(|row| {
            (0..columns)
                .map(|column| format!("cell-{row}-{column}"))
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
