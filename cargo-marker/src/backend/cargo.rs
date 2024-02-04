use crate::error::prelude::*;
use crate::observability::prelude::*;
use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use serde::Deserialize;
use std::process::Command;

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

    /// This returns a command calling rustup, which acts as a proxy for
    /// `cargo` of the selected toolchain. If no toolchain is selected, it
    /// will return a command calling the default `cargo` command.
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

    pub fn cargo_locate_project(&self) -> Result<Utf8PathBuf> {
        let mut cmd = self.command();

        cmd.arg("locate-project").arg("--color=always").arg("--workspace");
        let output = cmd
            .log()
            .output()
            .context(|| "Error locating workspace manifest Cargo.toml")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::wrap(
                stderr.trim(),
                format!("{} failed with {}", cmd.display(), output.status),
            ));
        }

        let manifest_location: ProjectLocation = serde_json::from_slice(&output.stdout).context(|| {
            format!(
                "Failed to deserialize cargo locate-project output (dumped it on the line bellow)\n\
                ---\n{}\n---",
                String::from_utf8_lossy(&output.stdout)
            )
        })?;

        Ok(manifest_location.root)
    }

    // Keep self for future changes. It's implemented in such way that clippy
    // doesn't ask to write it as an associative function.
    #[allow(clippy::unused_self)]
    pub fn metadata(&self) -> MetadataCommand {
        MetadataCommand::new()
    }
}
