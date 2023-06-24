//! This module hosts functions required to run rustc as a driver.
//!
//! The rustc driver depends on rustc's unstable interface. This means
//! that each driver version is bound to a specific version of rustc. The same
//! goes for Clippy. However, Clippy has the advantage, that it's distributes via
//! rustup, which handles the version matching for it. We're not so lucky, at
//! least not yet. Therefore, we're responsible that the driver is compiled and
//! run with the correct toolchain.
//!
//! If no driver is installed, the user will be requested to run the setup command.
//! That command will first ensure that the required toolchain is installed and then
//! run `cargo install` for the driver with a specific toolchain. The version and
//! toolchain are hardcoded in this crate.

use std::{ffi::OsString, path::PathBuf, process::Command, str::from_utf8};

use once_cell::sync::Lazy;

use crate::{cli::Flags, ExitStatus};

#[cfg(unix)]
const MARKER_DRIVER_BIN_NAME: &str = "marker_rustc_driver";
#[cfg(windows)]
const MARKER_DRIVER_BIN_NAME: &str = "marker_rustc_driver.exe";

/// This is the driver version and toolchain, that is used by the setup command
/// to install the driver.
static DEFAULT_DRIVER_INFO: Lazy<VersionInfo> = Lazy::new(|| VersionInfo {
    toolchain: "nightly-2023-06-01".to_string(),
    version: "0.1.0".to_string(),
    api_version: "0.1.0".to_string(),
});

pub struct VersionInfo {
    toolchain: String,
    version: String,
    api_version: String,
}

pub fn print_driver_version(flags: &Flags) {
    if let Ok(info) = RunInfo::find_driver_location(flags) {
        if let Ok(driver) = info.fetch_version_info() {
            println!(
                "rustc driver version: {} (toolchain: {}, api: {})",
                driver.version, driver.toolchain, driver.api_version
            );
        }
    }
}

/// This tries to install the rustc driver specified in [`DEFAULT_DRIVER_INFO`].
pub fn install_driver(flags: &Flags, auto_install_toolchain: bool) -> Result<(), ExitStatus> {
    // The toolchain, driver version and api version should ideally be configurable.
    // However, that will require more prototyping and has a low priority rn.
    // See #60

    // Prerequisites
    let toolchain = &DEFAULT_DRIVER_INFO.toolchain;
    if rustup_which(toolchain, "cargo", false).is_err() {
        if auto_install_toolchain {
            install_toolchain(toolchain, flags)?;
        } else {
            eprintln!("Error: The required toolchain `{toolchain}` can't be found");
            eprintln!();
            eprintln!("You can install the toolchain by running: `rustup toolchain install {toolchain}`");
            eprintln!("Or by adding the `--auto-install-toolchain` flag");
            return Err(ExitStatus::InvalidToolchain);
        }
    }

    build_driver(toolchain, &DEFAULT_DRIVER_INFO.version, flags)?;

    RunInfo::find_driver_location(flags).map(|_| ())
}

fn install_toolchain(toolchain: &str, flags: &Flags) -> Result<(), ExitStatus> {
    let mut cmd = Command::new("rustup");

    if flags.verbose {
        cmd.arg("--verbose");
    }

    cmd.args(["toolchain", "install", toolchain]);

    let status = cmd
        .spawn()
        .expect("unable to start rustup to install the toolchain")
        .wait()
        .expect("unable to wait on rustup to install the toolchain");
    if status.success() {
        Ok(())
    } else {
        // The user can see rustup's output, as the command output was passed on
        // to the user via the `.spawn()` call.
        Err(ExitStatus::InvalidToolchain)
    }
}

/// This tries to compile the driver. If successful the driver binary will
/// be places next to the executable of `cargo-linter`.
fn build_driver(toolchain: &str, version: &str, flags: &Flags) -> Result<(), ExitStatus> {
    if flags.dev_build {
        println!("Compiling rustc driver");
    } else {
        println!("Compiling rustc driver v{version} with {toolchain}");
    }

    // Build driver
    let mut cmd = Command::new("cargo");

    if !flags.dev_build {
        cmd.env("RUSTUP_TOOLCHAIN", toolchain);
    }

    if flags.verbose {
        cmd.arg("--verbose");
    }

    let mut rustc_flags = if flags.forward_rust_flags {
        std::env::var("RUSTFLAGS").unwrap_or_default()
    } else {
        String::new()
    };

    if flags.dev_build {
        cmd.args(["build", "--bin", "marker_rustc_driver"]);
    } else {
        // FIXME: This is awkward, it looks like I reserved `marker_rustc_driver`
        // as the crate name... I'll have to rename the other package -.-
        //
        // See: #143
        cmd.args(["install", "marker_rustc_driver", "--version", version]);
        rustc_flags += " --cap-lints=allow";

        let install_root = get_toolchain_folder(toolchain)?;
        cmd.arg("--root");
        cmd.arg(install_root.as_os_str());
        cmd.arg("--no-track");
    }
    cmd.env("RUSTFLAGS", rustc_flags);

    let status = cmd
        .spawn()
        .expect("unable to start cargo install for the driver")
        .wait()
        .expect("unable to wait on cargo install for the driver");
    if status.success() {
        Ok(())
    } else {
        // The user can see cargo's output, as the command output was passed on
        // to the user via the `.spawn()` call.
        Err(ExitStatus::DriverInstallationFailed)
    }
}

fn get_toolchain_folder(toolchain: &str) -> Result<PathBuf, ExitStatus> {
    if let Ok(toolchain_cargo) = rustup_which(toolchain, "cargo", false) {
        // ../toolchain/bin/cargo -> ../toolchain
        if let Some(path) = toolchain_cargo.ancestors().nth(2) {
            return Ok(path.to_path_buf());
        }
    }
    Err(ExitStatus::BadConfiguration)
}

