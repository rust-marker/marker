use crate::error::prelude::*;
use crate::observability::prelude::*;
use crate::utils::{is_local_driver, utf8::IntoUtf8};
use crate::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::process::Command;
use yansi::Paint;

use super::{
    cargo::Cargo,
    driver::{default_driver_info, marker_driver_bin_name},
    Config,
};

#[derive(Debug)]
pub struct Toolchain {
    pub(crate) driver_path: Utf8PathBuf,
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

    pub fn cargo_build_command(&self, config: &Config, manifest: &Utf8Path) -> Command {
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

    pub fn find_target_dir(&self) -> Result<Utf8PathBuf> {
        let metadata = self
            .cargo
            .metadata()
            .exec()
            .context(|| "Coudln't find the target directory")?;

        Ok(metadata.target_directory)
    }

    pub fn try_find_toolchain() -> Result<Toolchain> {
        if is_local_driver() {
            return Self::search_next_to_cargo_marker();
        }

        let mut errors = vec![];

        // First check if there is a rustc driver for the current toolchain. This
        // allows the user to override the used toolchain with `+<toolchain>` or
        // `rust-toolchain`
        if let Ok(toolchain) = std::env::var("RUSTUP_TOOLCHAIN") {
            match Self::search_driver(&toolchain) {
                Ok(toolchain) => return Ok(toolchain),
                Err(err) => errors.push(err),
            }
        }

        // Next we check, if we can find a driver for the linked marker toolchain.
        match Self::search_driver(&default_driver_info().toolchain) {
            Ok(toolchain) => return Ok(toolchain),
            Err(err) => errors.push(err),
        }

        // Check if this is a *weird* custom installation, where the driver is
        // placed next to the `cargo-marker` binary for one reason or another.
        match Self::search_next_to_cargo_marker() {
            Ok(toolchain) => return Ok(toolchain),
            Err(err) => errors.push(err),
        }

        Err(Error::from_kind(ErrorKind::DriverNotFound { errors }))
    }

    fn search_driver(toolchain: &str) -> Result<Toolchain> {
        let driver_path = rustup_which(toolchain, "marker_rustc_driver")?;

        Ok(Toolchain {
            driver_path,
            cargo: Cargo::with_toolchain(toolchain),
        })
    }

    fn search_next_to_cargo_marker() -> Result<Toolchain> {
        let current_exe = std::env::current_exe()
            .context(|| "Failed to get the current exe path")?
            .into_utf8()?;

        let driver_path = current_exe.with_file_name(marker_driver_bin_name());

        let _span = info_span!("search_next_to_cargo_marker", path = %driver_path).entered();

        info!("Searching for driver");

        if !driver_path.is_file() {
            return Err(Error::root(format!(
                "Could not find driver next to the cargo-marker binary at {}",
                driver_path.red().bold()
            )));
        }

        info!("Found driver");

        Ok(Toolchain {
            driver_path,
            cargo: Cargo::default(),
        })
    }
}

pub(crate) fn get_toolchain_folder(toolchain: &str) -> Result<Utf8PathBuf> {
    let toolchain_cargo = rustup_which(toolchain, "cargo")?;

    // ../toolchain/bin/cargo -> ../toolchain
    let path = toolchain_cargo.ancestors().nth(2).context(|| {
        format!(
            "Unexpected layout of the rustup toolchain binary dir. There are not \
            enough ancestors in the path `{toolchain_cargo}`"
        )
    })?;

    Ok(path.to_path_buf())
}

pub(crate) fn rustup_which(toolchain: &str, tool: &str) -> Result<Utf8PathBuf> {
    let mut cmd = Command::new("rustup");
    cmd.args(["which", "--toolchain", toolchain, tool]);

    let output = cmd.log().output().context(|| "Failed to execute rustup")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(Error::wrap(stderr.trim(), format!("Command failed: {}", cmd.display())));
    }

    let string_path = output.stdout.into_utf8()?;
    let path = Utf8PathBuf::from(string_path.trim());

    info!(%tool, %toolchain, path = %path, "Found the tool");

    Ok(path)
}
