use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPack {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub commands: Vec<ToolCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCommand {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub descriptor: Option<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub args: Vec<ToolArg>,
    #[serde(default)]
    pub subcommands: Vec<ToolCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolArg {
    pub name: String,
    pub kind: ToolArgKind,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub affects_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolArgKind {
    Flag,
    Option,
    Positional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedToolPack {
    pub path: PathBuf,
    pub pack: ToolPack,
}

impl ToolPack {
    pub fn descriptor_refs(&self) -> Vec<&str> {
        let mut refs = Vec::new();
        for command in &self.commands {
            collect_descriptor_refs(command, &mut refs);
        }
        refs
    }
}

pub fn load_tool_pack_file(path: impl AsRef<Path>) -> Result<LoadedToolPack, ToolPackError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| ToolPackError::ReadFile {
        path: path.to_path_buf(),
        source: source.to_string(),
    })?;
    let pack: ToolPack = toml::from_str(&text).map_err(|source| ToolPackError::ParseToml {
        path: path.to_path_buf(),
        source: source.to_string(),
    })?;

    validate_tool_pack(path, &pack)?;

    Ok(LoadedToolPack {
        path: path.to_path_buf(),
        pack,
    })
}

pub fn load_tool_packs_dir(path: impl AsRef<Path>) -> Result<Vec<LoadedToolPack>, ToolPackError> {
    let path = path.as_ref();
    let mut packs = Vec::new();

    if !path.exists() {
        return Ok(packs);
    }

    for entry in fs::read_dir(path).map_err(|source| ToolPackError::ReadDir {
        path: path.to_path_buf(),
        source: source.to_string(),
    })? {
        let entry = entry.map_err(|source| ToolPackError::ReadDir {
            path: path.to_path_buf(),
            source: source.to_string(),
        })?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            packs.extend(load_tool_packs_dir(entry_path)?);
            continue;
        }

        if entry_path.file_name().and_then(|name| name.to_str()) == Some("tool.toml") {
            packs.push(load_tool_pack_file(entry_path)?);
        }
    }

    packs.sort_by(|left, right| left.pack.id.cmp(&right.pack.id));
    Ok(packs)
}

fn collect_descriptor_refs<'a>(command: &'a ToolCommand, refs: &mut Vec<&'a str>) {
    if let Some(descriptor) = &command.descriptor {
        refs.push(descriptor);
    }
    for subcommand in &command.subcommands {
        collect_descriptor_refs(subcommand, refs);
    }
}

fn validate_tool_pack(path: &Path, pack: &ToolPack) -> Result<(), ToolPackError> {
    if !pack.id.starts_with("tool.") {
        return Err(invalid(path, "tool pack id must start with tool."));
    }
    if pack.name.trim().is_empty() {
        return Err(invalid(path, "tool pack name cannot be empty"));
    }
    if pack.version.parse::<u64>().is_err() {
        return Err(invalid(
            path,
            "tool pack version must be a positive integer string",
        ));
    }
    for command in &pack.commands {
        validate_command(path, command)?;
    }
    Ok(())
}

fn validate_command(path: &Path, command: &ToolCommand) -> Result<(), ToolPackError> {
    if command.name.trim().is_empty() {
        return Err(invalid(path, "tool command name cannot be empty"));
    }
    if let Some(descriptor) = &command.descriptor
        && descriptor.trim().is_empty()
    {
        return Err(invalid(path, "tool command descriptor cannot be empty"));
    }
    for arg in &command.args {
        if arg.name.trim().is_empty() {
            return Err(invalid(path, "tool arg name cannot be empty"));
        }
    }
    for subcommand in &command.subcommands {
        validate_command(path, subcommand)?;
    }
    Ok(())
}

fn invalid(path: &Path, reason: impl Into<String>) -> ToolPackError {
    ToolPackError::InvalidToolPack {
        path: path.to_path_buf(),
        reason: reason.into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPackError {
    ReadDir { path: PathBuf, source: String },
    ReadFile { path: PathBuf, source: String },
    ParseToml { path: PathBuf, source: String },
    InvalidToolPack { path: PathBuf, reason: String },
}

impl fmt::Display for ToolPackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolPackError::ReadDir { path, source } => {
                write!(
                    formatter,
                    "failed to read tool-pack dir {}: {source}",
                    path.display()
                )
            }
            ToolPackError::ReadFile { path, source } => {
                write!(
                    formatter,
                    "failed to read tool-pack file {}: {source}",
                    path.display()
                )
            }
            ToolPackError::ParseToml { path, source } => {
                write!(
                    formatter,
                    "failed to parse tool-pack file {}: {source}",
                    path.display()
                )
            }
            ToolPackError::InvalidToolPack { path, reason } => {
                write!(formatter, "invalid tool-pack {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for ToolPackError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn loads_tool_pack_as_data() {
        let dir = tempdir().expect("temp dir");
        let pack_dir = dir.path().join("git");
        fs::create_dir(&pack_dir).expect("create pack dir");
        fs::write(
            pack_dir.join("tool.toml"),
            r#"
id = "tool.git"
name = "git"
version = "1"

[[commands]]
name = "branch"
description = "Inspect branches"

[[commands.subcommands]]
name = "--verbose"
descriptor = "known.git.branch-verbose"
patterns = ["git branch -v", "git branch --verbose"]

[[commands.args]]
name = "--all"
kind = "flag"
patterns = ["-a", "--all"]
affects_output = true
"#,
        )
        .expect("write tool pack");

        let packs = load_tool_packs_dir(dir.path()).expect("packs load");

        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].pack.id, "tool.git");
        assert_eq!(packs[0].pack.commands[0].name, "branch");
        assert_eq!(
            packs[0].pack.descriptor_refs(),
            vec!["known.git.branch-verbose"]
        );
    }

    #[test]
    fn rejects_unknown_tool_pack_fields() {
        let dir = tempdir().expect("temp dir");
        let path = dir.path().join("tool.toml");
        fs::write(
            &path,
            r#"
id = "tool.bad"
name = "bad"
version = "1"
surprise = true
"#,
        )
        .expect("write tool pack");

        let error = load_tool_pack_file(&path).expect_err("unknown fields fail");

        assert!(error.to_string().contains("failed to parse tool-pack file"));
    }
}
