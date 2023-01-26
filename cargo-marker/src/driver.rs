//! This module hosts functions required to run rustc as a driver.
//!
//! The rustc driver depends on rustc, which interfaces is unstable. This means
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

use std::{ffi::OsString, path::PathBuf, process::Command};

use once_cell::sync::Lazy;

use crate::ExitStatus;

/// This is the driver version and toolchain, that is used by the setup command
/// to install the driver.
static DEFAULT_DRIVER_INFO: Lazy<RustcDriverInfo> = Lazy::new(|| RustcDriverInfo {
    toolchain: "nightly-2023-01-26".to_string(),
    version: "0.1.0".to_string(),
    api_version: "0.1.0".to_string(),
});

struct RustcDriverInfo {
    toolchain: String,
    version: String,
    #[allow(unused)]
    api_version: String,
}

pub fn print_driver_version() {
    println!(
        "rustc driver version: {} (toolchain: {}, api: {})",
        DEFAULT_DRIVER_INFO.version, DEFAULT_DRIVER_INFO.toolchain, DEFAULT_DRIVER_INFO.api_version
    );
}

/// This tries to install the rustc driver specified in [`DEFAULT_DRIVER_INFO`].
pub fn install_driver(verbose: bool, dev_build: bool) -> Result<(), ExitStatus> {
    // The toolchain, driver version and api version should ideally be configurable.
    // However, that will require more prototyping and has a low priority rn.
    // See #60

    // Prerequisites
    let toolchain = &DEFAULT_DRIVER_INFO.toolchain;
    check_toolchain(toolchain)?;

    build_driver(toolchain, &DEFAULT_DRIVER_INFO.version, verbose, dev_build)?;

    // We don't want to advice the user, to install the driver again.
    check_driver(verbose, false)
}

/// This function checks if the specified toolchain is installed. This requires
/// rustup. A dependency we have to live with for now.
fn check_toolchain(toolchain: &str) -> Result<(), ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.args([&format!("+{toolchain}"), "-V"]);
    if cmd.output().is_err() {
        eprintln!("Error: The required toolchain `{toolchain}` can't be found");
        eprintln!();
        eprintln!("You can install the toolchain by running: rustup toolchain install {toolchain}");
        Err(ExitStatus::InvalidToolchain)
    } else {
        Ok(())
    }
}

/// This tries to compile the driver. If successful the driver binary will
/// be places next to the executable of `cargo-linter`.
fn build_driver(toolchain: &str, version: &str, verbose: bool, dev_build: bool) -> Result<(), ExitStatus> {
    if dev_build {
        println!("Compiling rustc driver");
    } else {
        println!("Compiling rustc driver v{version} with {toolchain}");
    }

    // Build driver
    let mut cmd = Command::new("cargo");

    if !dev_build {
        cmd.arg(&format!("+{toolchain}"));
    }

    if verbose {
        cmd.arg("--verbose");
    }

    if dev_build {
        cmd.args(["build", "--bin", "marker_driver_rustc"]);
    } else {
        // FIXME: This currently installs the binary in Cargo's default location.
        // Ideally this should install the driver in the toolchain folder for the
        // installed nightly version. This would allow the user to have multiple
        // drivers depending on the project toolchain.
        //
        // We can just reuse rustup to select the correct driver for a defined
        // toolchain. This would also simulate how the driver would be delivered
        // in a perfect world.
        //
        // See #60
        cmd.args(["install", "marker_rustc_driver", "--version", version]);
    }

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

fn check_driver(verbose: bool, print_advice: bool) -> Result<(), ExitStatus> {
    let path = get_driver_path();
    if verbose {
        println!("Searching for driver at: {}", path.display());
    }

    if !path.exists() || !path.is_file() {
        if print_advice {
            eprintln!("Error: The driver binary could not be found.");
            eprintln!();
            eprintln!("Try installing it via `cargo marker setup`");
        }

        Err(ExitStatus::MissingDriver)
    } else {
        Ok(())
    }
}

pub fn run_driver(
    env: Vec<(OsString, OsString)>,
    cargo_args: impl Iterator<Item = String>,
    verbose: bool,
) -> Result<(), ExitStatus> {
    check_driver(verbose, true)?;
    println!();
    println!("Start linting:");

    let mut cmd = Command::new("cargo");
    cmd.envs(env).arg("check").args(cargo_args);
    if verbose {
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

pub fn get_driver_path() -> PathBuf {
    #[allow(unused_mut)]
    let mut path = std::env::current_exe()
        .expect("unable to retrieve the path of the current executable")
        .with_file_name("marker_driver_rustc");

    #[cfg(target_os = "windows")]
    path.set_extension("exe");

    path
}
