mod build_space;

use crate::backend::Config;
use crate::error::prelude::*;
use crate::observability::prelude::*;
use crate::utils::utf8::IntoUtf8;
use camino::Utf8PathBuf;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::env::consts::DLL_EXTENSION;
use std::process::Stdio;

/// The information of a compiled lint crate.
#[derive(Debug)]
pub(crate) struct LintDll {
    /// The name of the crate
    pub(crate) name: String,

    /// The absolute path of the compiled crate, as a dynamic library.
    pub(crate) file: Utf8PathBuf,
}

/// This function fetches and builds all lints specified in the given [`Config`]
pub(crate) fn build(config: &Config) -> Result<Vec<LintDll>> {
    // Check that there are at least some lints that we don't attempt to
    // run a `cargo build` with no packages.
    if config.lints.is_empty() {
        return Err(Error::root("No lints were specified"));
    }

    // FIXME(xFrednet): Potentially handle local crates compiled for UI tests
    // differently. Like running the build command in the project root. This
    // would allow cargo to cache the compilation better. Right now normal
    // Cargo and cargo-marker might invalidate each others caches.
    build_space::init(config)?;

    let packages = config.lints.iter().flat_map(|(name, entry)| {
        let package = entry.package.as_deref().unwrap_or(name);
        ["-p", package]
    });

    let mut cmd = config.toolchain.cargo.command();
    cmd.current_dir(&config.marker_dir);
    cmd.arg("build");
    cmd.arg("--lib");
    cmd.arg("--message-format");
    cmd.arg("json");
    cmd.args(packages);

    // Potential "--release" flag
    if !config.debug_build {
        cmd.arg("--release");
    }

    // Environment
    cmd.env("RUSTFLAGS", &config.build_rustc_flags);

    let output = cmd
        .log()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("could not run cargo")
        .wait_with_output()
        .expect("failed to wait for cargo?");

    if !output.status.success() {
        return Err(Error::root("Failed to compile the lint crates"));
    }

    let output = output.stdout.into_utf8()?;

    // Parse the output of `cargo build --message-format json` to find the
    // compiled dynamic libraries.
    let mut dlls: BTreeMap<_, _> = output
        .lines()
        .map(|line| {
            serde_json::from_str(&line).context(|| {
                format!(
                    "Failed to parse `cargo build` json output line:\n\
                    ---\n{line}\n---"
                )
            })
        })
        .filter_map_ok(|message| {
            let cargo_metadata::Message::CompilerArtifact(artifact) = message else {
                return None;
            };

            if !artifact.target.kind.iter().any(|kind| kind == "cdylib") {
                return None;
            }

            let file_name = artifact
                .filenames
                .into_iter()
                .find(|file| file.extension() == Some(DLL_EXTENSION))?;

            Some((artifact.package_id, file_name))
        })
        .collect::<Result<_>>()?;

    let meta = config
        .toolchain
        .cargo
        .metadata()
        .current_dir(&config.marker_dir)
        .exec()
        .context(|| "Failed to run `cargo metadata` in the lint crates build space")?;

    // If manipulations with all the various metadata structures fail, we
    // may ask the user to enable the trace loggin to help us debug the issue.
    trace!(
        target: "build_space_metadata",
        compiler_messages = %output,
        cargo_metadata = %serde_json::to_string(&meta).unwrap(),
        "Build space metadata"
    );

    let root_package = meta
        .root_package()
        .context(|| "The build space must contain a root package, but it doesn't")?
        .clone();

    let root_package = meta
        .resolve
        .context(|| "Dependency graph from `cargo metadata` in the build space was not resolved")?
        .nodes
        .into_iter()
        .find(|node| node.id == root_package.id)
        .context(|| {
            format!(
                "The root package `{}` was not found in the dependency graph",
                root_package.name,
            )
        })?;

    config
        .lints
        .keys()
        .map(|name| {
            let package_id = &root_package
                .deps
                .iter()
                // FIXME: thsi doesn't work because the name is of the library target
                .find(|dep| dep.name == *name)
                .context(|| format!("The lint crate `{name}` was not found in the dependency graph"))?
                .pkg;

            let file = dlls.remove(package_id).context(|| {
                format!(
                    "Failed to find the dll for `{package_id}`.\n\
                    The following dlls are available: {dlls:#?}",
                )
            })?;

            Ok(LintDll {
                name: name.clone(),
                file,
            })
        })
        .collect()
}
