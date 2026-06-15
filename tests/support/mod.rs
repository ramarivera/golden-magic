use golden_magic::descriptors::{Descriptor, DescriptorRegistry};
use golden_magic::{ParseOptions, ParseReport, parse_with_options};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};

#[derive(Debug, Clone)]
pub struct DescriptorFixture {
    path: PathBuf,
}

impl DescriptorFixture {
    pub fn all() -> Vec<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/descriptors");
        let mut fixtures = fs::read_dir(root)
            .expect("descriptor fixture root exists")
            .map(|entry| Self {
                path: entry.expect("fixture entry").path(),
            })
            .filter(|fixture| fixture.path.is_dir())
            .collect::<Vec<_>>();
        fixtures.sort_by(|left, right| left.path.cmp(&right.path));
        fixtures
    }

    pub fn with_nix_manifest() -> Vec<Self> {
        Self::all()
            .into_iter()
            .filter(|fixture| fixture.path.join("nix.toml").exists())
            .collect()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn input(&self) -> String {
        self.read_text("input.txt")
    }

    pub fn negative_input(&self) -> String {
        self.read_text("negative.txt")
    }

    pub fn expected_rows(&self) -> Value {
        self.read_json("expected.rows.json")
    }

    pub fn read_text(&self, relative_path: &str) -> String {
        fs::read_to_string(self.path.join(relative_path)).unwrap_or_else(|error| {
            panic!("fixture {:?} missing {relative_path}: {error}", self.path)
        })
    }

    pub fn read_json<T: DeserializeOwned>(&self, relative_path: &str) -> T {
        serde_json::from_str(&self.read_text(relative_path)).unwrap_or_else(|error| {
            panic!(
                "fixture {:?} has invalid JSON in {relative_path}: {error}",
                self.path
            )
        })
    }

    pub fn read_toml<T: DeserializeOwned>(&self, relative_path: &str) -> T {
        toml::from_str(&self.read_text(relative_path)).unwrap_or_else(|error| {
            panic!(
                "fixture {:?} has invalid TOML in {relative_path}: {error}",
                self.path
            )
        })
    }

    pub fn isolated_registry(&self) -> IsolatedDescriptorRegistry {
        let dir = tempdir().expect("temp descriptor dir");
        fs::copy(
            self.path.join("descriptor.toml"),
            dir.path().join("descriptor.toml"),
        )
        .unwrap_or_else(|error| panic!("fixture {:?} descriptor copy failed: {error}", self.path));
        let registry = DescriptorRegistry::load_dir(dir.path())
            .unwrap_or_else(|error| panic!("fixture {:?} registry failed: {error}", self.path));

        IsolatedDescriptorRegistry { dir, registry }
    }

    pub fn selected_descriptor(&self) -> Descriptor {
        let input = self.input();
        let isolated = self.isolated_registry();
        let selected = isolated.registry.select(&input);
        assert_eq!(
            selected.len(),
            1,
            "fixture {:?} should match exactly once",
            self.path
        );
        selected[0].descriptor.clone()
    }

    pub fn parse_report(&self) -> ParseReport {
        let input = self.input();
        let descriptor = self.selected_descriptor();
        parse_with_options(&input, &parse_options_from_descriptor(&descriptor))
    }

    pub fn assert_rows_match(&self) {
        let actual = serde_json::to_value(&self.parse_report().rows).expect("rows serialize");
        assert_eq!(
            actual,
            self.expected_rows(),
            "fixture {:?} parsed rows differ",
            self.path
        );
    }

    pub fn assert_negative_does_not_match(&self) {
        let isolated = self.isolated_registry();
        assert!(
            isolated.registry.select(&self.negative_input()).is_empty(),
            "fixture {:?} negative input should not match",
            self.path
        );
    }
}

pub struct IsolatedDescriptorRegistry {
    #[allow(dead_code)]
    dir: TempDir,
    pub registry: DescriptorRegistry,
}

pub fn parse_options_from_descriptor(descriptor: &Descriptor) -> ParseOptions {
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

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
