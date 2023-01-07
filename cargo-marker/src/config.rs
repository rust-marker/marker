use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

use serde::Deserialize;

use toml_edit::easy::{from_str, Value};

use crate::ExitStatus;

const CARGO_TOML: &str = "Cargo.toml";

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    lints: HashMap<String, LintDependency>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum LintDependency {
    /// Version string like: `lint = "0.0.1"`
    Simple(String),
    /// Full dependency entry, like:
    /// lint = { path = ".", version = "1.0.0" }
    Full(LintDependencyEntry),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LintDependencyEntry {
    path: Option<String>,
    // TODO: Everything below is not yet supported
    // Registry fetching:
    version: Option<String>,
    registry: Option<String>,
    // Git source fetching:
    git: Option<String>,
    rev: Option<String>,
    branch: Option<String>,
    // Features:
    features: Option<Vec<String>>,
    optional: Option<bool>,
    // TODO: do we want lint configuration here too?
    // configuration: Option<Value>
}

#[derive(Debug)]
pub enum ConfigFetchError {
    /// `Cargo.toml` wasn't found
    FileNotFound,
    /// Read failed
    IoError(io::Error),
    /// Couldn't parse `Cargo.toml`
    ParseError(toml_edit::de::Error),
    /// `workspace.metadata.marker` has invalid structure
    InvalidStructure,
    /// `workspace.metadata.marker` doesn't exist
    NotFound,
}

impl ConfigFetchError {
    pub fn emit_and_convert(self) -> ExitStatus {
        match self {
            ConfigFetchError::FileNotFound => eprintln!("`Cargo.toml` wasn't found"),
            ConfigFetchError::IoError(err) => eprintln!("IO error reading config: {err:?}"),
            ConfigFetchError::ParseError(err) => eprintln!("Can't parse config: {err:?}"),
            ConfigFetchError::NotFound => eprintln!("Marker config wasn't found"),
            ConfigFetchError::InvalidStructure => {
                eprintln!("`workspace.metadata.marker` has invalid structure");
                return ExitStatus::WrongStructure;
            },
        };
        ExitStatus::BadConfiguration
    }
}

macro_rules! unsupported_fields {
    ($name:expr, $dep:expr => [$($i:ident),+]) => {
        $(
            if let Some(ref $i) = $dep.$i {
                eprintln!(concat!("warning: {} ({:?}): marker doesn't yet support `", stringify!($i), "` field"), $name, $i);
            }
        )+
    }
}

impl Config {
    pub fn get_marker_config() -> Result<Config, ConfigFetchError> {
        let Ok(mut config_file) = File::open(CARGO_TOML) else {
            return Err(ConfigFetchError::FileNotFound);
        };

        let mut config_str = String::new();
        if let Err(e) = config_file.read_to_string(&mut config_str) {
            return Err(ConfigFetchError::IoError(e));
        }

        let cargo_config = match from_str::<Value>(&config_str) {
            Ok(v) => v,
            Err(e) => return Err(ConfigFetchError::ParseError(e)),
        };

        let Some(config_value) = cargo_config.get("workspace").and_then(|v| v.get("metadata")).and_then(|v| v.get("marker")) else {
            return Err(ConfigFetchError::NotFound);
        };

        let Some(marker_config) = config_value.clone().try_into::<Config>().ok() else {
            return Err(ConfigFetchError::InvalidStructure);
        };

        Ok(marker_config)
    }

    pub fn collect_paths(&self) -> Result<Vec<String>, ExitStatus> {
        self.lints
            .iter()
            .map(|(name, dep)| match dep {
                LintDependency::Simple(v) => {
                    eprintln!("{name} ({v}): marker doesn't yet support registries");
                    Err(ExitStatus::InvalidValue)
                },
                LintDependency::Full(dep) => {
                    unsupported_fields!(
                        name, dep => [
                            version,
                            registry,
                            git,
                            rev,
                            branch,
                            features,
                            optional
                        ]
                    );
                    if let Some(ref path) = dep.path {
                        return Ok(path.clone());
                    }
                    eprintln!("No `path` field found for lint crate {name}");
                    Err(ExitStatus::BadConfiguration)
                },
            })
            .collect()
    }
}