/// This validate function validates that a driver is available and
/// it fetches the data of the driver
pub fn validate_and_get_info(flags: &Flags, print_advice: bool) -> Result<(RunInfo, VersionInfo), ExitStatus> {
    let search = RunInfo::find_driver_location(flags);

    if let Err(ExitStatus::MissingDriver) = search {
        if print_advice {
            eprintln!("Error: The driver could not be found.");
            eprintln!();
            eprintln!("Try installing it via `cargo marker setup`");
        }
        return Err(ExitStatus::MissingDriver);
    }

    let run_info = search?;
    let version_info = run_info.fetch_version_info()?;
    Ok((run_info, version_info))
}

pub fn run_driver(
    info: &RunInfo,
    env: Vec<(OsString, OsString)>,
    cargo_args: impl Iterator<Item = String>,
    flags: &Flags,
) -> Result<(), ExitStatus> {
    println!();
    println!("Start linting:");

    let mut cmd = info.cargo_cmd();
    cmd.envs(env).arg("check").args(cargo_args);
    if flags.verbose {
        cmd.arg("--verbose");
    }

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(ExitStatus::MarkerCheckFailed)
    }
}

#[derive(Debug)]
pub struct RunInfo {
    pub(crate) driver_path: PathBuf,
    pub(crate) cargo_path: PathBuf,
    pub(crate) toolchain: Option<String>,
}

impl RunInfo {
    pub fn find_driver_location(flags: &Flags) -> Result<RunInfo, ExitStatus> {
        if flags.dev_build {
            Self::search_next_to_cargo_marker(flags)
        } else {
            // First check if there is a rustc driver for the current toolchain. This
            // allows the used to override the used toolchain with `+<toolchain>` or
            // `rust-toolchain`
            if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
                if let Ok(info) = Self::search_toolchain(&toolchain, flags) {
                    return Ok(info);
                }
            }

            // Next we check, if we can find a driver for the linked marker toolchain.
            if let Ok(info) = Self::search_toolchain(&DEFAULT_DRIVER_INFO.toolchain, flags) {
                return Ok(info);
            }

            // Check if this is a *weird* custom installation, where the driver is
            // placed next to the `cargo-marker` binary for one reason or another.
            if let Ok(path) = Self::search_next_to_cargo_marker(flags) {
                return Ok(path);
            }

            Err(ExitStatus::MissingDriver)
        }
    }

    fn search_toolchain(toolchain: &str, flags: &Flags) -> Result<RunInfo, ExitStatus> {
        if let Ok(driver_path) = rustup_which(toolchain, "marker_rustc_driver", flags.verbose) {
            if let Ok(cargo_path) = rustup_which(toolchain, "cargo", flags.verbose) {
                return Ok(RunInfo {
                    driver_path,
                    cargo_path,
                    toolchain: Some(toolchain.to_string()),
                });
            }
        }

        Err(ExitStatus::MissingDriver)
    }

    fn search_next_to_cargo_marker(flags: &Flags) -> Result<RunInfo, ExitStatus> {
        if let Ok(path) = std::env::current_exe() {
            let driver_path = path.with_file_name(MARKER_DRIVER_BIN_NAME);
            if flags.verbose {
                println!("Searching for driver at '{}'", driver_path.to_string_lossy());
            }

            if driver_path.exists() && driver_path.is_file() {
                if flags.verbose {
                    println!("Found driver at '{}'", driver_path.to_string_lossy());
                }
                return Ok(RunInfo {
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

    fn cargo_cmd(&self) -> Command {
        let mut cmd = Command::new(&self.cargo_path);

        // In theory, it's not necessary to set the toolchain, as the comment
        // above takes the absolute path to the cargo of the toolchain. It's
        // still better to set it, to keep everything consistent.
        self.toolchain
            .as_ref()
            .map(|toolchain| cmd.env("RUSTUP_TOOLCHAIN", toolchain));

        cmd
    }

    fn fetch_version_info(&self) -> Result<VersionInfo, ExitStatus> {
        if let Ok(output) = Command::new(self.driver_path.as_os_str()).arg("--toolchain").output() {
            if !output.status.success() {
                return Err(ExitStatus::DriverFailed);
            }

            if let Ok(info) = from_utf8(&output.stdout) {
                let mut toolchain = Err(ExitStatus::InvalidValue);
                let mut driver_version = Err(ExitStatus::InvalidValue);
                let mut api_version = Err(ExitStatus::InvalidValue);
                for line in info.lines() {
                    if let Some(value) = line.strip_prefix("toolchain: ") {
                        toolchain = Ok(value.trim().to_string());
                    } else if let Some(value) = line.strip_prefix("driver: ") {
                        driver_version = Ok(value.trim().to_string());
                    } else if let Some(value) = line.strip_prefix("marker-api: ") {
                        api_version = Ok(value.trim().to_string());
                    }
                }

                return Ok(VersionInfo {
                    toolchain: toolchain?,
                    version: driver_version?,
                    api_version: api_version?,
                });
            }
        }

        Err(ExitStatus::DriverFailed)
    }
}

fn rustup_which(toolchain: &str, tool: &str, verbose: bool) -> Result<PathBuf, ExitStatus> {
    if verbose {
        println!("Searching for `{tool}` with rustup for toolchain `{toolchain}`");
    }

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

#[allow(clippy::unnecessary_wraps)]
fn to_os_str(bytes: Vec<u8>) -> Option<OsString> {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::OsStringExt;
        Some(OsString::from_vec(bytes))
    }

    // Windows paths are guaranteed to be valid UTF
    #[cfg(windows)]
    Some(OsString::from(String::from_utf8(bytes).ok()?))
}
