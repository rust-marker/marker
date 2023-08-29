//! This module is responsible for translating the `[workspace.metadata.marker]`
//! section in `Cargo.toml` files.
//!
//! The TOML format specifies that every TOML file must be a valid UTF-8.
//! ([source](https://toml.io/en/v1.0.0)) This allows Marker to just use
//! strings here, without worrying about OS specific string magic.

use crate::error::prelude::*;
use crate::observability::display;
use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use yansi::Paint;

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
// We want to make sure users don't mess up the configuration thinking that
// the values that they specified are used, when they are not.
// For example, `cargo` doesn't allow unknown fields in its config.
#[serde(deny_unknown_fields)]
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
    fn normalize(&mut self, package: &str, workspace_path: &Utf8Path) -> Result {
        match self {
            LintDependency::Full(full) => full.source.normalize(package, workspace_path),
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
    fn normalize(&mut self, package: &str, workspace_path: &Utf8Path) -> Result {
        let Source::Path { path } = self else {
            return Ok(());
        };
        *path = workspace_path
            .join(&path)
            .canonicalize_utf8()
            .context(|| {
                format!(
                    "Path to a lint crate is invalid: {}",
                    display::toml(&format!("{package} = {{ path = \"{path}\" }}",))
                )
            })?
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

impl Config {
    pub fn try_from_manifest(path: &Utf8Path) -> Result<Option<Config>> {
        let config_str = fs::read_to_string(path).context(|| format!("Failed to read config at {}", path.red()))?;

        Self::try_from_str(&config_str, path)
    }

    pub(crate) fn try_from_str(config_str: &str, path: &Utf8Path) -> Result<Option<Config>> {
        let cargo_toml: CargoToml =
            toml::from_str(config_str).context(|| format!("Could't parse config at {}", path.red()))?;

        let config = cargo_toml.workspace.and_then(|x| x.metadata).and_then(|x| x.marker);

        let Some(mut config) = config else { return Ok(None) };

        let workspace_path = path
            .parent()
            .expect("path must have a parent after reading the `Cargo.toml` file");
        config.normalize(workspace_path)?;

        Ok(Some(config))
    }

    /// This function normalizes the config, to be generally applicable. Currently,
    /// it normalizes all relative paths to be absolute paths instead.
    fn normalize(&mut self, workspace_path: &Utf8Path) -> Result {
        for (package, lint) in &mut self.lints {
            lint.normalize(package, workspace_path)?;
        }
        Ok(())
    }
}
