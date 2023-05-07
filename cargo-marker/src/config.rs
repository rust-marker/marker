use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use cargo_fetch::{GitReference, PackageSource};
use serde::Deserialize;

use toml_edit::easy::{from_str, Value};

use crate::{
    lints::{LintCrateSpec, PackageName},
    ExitStatus,
};

const CARGO_TOML: &str = "Cargo.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    lints: HashMap<String, LintDependency>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LintDependency {
    /// Version string like: `lint = "0.0.1"`
    Simple(String),
    /// Full dependency entry, like:
    /// lint = { path = ".", version = "1.0.0" }
    Full(LintDependencyEntry),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GitRef {
    Rev(String),
    Tag(String),
    Branch(String),
}

impl From<GitRef> for GitReference {
    fn from(value: GitRef) -> Self {
        match value {
            GitRef::Rev(rev) => GitReference::Revision(rev),
            GitRef::Tag(tag) => GitReference::Tag(tag),
            GitRef::Branch(branch) => GitReference::Branch(branch),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Source {
    // TODO: Registries are not supported yet, see https://github.com/rust-marker/marker/issues/87
    Registry {
        version: String,
        registry: Option<String>,
    },
    Git {
        git: String,
        #[serde(flatten)]
        git_ref: Option<GitRef>,
    },
    Path {
        path: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct LintDependencyEntry {
    #[serde(flatten)]
    source: Source,
    package: Option<String>,
    // TODO: Features are not supported yet, see https://github.com/rust-marker/marker/issues/81
    #[serde(rename = "default-features")]
    default_features: Option<bool>,
    features: Option<Vec<String>>,
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

impl Config {
    fn get_raw_manifest() -> Result<String, ConfigFetchError> {
        let Ok(mut config_file) = File::open(CARGO_TOML) else {
            return Err(ConfigFetchError::FileNotFound);
        };

        let mut config_str = String::new();
        if let Err(e) = config_file.read_to_string(&mut config_str) {
            return Err(ConfigFetchError::IoError(e));
        }
        Ok(config_str)
    }

    pub fn get_marker_config() -> Result<Config, ConfigFetchError> {
        let config_str = Self::get_raw_manifest()?;

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

    pub fn collect_crates(self) -> Result<Vec<LintCrateSpec>, ExitStatus> {
        self.lints.into_iter().map(dep_to_spec).collect()
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

fn dep_to_spec((name, dep): (String, LintDependency)) -> Result<LintCrateSpec, ExitStatus> {
    let dep = match dep {
        LintDependency::Simple(ver) => {
            eprintln!("{name} ({ver}): marker does not yet support registries");
            return Err(ExitStatus::InvalidValue);
        },
        LintDependency::Full(dep) => dep,
    };

    unsupported_fields!(
        name, dep => [
            default_features,
            features
        ]
    );

    let pkg_name = if let Some(package) = dep.package {
        PackageName::Renamed {
            orig: package,
            new: name.clone(),
        }
    } else {
        PackageName::Named(name.clone())
    };

    match dep.source {
        Source::Registry { version, .. } => {
            eprintln!("{name} ({version}): marker does not yet support registries");
            Err(ExitStatus::BadConfiguration)
        },
        Source::Git { git, git_ref } => {
            let src = PackageSource::git(&git, git_ref.map(Into::into)).map_err(|e| {
                eprintln!("{name}: {git} is not a valid git repository url ({e})");
                ExitStatus::InvalidValue
            })?;
            Ok(LintCrateSpec::new(pkg_name, None, src))
        },
        Source::Path { path } => {
            let path: PathBuf = path.into();
            let src = PackageSource::path(path.clone()).map_err(|e| {
                eprintln!("{name}: {} is not a valid lint crate path ({e})", path.display());
                ExitStatus::LintCrateNotFound
            })?;
            Ok(LintCrateSpec::new(pkg_name, None, src))
        },
    }
}
