use std::{
    path::{Path, PathBuf},
    process::Command,
};

use cargo_metadata::MetadataCommand;

use crate::{utils::to_os_str, ExitStatus};

use super::{
    driver::{DEFAULT_DRIVER_INFO, MARKER_DRIVER_BIN_NAME},
    Config,
};

#[derive(Debug)]
pub struct Toolchain {
    pub(crate) driver_path: PathBuf,
    /// The Cargo binary that should be used for all Cargo commands. Prefer this
    /// over the `CARGO` environment value as this is the Cargo binary used for
    /// the specified toolchain.
    pub(crate) cargo_path: PathBuf,
    /// The rustc toolchain this driver belongs to. This can be `None` during
    /// custom builds where the driver was found but not the connected toolchain.
    pub(crate) toolchain: Option<String>,
}

impl Toolchain {
    /// This returns a command, calling the Cargo instance of the selected toolchain.
    /// It may add additional flags for verbose output.
    ///
    /// See also [`Self::cargo_build_command`] if the commands is intended to build
    /// a crate.
    pub fn cargo_command(&self) -> Command {
        // Marker requires rustc's shared libraries to be available. These are
        // added by rustup, when it acts like a proxy for cargo/rustc/etc.
        // This also means that cargo needs to be invoked via rustup, to have
        // the libraries available when Marker is invoked. This is such a mess...
        // All of this would be so, so much simpler if marker was part of rustup :/
        if let Some(toolchain) = &self.toolchain {
            let mut cmd = Command::new("cargo");

            cmd.env("RUSTUP_TOOLCHAIN", toolchain);

            cmd
        } else {
            Command::new(&self.cargo_path)
        }
    }

    pub fn cargo_with_driver(&self) -> Command {
        let mut cmd = self.cargo_command();

        cmd.env("RUSTC_WORKSPACE_WRAPPER", &self.driver_path);

        cmd
    }

    pub fn cargo_build_command(&self, config: &Config, manifest: &Path) -> Command {
        let mut cmd = self.cargo_command();
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

    pub fn cargo_metadata_command(&self) -> MetadataCommand {
        let mut command = MetadataCommand::new();
        command.cargo_path(&self.cargo_path);
        if let Some(toolchain) = self.toolchain.as_ref() {
            command.env("RUSTUP_TOOLCHAIN", toolchain);
        }
        command
    }

    pub fn find_target_dir(&self) -> Result<PathBuf, ExitStatus> {
        // FIXME(xFrednet): Handle errors properly.
        let metadata = self
            .cargo_metadata_command()
            .exec()
            .map_err(|_| ExitStatus::NoTargetDir)?;

        Ok(metadata.target_directory.into())
    }

    pub fn try_find_toolchain(dev_build: bool, verbose: bool) -> Result<Toolchain, ExitStatus> {
        if dev_build {
            Self::search_next_to_cargo_marker(verbose)
        } else {
            // First check if there is a rustc driver for the current toolchain. This
            // allows the used to override the used toolchain with `+<toolchain>` or
            // `rust-toolchain`
            if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
                if let Ok(info) = Self::search_toolchain(&toolchain, verbose) {
                    return Ok(info);
                }
            }

            // Next we check, if we can find a driver for the linked marker toolchain.
            if let Ok(info) = Self::search_toolchain(&DEFAULT_DRIVER_INFO.toolchain, verbose) {
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

    fn search_toolchain(toolchain: &str, verbose: bool) -> Result<Toolchain, ExitStatus> {
        if let Ok(driver_path) = rustup_which(toolchain, "marker_rustc_driver", verbose) {
            if let Ok(cargo_path) = rustup_which(toolchain, "cargo", verbose) {
                return Ok(Toolchain {
                    driver_path,
                    cargo_path,
                    toolchain: Some(toolchain.to_string()),
                });
            }
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
                    cargo_path: PathBuf::from(
                        std::env::var_os("CARGO").expect("expected environment value `CARGO` to be set"),
                    ),
                    toolchain: None,
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

    // Check if the toolchain is installed. We don't want to install it accidentally
    if let Ok(output) = Command::new("rustup").args(["toolchain", "list"]).output() {
        let text = to_os_str(output.stdout).expect("`Command` output should always be a valid `OsString`");
        if !text.to_string_lossy().contains(toolchain) {
            return Err(ExitStatus::MissingDriver);
        }
    } else {
        return Err(ExitStatus::ToolExecutionFailed);
    }

    // Check if the driver is installed
    if let Ok(output) = Command::new("rustup")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["which", tool])
        .output()
    {
        // rustup will error, if it can't find the binary file. Therefore,
        // we know that it exists if this succeeds
        if output.status.success() {
            if let Some(path_str) = to_os_str(output.stdout) {
                let path = PathBuf::from(path_str);

                // Remove the `\n` from the print output
                let trimmed_name = path.file_name().unwrap().to_str().unwrap().trim();
                let path = path.with_file_name(trimmed_name);

                if verbose {
                    println!("Found `{tool}` for `{toolchain}` at {}", path.to_string_lossy());
                }

                return Ok(path);
            }
        }
        return Err(ExitStatus::MissingDriver);
    }
    Err(ExitStatus::ToolExecutionFailed)
}
