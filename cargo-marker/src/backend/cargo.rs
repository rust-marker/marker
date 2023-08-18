use std::process::Command;

use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use serde::Deserialize;

use crate::ExitStatus;

#[derive(Debug, Default)]
pub struct Cargo {
    /// The rustc toolchain this driver belongs to. This can be `None` during
    /// execution commands such as `cargo locate-project`
    pub(crate) toolchain: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ProjectLocation {
    root: Utf8PathBuf,
}

impl Cargo {
    pub fn with_toolchain(toolchain: impl Into<String>) -> Self {
        Self {
            toolchain: Some(toolchain.into()),
        }
    }

    /// This returns a command calling rustup, which acts as a proxy for the
    /// Cargo of the selected toolchain.
    /// It may add additional flags for verbose output.
    ///
    /// See also [`super::toolchain::Toolchain::cargo_build_command`] if the
    /// commands is intended to build a crate.
    pub fn command(&self) -> Command {
        // Marker requires rustc's shared libraries to be available. These are
        // added by rustup, when it acts like a proxy for cargo/rustc/etc.
        // This also means that cargo needs to be invoked via rustup, to have
        // the libraries available when Marker is invoked. This is such a mess...
        // All of this would be so, so much simpler if marker was part of rustup :/
        if let Some(toolchain) = &self.toolchain {
            let mut cmd = Command::new("rustup");
            cmd.args(["run", toolchain, "cargo"]);
            cmd
        } else {
            // for cargo locate-project - this command can be run without
            // specified toolchain
            Command::new("cargo")
        }
    }

    pub fn cargo_locate_project(&self) -> Result<Utf8PathBuf, ExitStatus> {
        let mut cmd = self.command();

        let output = cmd
            .arg("locate-project")
            .arg("--workspace")
            .output()
            .map_err(|err| ExitStatus::fatal(err, "error locating workspace manifest Cargo.toml"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExitStatus::fatal(
                stderr.trim(),
                format!("cargo locate-project failed with {}", output.status),
            ));
        }

        let manifest_location: ProjectLocation = serde_json::from_slice(&output.stdout)
            .map_err(|err| ExitStatus::fatal(err, "failed to deserialize cargo locate-project output"))?;

        Ok(manifest_location.root)
    }

    // Keep self for future changes. It's implemented in such way that clippy
    // doesn't ask to write it as an associative function.
    #[allow(clippy::unused_self)]
    pub fn metadata(&self) -> MetadataCommand {
        MetadataCommand::new()
    }
}
