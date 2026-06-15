mod support;

use golden_magic::descriptors::DescriptorRegistry;
use std::fs;
use std::path::Path;
use support::DescriptorFixture;
use tempfile::tempdir;

#[test]
fn descriptor_fixtures_match_expected_rows() {
    for fixture in DescriptorFixture::all() {
        fixture.assert_rows_match();
    }
}

#[test]
fn descriptor_fixtures_apply_parser_backend_hints() {
    let fixture = DescriptorFixture::all()
        .into_iter()
        .find(|fixture| fixture.path().ends_with("generic-pipes"))
        .expect("generic pipes fixture exists");
    let report = fixture.parse_report();

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
    for fixture in DescriptorFixture::all() {
        fixture.assert_negative_does_not_match();
    }
}

#[test]
fn descriptor_fixture_utility_loads_each_fixture_in_isolation() {
    for fixture in DescriptorFixture::all() {
        let descriptor = fixture.selected_descriptor();
        assert!(
            descriptor.id.contains('.') || descriptor.id.starts_with("known."),
            "fixture {:?} should expose a stable descriptor-ish id: {}",
            fixture.path(),
            descriptor.id
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
