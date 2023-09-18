use crate::config::{Config, LintDependency};
use crate::error::prelude::*;
use crate::{backend, utils};
use camino::Utf8Path;
use clap::Args;
use std::collections::BTreeMap;

#[derive(Args, Debug)]
#[command(override_usage = "cargo marker check [OPTIONS] -- <CARGO ARGS>")]
pub(crate) struct CheckCommand {
    /// Specifies lint crates which should be used. (Lints in `Cargo.toml` will be ignored)
    #[arg(short, long)]
    pub(crate) lints: Vec<String>,

    /// Forward the current `RUSTFLAGS` value during the lint crate compilation
    #[arg(long)]
    pub(crate) forward_rust_flags: bool,

    /// Arguments which will be forwarded to Cargo. See `cargo check --help`
    #[clap(last = true)]
    pub(crate) cargo_args: Vec<String>,
}

impl CheckCommand {
    pub(crate) fn run(self, config: Option<Config>) -> Result {
        self.compile_lints(config)?.lint()
    }

    pub(crate) fn compile_lints(self, config: Option<Config>) -> Result<CompiledLints> {
        // determine lints
        let lints: BTreeMap<_, _> = self
            .lints_from_cli()?
            .or_else(|| config.map(|config| config.lints))
            .into_iter()
            .flatten()
            .map(|(name, dep)| (name, dep.into_dep_entry()))
            .collect();

        // Validation
        if lints.is_empty() {
            return Err(Error::from_kind(ErrorKind::LintsNotFound));
        }

        // If this is a dev build, we want to rebuild the driver before checking
        if utils::is_local_driver() {
            backend::driver::install_driver(false, None)?;
        }

        // Configure backend
        let toolchain = backend::toolchain::Toolchain::try_find_toolchain()?;
        let backend_conf = backend::Config {
            lints,
            ..backend::Config::try_base_from(toolchain)?
        };

        // Prepare backend
        let info = backend::prepare_check(&backend_conf)?;

        Ok(CompiledLints {
            backend_conf,
            info,
            cargo_args: self.cargo_args,
        })
    }

    fn lints_from_cli(&self) -> Result<Option<BTreeMap<String, LintDependency>>> {
        if self.lints.is_empty() {
            return Ok(None);
        }

        let mut virtual_manifest = "[workspace.metadata.marker.lints]\n".to_string();
        for dep in &self.lints {
            virtual_manifest.push_str(dep);
            virtual_manifest.push('\n');
        }

        let path = Utf8Path::new(".");

        let Config { lints } = Config::try_from_str(&virtual_manifest, path)?.unwrap_or_else(|| {
            panic!(
                "BUG: the config must definitely contain the marker metadata:\
                \n---\n{virtual_manifest}\n---"
            );
        });

        Ok(Some(lints))
    }
}

/// The result of discovering and compiling the lint libraries
#[derive(Debug)]
pub(crate) struct CompiledLints {
    pub(crate) backend_conf: backend::Config,
    pub(crate) info: backend::CheckInfo,
    pub(crate) cargo_args: Vec<String>,
}

impl CompiledLints {
    fn lint(self) -> Result {
        backend::run_check(&self.backend_conf, self.info, &self.cargo_args)
    }
}
