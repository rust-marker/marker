use std::{path::Path, process::Command, str::from_utf8};

use once_cell::sync::Lazy;

use crate::ExitStatus;

use super::toolchain::{get_toolchain_folder, rustup_which, Toolchain};

#[cfg(unix)]
pub const MARKER_DRIVER_BIN_NAME: &str = "marker_rustc_driver";
#[cfg(windows)]
pub const MARKER_DRIVER_BIN_NAME: &str = "marker_rustc_driver.exe";

/// This is the driver version and toolchain, that is used by the setup command
/// to install the driver.
pub static DEFAULT_DRIVER_INFO: Lazy<DriverVersionInfo> = Lazy::new(|| DriverVersionInfo {
    toolchain: "nightly-2023-06-01".to_string(),
    version: "0.1.0".to_string(),
    api_version: "0.1.0".to_string(),
});

/// The version info of one specific driver
pub struct DriverVersionInfo {
    pub toolchain: String,
    pub version: String,
    pub api_version: String,
}

impl DriverVersionInfo {
    pub fn try_from_toolchain(toolchain: &Toolchain, manifest: &Path) -> Result<DriverVersionInfo, ExitStatus> {
        // The driver has to be invoked via cargo, to ensure that the libraries
        // are correctly linked. Toolchains are truly fun...
        if let Ok(output) = toolchain
            .cargo_with_driver()
            .arg("rustc")
            .arg("--quiet")
            .arg("--manifest-path")
            .arg(manifest.as_os_str())
            .arg("--")
            .arg("--toolchain")
            .output()
        {
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

                return Ok(DriverVersionInfo {
                    toolchain: toolchain?,
                    version: driver_version?,
                    api_version: api_version?,
                });
            }
        }

        Err(ExitStatus::DriverFailed)
    }
}

/// This tries to install the rustc driver specified in [`DEFAULT_DRIVER_INFO`].
pub fn install_driver(
    auto_install_toolchain: bool,
    dev_build: bool,
    additional_rustc_flags: &str,
) -> Result<(), ExitStatus> {
    // The toolchain, driver version and api version should ideally be configurable.
    // However, that will require more prototyping and has a low priority rn.
    // See #60

    // Prerequisites
    let toolchain = &DEFAULT_DRIVER_INFO.toolchain;
    if rustup_which(toolchain, "cargo", false).is_err() {
        if auto_install_toolchain {
            install_toolchain(toolchain)?;
        } else {
            eprintln!("Error: The required toolchain `{toolchain}` can't be found");
            eprintln!();
            eprintln!("You can install the toolchain by running: `rustup toolchain install {toolchain}`");
            eprintln!("Or by adding the `--auto-install-toolchain` flag");
            return Err(ExitStatus::InvalidToolchain);
        }
    }

    build_driver(
        toolchain,
        &DEFAULT_DRIVER_INFO.version,
        dev_build,
        additional_rustc_flags,
    )
}

fn install_toolchain(toolchain: &str) -> Result<(), ExitStatus> {
    let mut cmd = Command::new("rustup");

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

/// This tries to compile the driver.
fn build_driver(
    toolchain: &str,
    version: &str,
    dev_build: bool,
    additional_rustc_flags: &str,
) -> Result<(), ExitStatus> {
    if dev_build {
        println!("Compiling rustc driver");
    } else {
        println!("Compiling rustc driver v{version} with {toolchain}");
    }

    let mut rustc_flags = additional_rustc_flags.to_string();

    // Build driver
    let mut cmd = Command::new("cargo");
    if dev_build {
        cmd.args(["build", "--bin", "marker_rustc_driver"]);
    } else {
        cmd.env("RUSTUP_TOOLCHAIN", toolchain);
        cmd.args(["install", "marker_rustc_driver", "--version", version]);
        rustc_flags += " --cap-lints=allow";

        let install_root = get_toolchain_folder(toolchain)?;
        cmd.arg("--root");
        cmd.arg(install_root.as_os_str());
        cmd.arg("--no-track");
    }
    cmd.env("RUSTFLAGS", rustc_flags);
    cmd.env("MARKER_ALLOW_DRIVER_BUILD", "1");

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
