use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use crate::{utils::is_local_driver, ExitStatus};

use super::{
    cargo::Cargo,
    driver::{DEFAULT_DRIVER_INFO, MARKER_DRIVER_BIN_NAME},
    Config,
};

#[derive(Debug)]
pub struct Toolchain {
    pub(crate) driver_path: PathBuf,
    /// A type containing toolchain to which the driver belongs.
    /// May not have a toolchain during custom builds when
    /// a driver was found but not the connected toolchain.
    pub(crate) cargo: Cargo,
}

impl Toolchain {
    pub fn cargo_with_driver(&self) -> Command {
        let mut cmd = self.cargo.command();

        cmd.env("RUSTC_WORKSPACE_WRAPPER", &self.driver_path);

        cmd
    }

    pub fn cargo_build_command(&self, config: &Config, manifest: &Path) -> Command {
        let mut cmd = self.cargo.command();
        cmd.arg("build");

        // Manifest
        cmd.arg("--manifest-path");
        cmd.arg(manifest.as_os_str());

        // Target dir
        cmd.arg("--target-dir");
        cmd.arg(config.markers_target_dir().as_os_str());

        // Potential "--release" flag
        if !config.debug_build {
            cmd.arg("--release");
        }

        // Environment
        cmd.env("RUSTFLAGS", &config.build_rustc_flags);

        cmd
    }

    pub fn find_target_dir(&self) -> Result<PathBuf, ExitStatus> {
        // FIXME(xFrednet): Handle errors properly.
        let metadata = self.cargo.metadata().exec().map_err(|_| ExitStatus::NoTargetDir)?;

        Ok(metadata.target_directory.into())
    }

    pub fn try_find_toolchain(verbose: bool) -> Result<Toolchain, ExitStatus> {
        if is_local_driver() {
            Self::search_next_to_cargo_marker(verbose)
        } else {
            // First check if there is a rustc driver for the current toolchain. This
            // allows the user to override the used toolchain with `+<toolchain>` or
            // `rust-toolchain`
            if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
                if let Ok(info) = Self::search_driver(&toolchain, verbose) {
                    return Ok(info);
                }
            }

            // Next we check, if we can find a driver for the linked marker toolchain.
            if let Ok(info) = Self::search_driver(&DEFAULT_DRIVER_INFO.toolchain, verbose) {
                return Ok(info);
            }

            // Check if this is a *weird* custom installation, where the driver is
            // placed next to the `cargo-marker` binary for one reason or another.
            if let Ok(path) = Self::search_next_to_cargo_marker(verbose) {
                return Ok(path);
            }

            Err(ExitStatus::MissingDriver)
        }
    }

    fn search_driver(toolchain: &str, verbose: bool) -> Result<Toolchain, ExitStatus> {
        if let Ok(driver_path) = rustup_which(toolchain, "marker_rustc_driver", verbose) {
            return Ok(Toolchain {
                driver_path,
                cargo: Cargo::with_toolchain(toolchain),
            });
        }

        Err(ExitStatus::MissingDriver)
    }

    fn search_next_to_cargo_marker(verbose: bool) -> Result<Toolchain, ExitStatus> {
        if let Ok(path) = std::env::current_exe() {
            let driver_path = path.with_file_name(MARKER_DRIVER_BIN_NAME);
            if verbose {
                println!("Searching for driver at '{}'", driver_path.to_string_lossy());
            }

            if driver_path.exists() && driver_path.is_file() {
                if verbose {
                    println!("Found driver at '{}'", driver_path.to_string_lossy());
                }
                return Ok(Toolchain {
                    driver_path,
                    cargo: Cargo::default(),
                });
            }
        }

        Err(ExitStatus::MissingDriver)
    }
}

pub(crate) fn get_toolchain_folder(toolchain: &str) -> Result<PathBuf, ExitStatus> {
    if let Ok(toolchain_cargo) = rustup_which(toolchain, "cargo", false) {
        // ../toolchain/bin/cargo -> ../toolchain
        if let Some(path) = toolchain_cargo.ancestors().nth(2) {
            return Ok(path.to_path_buf());
        }
    }
    Err(ExitStatus::BadConfiguration)
}

pub(crate) fn rustup_which(toolchain: &str, tool: &str, verbose: bool) -> Result<PathBuf, ExitStatus> {
    if verbose {
        println!("Searching for `{tool}` with rustup for toolchain `{toolchain}`");
    }

    let output = Command::new("rustup")
        .args(["which", "--toolchain", toolchain, tool])
        .output()
        .map_err(|err| ExitStatus::fatal(err, "failed to execute rustup"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ExitStatus::fatal(
            stderr.trim(),
            format!("failed to execute `rustup which` for `{tool}` with `{toolchain}` toolchain"),
        ));
    }

    let string_path = String::from_utf8(output.stdout).map_err(|err| ExitStatus::fatal(err, "incorrect bytes"))?;
    let path = PathBuf::from_str(string_path.trim())
        .map_err(|err| ExitStatus::fatal(err, format!("failed to parse path for `{tool}`")))?;

    if verbose {
        println!("Found `{tool}` for `{toolchain}` at {}", path.to_string_lossy());
    }
    Ok(path)
}
