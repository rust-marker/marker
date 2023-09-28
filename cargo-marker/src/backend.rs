//! The backend is the brains of rust-marker, it's responsible for installing or
//! finding the correct driver, building lints and start linting. The backend should
//! be decoupled from the frontend. Most of the time the frontend will be the
//! `cargo-marker` CLI. However, `cargo-marker` might also be used as a library for UI
//! tests later down the line.

use self::{lints::LintCrate, toolchain::Toolchain};
use crate::config::LintDependencyEntry;
use crate::error::prelude::*;
use crate::observability::display::{self, print_stage};
use crate::observability::prelude::*;
use camino::Utf8PathBuf;
use itertools::Itertools;
use std::collections::BTreeMap;

pub mod cargo;
pub mod driver;
pub mod lints;
pub mod toolchain;

/// Markers configuration for any action that requires lint crates to be available.
///
/// It's assumed that all paths in this struct are absolute paths.
#[derive(Debug)]
pub struct Config {
    /// The base directory used by Marker to fetch and compile lints.
    /// This will default to something like `./target/marker`.
    ///
    /// This should generally be used as a base path for everything. Notable
    /// exceptions can be the installation of a driver or the compilation of
    /// a lint for uitests.
    pub marker_dir: Utf8PathBuf,
    /// The list of lints.
    pub lints: BTreeMap<String, LintDependencyEntry>,
    /// Additional flags, which should be passed to rustc during the compilation
    /// of crates.
    pub build_rustc_flags: String,
    /// Indicates if this is a release or debug build.
    pub debug_build: bool,
    pub toolchain: Toolchain,
}

impl Config {
    pub fn try_base_from(toolchain: Toolchain) -> Result<Self> {
        Ok(Self {
            marker_dir: toolchain.find_target_dir()?.join("marker"),
            lints: BTreeMap::default(),
            build_rustc_flags: String::new(),
            debug_build: false,
            toolchain,
        })
    }

    fn markers_target_dir(&self) -> Utf8PathBuf {
        self.marker_dir.join("target")
    }

    fn lint_crate_dir(&self) -> Utf8PathBuf {
        self.marker_dir.join("lints")
    }
}

/// This struct contains all information to use rustc as a driver.
#[derive(Debug)]
pub struct CheckInfo {
    pub env: Vec<(&'static str, String)>,
}

pub fn prepare_check(config: &Config) -> Result<CheckInfo> {
    print_stage("compiling lints");
    let lints = lints::build_lints(config)?
        .iter()
        .map(|LintCrate { name, file }| format!("{name}:{file}"))
        .join(";");

    #[rustfmt::skip]
    let mut env = vec![
        ("RUSTC_WORKSPACE_WRAPPER", config.toolchain.driver_path.clone().into_string()),
        ("MARKER_LINT_CRATES", lints),
    ];
    if let Some(toolchain) = &config.toolchain.cargo.toolchain {
        env.push(("RUSTUP_TOOLCHAIN", toolchain.into()));
    }

    Ok(CheckInfo { env })
}

pub fn run_check(config: &Config, info: CheckInfo, additional_cargo_args: &[String]) -> Result {
    let stage = "linting";
    print_stage(stage);

    let mut cmd = config.toolchain.cargo_with_driver();
    cmd.arg("check");
    cmd.args(additional_cargo_args);

    cmd.envs(info.env);

    let exit_status = cmd
        .log()
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        return Ok(());
    }

    Err(Error::root(format!("{} finished with an error", display::stage(stage))))
}
