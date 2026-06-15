use golden_magic::descriptors::{Descriptor, DescriptorRegistry};
use golden_magic::{ParseOptions, parse_with_options};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn descriptor_fixtures_match_expected_rows() {
    for fixture in descriptor_fixtures() {
        let descriptor_dir = tempdir().expect("temp descriptor dir");
        fs::copy(
            fixture.join("descriptor.toml"),
            descriptor_dir.path().join("descriptor.toml"),
        )
        .expect("copy descriptor");

        let registry = DescriptorRegistry::load_dir(descriptor_dir.path()).expect("registry loads");
        let input = fs::read_to_string(fixture.join("input.txt")).expect("input fixture exists");
        let selected = registry.select(&input);
        assert_eq!(selected.len(), 1, "fixture {fixture:?} should match once");

        let descriptor = &selected[0].descriptor;
        let options = options_from_descriptor(descriptor);

        let report = parse_with_options(&input, &options);
        let expected: Value = serde_json::from_str(
            &fs::read_to_string(fixture.join("expected.rows.json")).expect("expected rows exist"),
        )
        .expect("expected rows are valid JSON");
        let actual = serde_json::to_value(&report.rows).expect("rows serialize");

        assert_eq!(actual, expected, "fixture {fixture:?} parsed rows differ");
    }
}

#[test]
fn descriptor_fixtures_apply_parser_backend_hints() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/descriptors/generic-pipes");
    let descriptor_dir = tempdir().expect("temp descriptor dir");
    fs::copy(
        fixture.join("descriptor.toml"),
        descriptor_dir.path().join("descriptor.toml"),
    )
    .expect("copy descriptor");

    let registry = DescriptorRegistry::load_dir(descriptor_dir.path()).expect("registry loads");
    let input = fs::read_to_string(fixture.join("input.txt")).expect("input fixture exists");
    let selected = registry.select(&input);
    assert_eq!(selected.len(), 1, "generic pipes fixture should match once");

    let report = parse_with_options(&input, &options_from_descriptor(&selected[0].descriptor));

    assert!(
        report
            .trace
            .iter()
            .any(|event| event.rule_id == "backend.heuristic"),
        "descriptor backend hint should be applied by fixture harness"
    );
}

#[test]
fn descriptor_fixtures_include_negative_inputs() {
    for fixture in descriptor_fixtures() {
        let descriptor_dir = tempdir().expect("temp descriptor dir");
        fs::copy(
            fixture.join("descriptor.toml"),
            descriptor_dir.path().join("descriptor.toml"),
        )
        .expect("copy descriptor");

        let registry = DescriptorRegistry::load_dir(descriptor_dir.path()).expect("registry loads");
        let negative =
            fs::read_to_string(fixture.join("negative.txt")).expect("negative fixture exists");

        assert!(
            registry.select(&negative).is_empty(),
            "fixture {fixture:?} negative input should not match"
        );
    }
}

#[test]
fn full_registry_rejects_duplicate_fixture_ids() {
    let registry_dir = tempdir().expect("temp registry dir");
    let first = registry_dir.path().join("first.toml");
    let second = registry_dir.path().join("second.toml");
    let descriptor = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/descriptors/generic-pipes/descriptor.toml"),
    )
    .expect("fixture descriptor exists");

    fs::write(first, &descriptor).expect("write first descriptor");
    fs::write(second, descriptor).expect("write second descriptor");

    let error = DescriptorRegistry::load_dir(registry_dir.path()).expect_err("duplicates fail");

    assert!(error.to_string().contains("duplicate descriptor id"));
}

fn descriptor_fixtures() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/descriptors");
    fs::read_dir(root)
        .expect("descriptor fixture root exists")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| path.is_dir())
        .collect()
}

fn options_from_descriptor(descriptor: &Descriptor) -> ParseOptions {
    let mut options = ParseOptions::new();
    if let Some(backend) = &descriptor.parser.backend {
        options = options.backend(backend);
    }
    for rule in &descriptor.parser.only_rules {
        options = options.only_rule(rule);
    }
    for rule in &descriptor.parser.disable_rules {
        options = options.disable_rule(rule);
    }

    options
}
