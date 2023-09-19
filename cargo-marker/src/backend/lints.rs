mod build_space;

use crate::backend::Config;
use crate::error::prelude::*;
use crate::observability::prelude::*;
use camino::Utf8PathBuf;
use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

/// The information of a compiled lint crate.
#[derive(Debug)]
pub(crate) struct LintDll {
    /// The name of the crate
    pub(crate) name: String,

    /// The absolute path of the compiled crate, as a dynamic library.
    pub(crate) file: Utf8PathBuf,
}

/// This function fetches and builds all lints specified in the given [`Config`]
pub(crate) fn build(config: &Config) -> Result<Vec<LintDll>> {
    // Check that there are at least some lints that we don't attempt to
    // run a `cargo build` with no packages.
    if config.lints.is_empty() {
        return Err(Error::root("No lints were specified"));
    }

    // FIXME(xFrednet): Potentially handle local crates compiled for UI tests
    // differently. Like running the build command in the project root. This
    // would allow cargo to cache the compilation better. Right now normal
    // Cargo and cargo-marker might invalidate each others caches.
    build_space::init(config)?;

    let packages = config.lints.keys().flat_map(|package| ["-p", package]);

    let mut cmd = config.toolchain.cargo.command();
    cmd.current_dir(&config.marker_dir);
    cmd.arg("build");
    cmd.args(packages);

    // Potential "--release" flag
    if !config.debug_build {
        cmd.arg("--release");
    }

    // Environment
    cmd.env("RUSTFLAGS", &config.build_rustc_flags);

    let exit_status = cmd
        .log()
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if !exit_status.success() {
        return Err(Error::root("Failed to compile the lint crates"));
    }

    let profile = if config.debug_build { "debug" } else { "release" };

    let dlls = config
        .lints
        .keys()
        .map(|package| {
            let dll_name = package.replace('-', "_");
            let file = config
                .marker_dir
                .join("target")
                .join(profile)
                .join(format!("{DLL_PREFIX}{dll_name}{DLL_SUFFIX}"));

            LintDll {
                name: package.clone(),
                file,
            }
        })
        .collect();

    Ok(dlls)
}
