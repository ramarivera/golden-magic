use criterion::{Criterion, criterion_group, criterion_main};
use golden_magic::{HeaderMode, ParseOptions, parse, parse_with_options};

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

fn bench_parser(c: &mut Criterion) {
    let medium_tsv = rectangular_tsv(1_000, 8);
    let large_tsv = rectangular_tsv(10_000, 8);
    let first_row_headers = format!("name\tstatus\n{}", rectangular_tsv(1_000, 2));
    let first_row_options = ParseOptions::new().header_mode(HeaderMode::FirstRow);

    c.bench_function("parse medium rectangular tsv", |b| {
        b.iter(|| parse(&medium_tsv));
    });

    c.bench_function("parse large rectangular tsv", |b| {
        b.iter(|| parse(&large_tsv));
    });

    c.bench_function("parse first-row headers", |b| {
        b.iter(|| parse_with_options(&first_row_headers, &first_row_options));
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
