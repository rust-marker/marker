//! This module is responsible for translating the `[workspace.metadata.marker]`
//! section in `Cargo.toml` files.
//!
//! The TOML format specifies that every TOML file must be a valid UTF-8.
//! ([source](https://toml.io/en/v1.0.0)) This allows Marker to just use
//! strings here, without worrying about OS specific string magic.

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

use serde::{Deserialize, Serialize};

use crate::ExitStatus;

const CARGO_TOML: &str = "Cargo.toml";

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
    fn normalize(&mut self) {
        match self {
            LintDependency::Full(full) => full.source.normalize(),
            LintDependency::Simple(_) => {},
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
    source: Source,
    package: Option<String>,
    // FIXME: Features are not supported yet, see https://github.com/rust-marker/marker/issues/81
    #[serde(rename = "default-features")]
    default_features: Option<bool>,
    features: Option<Vec<String>>,
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
    fn normalize(&mut self) {
        if let Source::Path { ref mut path } = self {
            if let Ok(absolute_path) = std::fs::canonicalize(&path) {
                if let Some(absolute_path) = absolute_path.to_str() {
                    *path = absolute_path.to_string();
                }
            }
        }
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
    /// `Cargo.toml` wasn't found
    FileNotFound,
    /// Read failed
    IoError(io::Error),
    /// Couldn't parse `Cargo.toml`
    ParseError(toml::de::Error),
    /// `workspace.metadata.marker` has invalid structure
    InvalidStructure,
    /// `workspace.metadata.marker` doesn't exist
    SectionNotFound,
}

impl ConfigFetchError {
    pub fn emit_and_convert(self) -> ExitStatus {
        match self {
            ConfigFetchError::FileNotFound => eprintln!("`Cargo.toml` wasn't found"),
            ConfigFetchError::IoError(err) => eprintln!("IO error reading config: {err:?}"),
            ConfigFetchError::ParseError(err) => eprintln!("Can't parse config: {err:?}"),
            ConfigFetchError::SectionNotFound => eprintln!("Marker config wasn't found"),
            ConfigFetchError::InvalidStructure => {
                eprintln!("`workspace.metadata.marker` has invalid structure");
                return ExitStatus::WrongStructure;
            },
        };
        ExitStatus::BadConfiguration
    }
}

impl Config {
    pub fn try_from_manifest() -> Result<Config, ConfigFetchError> {
        let config_str = Self::load_raw_manifest()?;

        Self::try_from_str(&config_str)
    }

    pub fn try_from_str(config_str: &str) -> Result<Config, ConfigFetchError> {
        let cargo_config: toml::Value = match toml::from_str(config_str) {
            Ok(v) => v,
            Err(e) => return Err(ConfigFetchError::ParseError(e)),
        };

        let config_value = if let Some(value) = cargo_config
            .get("workspace")
            .and_then(|v| v.get("metadata"))
            .and_then(|v| v.get("marker"))
        {
            value
        } else {
            return Err(ConfigFetchError::SectionNotFound);
        };

        let mut config: Config = if let Ok(config) = config_value.clone().try_into() {
            config
        } else {
            return Err(ConfigFetchError::InvalidStructure);
        };

        config.normalize();

        Ok(config)
    }

    fn load_raw_manifest() -> Result<String, ConfigFetchError> {
        // FIXME(xFrednet): Use `cargo locate-project` to find the `Cargo.toml` file
        let Ok(mut config_file) = File::open(CARGO_TOML) else {
            return Err(ConfigFetchError::FileNotFound);
        };

        // FIXME(xFrednet): Maybe load `Cargo.toml` with `toml` that allows to display
        // warnings with a span
        let mut config_str = String::new();
        if let Err(e) = config_file.read_to_string(&mut config_str) {
            return Err(ConfigFetchError::IoError(e));
        }
        Ok(config_str)
    }

    /// This function normalizes the config, to be generally applicable. Currently,
    /// it normalizes all relative paths to be absolute paths instead.
    fn normalize(&mut self) {
        self.lints.iter_mut().for_each(|(_name, lint)| lint.normalize());
    }
}
