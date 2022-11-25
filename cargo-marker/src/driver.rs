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
//!
//! If a driver is already installed. We'll first run the driver to request the
//! required toolchain and then run the driver using that toolchain. Requesting
//! the toolchain works, since the argument will be processed before rustc is run.
//! At least, that's the idea.

use std::process::Command;

use once_cell::sync::Lazy;

use crate::ExitStatus;

/// This is the driver version and toolchain, that is used by the setup command
/// to install the driver.
static DEFAULT_DRIVER_INFO: Lazy<RustcDriverInfo> = Lazy::new(|| RustcDriverInfo {
    toolchain: "nightly-2022-11-03".to_string(),
    version: "0.1.0".to_string(),
});

struct RustcDriverInfo {
    toolchain: String,
    version: String,
}

/// This function checks if the specified toolchain is installed. This requires
/// rustup. A dependency we have to live with for now.
fn check_toolchain(toolchain: &str) -> Result<(), ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.args([&format!("+{toolchain}"), "-V"]);
    if cmd.output().is_err() {
        eprintln!("error: the required toolchain `{toolchain}` can't be found");
        eprintln!();
        eprintln!("You can install the toolchain by running: rustup toolchain install {toolchain}");
        Err(ExitStatus::InvalidToolchain)
    } else {
        Ok(())
    }
}

/// This tries to install the rustc driver specified in [`DEFAULT_DRIVER_INFO`].
pub fn install_driver(verbose: bool) -> Result<(), ExitStatus> {
    // Prerequisites
    let toolchain = &DEFAULT_DRIVER_INFO.toolchain;
    check_toolchain(&toolchain)?;

    // Build driver
    let mut cmd = Command::new("cargo");
    cmd.arg(&format!("+{toolchain}"));

    if verbose {
        cmd.arg("--verbose");
    }

    #[cfg(feature = "dev-build")]
    cmd.args(["build", "--bin", "linter_driver_rustc"]);
    #[cfg(not(feature = "dev-build"))]
    cmd.args([
        "install",
        "marker_rustc_driver",
        "--version",
        &DEFAULT_DRIVER_INFO.version,
    ]);

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
