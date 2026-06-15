use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub descriptor_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub tool_pack_dirs: Vec<PathBuf>,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
        path: path.to_path_buf(),
        source: source.to_string(),
    })?;
    toml::from_str(&text).map_err(|source| ConfigError::ParseToml {
        path: path.to_path_buf(),
        source: source.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    ReadFile { path: PathBuf, source: String },
    ParseToml { path: PathBuf, source: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read config file {}: {source}",
                    path.display()
                )
            }
            ConfigError::ParseToml { path, source } => {
                write!(
                    formatter,
                    "failed to parse config file {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_config_dirs_from_toml() {
        let dir = tempdir().expect("temp dir");
        let config_path = dir.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
descriptor_dirs = ["/tmp/one", "/tmp/two"]
tool_pack_dirs = ["/tmp/tools"]
"#,
        )
        .expect("write config");

        let config = load_config(&config_path).expect("config loads");

        assert_eq!(
            config.descriptor_dirs,
            vec![PathBuf::from("/tmp/one"), PathBuf::from("/tmp/two")]
        );
        assert_eq!(config.tool_pack_dirs, vec![PathBuf::from("/tmp/tools")]);
    }

    #[test]
    fn rejects_unknown_fields() {
        let dir = tempdir().expect("temp dir");
        let config_path = dir.path().join("config.toml");
        fs::write(&config_path, "mystery = true\n").expect("write config");

        let error = load_config(&config_path).expect_err("unknown fields fail");

        assert!(matches!(error, ConfigError::ParseToml { .. }));
    }
}
