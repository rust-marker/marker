//! This module is responsible for translating the `[workspace.metadata.marker]`
//! section in `Cargo.toml` files.
//!
//! The TOML format specifies that every TOML file must be a valid UTF-8.
//! ([source](https://toml.io/en/v1.0.0)) This allows Marker to just use
//! strings here, without worrying about OS specific string magic.

use std::{collections::HashMap, fs, io};

use camino::Utf8Path;
use serde::{Deserialize, Serialize};

use crate::ExitStatus;

#[derive(Deserialize, Debug)]
struct CargoToml {
    workspace: Option<Workspace>,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    metadata: Option<WorkspaceMetadata>,
}

#[derive(Deserialize, Debug)]
struct WorkspaceMetadata {
    marker: Option<Config>,
}

/// Markers metadata section `workspace.metadata.marker` in `Cargo.toml`
#[derive(Deserialize, Debug)]
pub struct Config {
    /// A list of lints.
    pub lints: HashMap<String, LintDependency>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum LintDependency {
    /// Version string like: `lint = "0.0.1"`
    Simple(String),
    /// Full dependency entry, like:
    /// lint = { path = ".", version = "1.0.0" }
    Full(LintDependencyEntry),
}

impl LintDependency {
    /// This function normalizes the struct, by making all paths absolute paths
    fn normalize(&mut self, workspace_path: &Utf8Path) -> Result<(), ConfigFetchError> {
        match self {
            LintDependency::Full(full) => full.source.normalize(workspace_path),
            LintDependency::Simple(_) => Ok(()),
        }
    }

    pub fn to_dep_entry(&self) -> LintDependencyEntry {
        match self {
            LintDependency::Simple(version) => LintDependencyEntry {
                source: Source::Registry {
                    version: version.clone(),
                    registry: None,
                },
                package: None,
                default_features: None,
                features: None,
            },
            LintDependency::Full(entry) => entry.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LintDependencyEntry {
    #[serde(flatten)]
    pub(crate) source: Source,
    pub(crate) package: Option<String>,
    // FIXME: Features are not supported yet, see https://github.com/rust-marker/marker/issues/81
    #[serde(rename = "default-features")]
    pub(crate) default_features: Option<bool>,
    pub(crate) features: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Source {
    /// A registry dependency, like `lint_crate = "1.0"`
    Registry { version: String, registry: Option<String> },
    /// A git dependency, like: `lint_crate = { git = "./lint_crate"}`
    Git {
        git: String,
        #[serde(flatten)]
        git_ref: Option<GitRef>,
    },
    /// A path dependency, like: `lint_crate = { path = "./lint_crate"}`
    Path { path: String },
}

impl Source {
    /// This function normalizes the struct, by making all paths absolute paths
    fn normalize(&mut self, workspace_path: &Utf8Path) -> Result<(), ConfigFetchError> {
        let Source::Path { path } = self else {
            return Ok(());
        };
        *path = workspace_path
            .join(&path)
            .canonicalize_utf8()
            .map_err(ConfigFetchError::IoError)?
            .into_string();
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GitRef {
    Rev(String),
    Tag(String),
    Branch(String),
}

#[derive(Debug)]
pub enum ConfigFetchError {
    /// Read failed
    IoError(io::Error),
    /// Couldn't parse `Cargo.toml`
    ParseError(toml::de::Error),
    /// `workspace.metadata.marker` doesn't exist
    SectionNotFound,
}

impl ConfigFetchError {
    pub fn emit_and_convert(self) -> ExitStatus {
        match self {
            ConfigFetchError::IoError(err) => eprintln!("IO error reading config: {err:?}"),
            // Better to use Display than Debug for toml error because it
            // will display the snippet of toml and highlight the error span
            ConfigFetchError::ParseError(err) => eprintln!("Can't parse config: {err}"),
            ConfigFetchError::SectionNotFound => eprintln!("Marker config wasn't found"),
        };
        ExitStatus::BadConfiguration
    }
}

impl Config {
    pub fn try_from_manifest(path: &Utf8Path) -> Result<Config, ConfigFetchError> {
        let config_str = fs::read_to_string(path).map_err(ConfigFetchError::IoError)?;

        Self::try_from_str(&config_str, path)
    }

    pub(crate) fn try_from_str(config_str: &str, path: &Utf8Path) -> Result<Config, ConfigFetchError> {
        let cargo_toml: CargoToml = toml::from_str(config_str).map_err(ConfigFetchError::ParseError)?;

        let mut config = cargo_toml
            .workspace
            .ok_or(ConfigFetchError::SectionNotFound)?
            .metadata
            .ok_or(ConfigFetchError::SectionNotFound)?
            .marker
            .ok_or(ConfigFetchError::SectionNotFound)?;

        let workspace_path = path
            .parent()
            .expect("path must have a parent after reading the `Cargo.toml` file");
        config.normalize(workspace_path)?;

        Ok(config)
    }

    /// This function normalizes the config, to be generally applicable. Currently,
    /// it normalizes all relative paths to be absolute paths instead.
    fn normalize(&mut self, workspace_path: &Utf8Path) -> Result<(), ConfigFetchError> {
        for lint in &mut self.lints.values_mut() {
            lint.normalize(workspace_path)?;
        }
        Ok(())
    }
}
