use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub matches: MatchRules,
    #[serde(default)]
    pub parser: ParserHint,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRules {
    #[serde(default)]
    pub required_substrings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParserHint {
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub executable: Option<PathBuf>,
    #[serde(default)]
    pub only_rules: Vec<String>,
    #[serde(default)]
    pub disable_rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedDescriptor {
    pub path: PathBuf,
    pub descriptor: Descriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorRegistry {
    descriptors: Vec<LoadedDescriptor>,
}

impl DescriptorRegistry {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self, DescriptorError> {
        let mut descriptors = Vec::new();
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self { descriptors });
        }

        for entry in fs::read_dir(path).map_err(|source| DescriptorError::ReadDir {
            path: path.to_path_buf(),
            source: source.to_string(),
        })? {
            let entry = entry.map_err(|source| DescriptorError::ReadDir {
                path: path.to_path_buf(),
                source: source.to_string(),
            })?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                descriptors.extend(Self::load_dir(&entry_path)?.descriptors);
                continue;
            }

            if !is_descriptor_file(&entry_path) {
                continue;
            }

            descriptors.push(load_descriptor_file(&entry_path)?);
        }

        descriptors.sort_by(|left, right| {
            right
                .descriptor
                .priority
                .cmp(&left.descriptor.priority)
                .then_with(|| left.descriptor.id.cmp(&right.descriptor.id))
        });

        reject_duplicate_ids(&descriptors)?;

        Ok(Self { descriptors })
    }

    pub fn descriptors(&self) -> &[LoadedDescriptor] {
        &self.descriptors
    }

    pub fn select<'a>(&'a self, input: &str) -> Vec<&'a LoadedDescriptor> {
        self.descriptors
            .iter()
            .filter(|loaded| loaded.descriptor.matches_input(input))
            .collect()
    }
}

impl Descriptor {
    pub fn matches_input(&self, input: &str) -> bool {
        self.matches
            .required_substrings
            .iter()
            .all(|needle| input.contains(needle))
    }
}

fn is_descriptor_file(path: &Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
        return false;
    }

    path.file_name().and_then(|name| name.to_str()) != Some("nix.toml")
}

pub fn load_descriptor_file(path: impl AsRef<Path>) -> Result<LoadedDescriptor, DescriptorError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| DescriptorError::ReadFile {
        path: path.to_path_buf(),
        source: source.to_string(),
    })?;
    let descriptor: Descriptor =
        toml::from_str(&text).map_err(|source| DescriptorError::ParseToml {
            path: path.to_path_buf(),
            source: source.to_string(),
        })?;

    if descriptor.id.trim().is_empty() {
        return Err(DescriptorError::InvalidDescriptor {
            path: path.to_path_buf(),
            reason: "descriptor id cannot be empty".to_string(),
        });
    }

    Ok(LoadedDescriptor {
        path: path.to_path_buf(),
        descriptor,
    })
}

fn reject_duplicate_ids(descriptors: &[LoadedDescriptor]) -> Result<(), DescriptorError> {
    let mut by_id: BTreeMap<&str, Vec<PathBuf>> = BTreeMap::new();
    for loaded in descriptors {
        by_id
            .entry(&loaded.descriptor.id)
            .or_default()
            .push(loaded.path.clone());
    }

    let duplicates: BTreeMap<String, Vec<PathBuf>> = by_id
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(id, paths)| (id.to_string(), paths))
        .collect();

    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(DescriptorError::DuplicateIds { duplicates })
    }
}

pub fn descriptor_rule_ids(descriptor: &Descriptor) -> BTreeSet<&str> {
    descriptor
        .parser
        .only_rules
        .iter()
        .chain(descriptor.parser.disable_rules.iter())
        .map(String::as_str)
        .collect()
}

pub fn descriptor_backend_id(descriptor: &Descriptor) -> Option<&str> {
    descriptor.parser.backend.as_deref()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorError {
    ReadDir {
        path: PathBuf,
        source: String,
    },
    ReadFile {
        path: PathBuf,
        source: String,
    },
    ParseToml {
        path: PathBuf,
        source: String,
    },
    InvalidDescriptor {
        path: PathBuf,
        reason: String,
    },
    DuplicateIds {
        duplicates: BTreeMap<String, Vec<PathBuf>>,
    },
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescriptorError::ReadDir { path, source } => {
                write!(
                    formatter,
                    "failed to read descriptor dir {}: {source}",
                    path.display()
                )
            }
            DescriptorError::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read descriptor file {}: {source}",
                    path.display()
                )
            }
            DescriptorError::ParseToml { path, source } => {
                write!(
                    formatter,
                    "failed to parse descriptor file {}: {source}",
                    path.display()
                )
            }
            DescriptorError::InvalidDescriptor { path, reason } => {
                write!(formatter, "invalid descriptor {}: {reason}", path.display())
            }
            DescriptorError::DuplicateIds { duplicates } => {
                let ids = duplicates.keys().cloned().collect::<Vec<_>>().join(", ");
                write!(formatter, "duplicate descriptor id(s): {ids}")
            }
        }
    }
}

impl std::error::Error for DescriptorError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_toml_descriptors_sorted_by_priority() {
        let dir = tempdir().expect("temp dir");
        fs::write(
            dir.path().join("low.toml"),
            r#"
id = "low"
name = "Low"
priority = 1
[matches]
required_substrings = ["alpha"]
"#,
        )
        .expect("write low descriptor");
        fs::write(
            dir.path().join("high.toml"),
            r#"
id = "high"
name = "High"
priority = 10
[matches]
required_substrings = ["alpha"]
"#,
        )
        .expect("write high descriptor");

        let registry = DescriptorRegistry::load_dir(dir.path()).expect("registry loads");

        assert_eq!(registry.descriptors()[0].descriptor.id, "high");
        assert_eq!(registry.descriptors()[1].descriptor.id, "low");
    }

    #[test]
    fn ignores_nix_fixture_manifests_in_descriptor_dirs() {
        let dir = tempdir().expect("temp dir");
        fs::write(
            dir.path().join("descriptor.toml"),
            r#"
id = "example"
name = "Example"
"#,
        )
        .expect("write descriptor");
        fs::write(
            dir.path().join("nix.toml"),
            r#"
packages = ["nixpkgs#coreutils"]
command = "printf 'alpha|beta\n'"
"#,
        )
        .expect("write nix manifest");

        let registry = DescriptorRegistry::load_dir(dir.path()).expect("registry loads");

        assert_eq!(registry.descriptors().len(), 1);
        assert_eq!(registry.descriptors()[0].descriptor.id, "example");
    }

    #[test]
    fn detects_duplicate_descriptor_ids() {
        let dir = tempdir().expect("temp dir");
        for name in ["one.toml", "two.toml"] {
            fs::write(
                dir.path().join(name),
                r#"
id = "same"
name = "Duplicate"
"#,
            )
            .expect("write descriptor");
        }

        let error = DescriptorRegistry::load_dir(dir.path()).expect_err("duplicates fail");

        assert!(matches!(error, DescriptorError::DuplicateIds { .. }));
    }

    #[test]
    fn selects_matching_descriptors() {
        let dir = tempdir().expect("temp dir");
        fs::write(
            dir.path().join("match.toml"),
            r#"
id = "matching"
name = "Matching"
priority = 1
[matches]
required_substrings = ["alpha", "omega"]
"#,
        )
        .expect("write matching descriptor");

        let registry = DescriptorRegistry::load_dir(dir.path()).expect("registry loads");

        assert_eq!(registry.select("alpha and omega").len(), 1);
        assert!(registry.select("alpha only").is_empty());
    }
}
