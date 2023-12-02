use super::cargo::Cargo;
use super::toolchain::{get_toolchain_folder, rustup_which, Toolchain};
use crate::error::prelude::*;
use crate::observability::display::print_stage;
use crate::observability::prelude::*;
use crate::utils::utf8::IntoUtf8;
use crate::{utils::is_local_driver, Result};
use camino::Utf8Path;
use std::process::Command;
use yansi::Paint;

pub fn marker_driver_bin_name() -> String {
    format!("marker_rustc_driver{}", std::env::consts::EXE_SUFFIX)
}

/// This is the driver version and toolchain, that is used by the setup command
/// to install the driver.
pub(crate) fn default_driver_info() -> DriverVersionInfo {
    DriverVersionInfo {
        // region replace rust toolchain dev
        toolchain: "nightly-2023-11-16".to_string(),
        // endregion replace rust toolchain dev
        // region replace marker version dev
        version: "0.4.3-rc".to_string(),
        api_version: "0.4.3-rc".to_string(),
        // endregion replace marker version dev
    }
}

/// The version info of one specific driver
pub struct DriverVersionInfo {
    pub toolchain: String,
    pub version: String,
    pub api_version: String,
}

impl DriverVersionInfo {
    pub fn try_from_toolchain(toolchain: &Toolchain, manifest: &Utf8Path) -> Result<DriverVersionInfo> {
        // The driver has to be invoked via cargo, to ensure that the libraries
        // are correctly linked. Toolchains are truly fun...
        let output = toolchain
            .cargo_with_driver()
            .arg("rustc")
            .arg("--quiet")
            .arg("--manifest-path")
            .arg(manifest.as_os_str())
            .arg("--")
            .arg("--toolchain")
            .log()
            .output()
            .context(|| "Failed to run the command `cargo rustc` to get the driver metadata")?;

        if !output.status.success() {
            return Err(Error::wrap(
                String::from_utf8_lossy(&output.stderr),
                "Command `cargo rustc` to get the driver metadata failed",
            ));
        }

        let info = output.stdout.into_utf8()?;

        let fields = ["toolchain", "driver", "marker-api"];

        let [toolchain, driver, marker_api] = fields.map(|field| {
            info.lines()
                .find_map(|line| line.strip_prefix(&format!("{field}: ")))
                .map(ToOwned::to_owned)
                .context(|| {
                    format!(
                        "The driver metadata doesn't contain the `{field}` field \
                        (dumped metadata on the next line)\n---\n{info}\n---"
                    )
                })
        });

        Ok(DriverVersionInfo {
            toolchain: toolchain?,
            version: driver?,
            api_version: marker_api?,
        })
    }
}

/// This tries to install the rustc driver specified in [`default_driver_info`].
pub(crate) fn install_driver(auto_install_toolchain: bool, additional_rustc_flags: Option<String>) -> Result {
    // The toolchain, driver version and api version should ideally be configurable.
    // However, that will require more prototyping and has a low priority rn.
    // See #60
    let default_driver = default_driver_info();

    let toolchain = &default_driver.toolchain;

    // If `auto-install-toolchain` is set, we want to run it regardless
    if auto_install_toolchain {
        install_toolchain(toolchain)?;
    }

    // Prerequisites
    rustup_which(toolchain, "cargo").map_err(|source| ErrorKind::ToolchainNotFound {
        source,
        toolchain: toolchain.clone(),
    })?;

    build_driver(toolchain, &default_driver.version, additional_rustc_flags)
}

fn install_toolchain(toolchain: &str) -> Result {
    let mut cmd = Command::new("rustup");

    cmd.args([
        "toolchain",
        "install",
        toolchain,
        "--component",
        "rustc-dev",
        "llvm-tools",
    ]);

    let status = cmd
        .log()
        .spawn()
        .expect("unable to start rustup to install the toolchain")
        .wait()
        .expect("unable to wait on rustup to install the toolchain");
    if status.success() {
        return Ok(());
    }

    Err(Error::root(format!(
        "Failed to install the toolchain {}",
        toolchain.red().bold()
    )))
}

/// This tries to compile the driver.
fn build_driver(toolchain: &str, version: &str, mut additional_rustc_flags: Option<String>) -> Result {
    if is_local_driver() {
        print_stage("compiling rustc driver");
    } else {
        print_stage(&format!("compiling rustc driver v{version} with {toolchain}"));
    }

    // Build driver
    let mut cmd = Cargo::with_toolchain(toolchain).command();
    if is_local_driver() {
        cmd.args(["build", "--bin", "marker_rustc_driver"]);
    } else {
        cmd.args(["install", "marker_rustc_driver", "--version", version, "--force"]);

        *additional_rustc_flags.get_or_insert_with(Default::default) += " --cap-lints=allow";

        let install_root = get_toolchain_folder(toolchain)?;
        cmd.arg("--root");
        cmd.arg(install_root.as_os_str());
        cmd.arg("--no-track");
    }

    if let Some(rustc_flags) = additional_rustc_flags {
        cmd.env("RUSTFLAGS", rustc_flags);
    }

    cmd.env("MARKER_ALLOW_DRIVER_BUILD", "1");

    let status = cmd
        .log()
        .spawn()
        .expect("unable to start cargo install for the driver")
        .wait()
        .expect("unable to wait on cargo install for the driver");
    if status.success() {
        return Ok(());
    }

    Err(Error::from_kind(ErrorKind::BuildDriver))
}
