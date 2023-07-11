#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_let_else)] // Rustfmt doesn't like `let ... else {` rn

#[allow(dead_code)] // Only check for dead code when the binary gets compiled
mod backend;
#[allow(dead_code)] // Only check for dead code when the binary gets compiled
mod config;
mod exit;
#[allow(dead_code)] // Only check for dead code when the binary gets compiled
mod utils;

use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

pub use exit::ExitStatus;

use crate::{backend::Config, config::LintDependencyEntry};

#[derive(Debug)]
pub struct TestSetup {
    pub rustc_path: PathBuf,
    /// The environment values that should be set. The first element is the
    /// value name, the second is the value the it should be set to.
    pub env_vars: HashMap<String, OsString>,
}

#[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
pub fn test_setup(krate_name: String, krate_dir: &Path) -> Result<TestSetup, ExitStatus> {
    let dev_build = cfg!(feature = "dev-build");

    let toolchain;
    if dev_build {
        let lint_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(lint_dir.parent().unwrap()).unwrap();
        backend::driver::install_driver(false, dev_build, "")?;

        toolchain = backend::toolchain::Toolchain::try_find_toolchain(dev_build, true)?;
        std::env::set_current_dir(lint_dir).unwrap();
    } else {
        toolchain = backend::toolchain::Toolchain::try_find_toolchain(dev_build, true)?;
    }

    let mut config = Config::try_base_from(toolchain)?;
    config.lints.insert(
        krate_name,
        LintDependencyEntry {
            source: config::Source::Path {
                path: krate_dir.to_str().unwrap().to_string(),
            },
            package: None,
            default_features: None,
            features: None,
        },
    );

    println!();
    println!("Compiling Lints:");
    let lints = backend::lints::build_lints(&config)?;
    let env_vars = HashMap::from([(
        "MARKER_LINT_CRATES".to_string(),
        backend::to_marker_lint_crates_env(&lints),
    )]);
    Ok(TestSetup {
        rustc_path: config.toolchain.driver_path.clone(),
        env_vars,
    })
}
