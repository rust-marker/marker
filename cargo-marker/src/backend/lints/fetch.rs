//! This module is responsible for fetching the lint crates and returning the
//! absolute path of the lints, for further processing.
//!
//! Cargo sadly doesn't provide an official public interface, to fetch crates
//! from git and registries. This will hopefully come at one point (rust-lang/cargo#1861)
//! but until then, we have to do some manual fetching.
//!
//! This module uses a small hack. Marker creates a new Cargo project, with the
//! specified lint crates as dependencies. Then `cargo fetch` is called, which
//! will download the crates into Cargo's cache. The absolute path to the lints
//! can then be retrieved from `cargo metadata`.

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use cargo_metadata::{Metadata, MetadataCommand};

use crate::{backend::Config, config::LintDependencyEntry, ExitStatus};

use super::LintCrateSource;

/// This function fetches and locates all lint crates specified in the given
/// configuration.
pub fn fetch_crates(config: &Config) -> Result<Vec<LintCrateSource>, ExitStatus> {
    // FIXME(xFrednet): Only create the dummy crate, if there is a non
    // local dependency.

    let manifest = setup_dummy_crate(config)?;

    call_cargo_fetch(&manifest, config)?;

    let metadata = call_cargo_metadata(&manifest, config)?;

    Ok(extract_lint_crate_sources(&metadata, config))
}

/// This function sets up the dummy crate with all the lints listed as dependencies.
/// It returns the path of the manifest, if everything was successful.
fn setup_dummy_crate(config: &Config) -> Result<PathBuf, ExitStatus> {
    /// A small hack, to have the lints namespaced under the `[dependencies]` section
    #[derive(serde::Serialize)]
    struct DepNamespace<'a> {
        dependencies: &'a HashMap<String, LintDependencyEntry>,
    }

    // Manifest
    let lints_as_deps = if let Ok(lints_as_deps) = toml::to_string(&DepNamespace {
        dependencies: &config.lints,
    }) {
        lints_as_deps
    } else {
        unreachable!("a valid toml structure is enforced my rustc's type system");
    };
    let manifest_content = DUMMY_MANIFEST_TEMPLATE.to_string() + &lints_as_deps;
    let manifest_path = config.marker_dir.join("Cargo.toml");
    write_to_file(&manifest_path, &manifest_content)?;

    // `./src/main.rs` file
    write_to_file(&config.marker_dir.join("src").join("main.rs"), DUMMY_MAIN_CONTENT)?;

    Ok(manifest_path)
}

fn write_to_file(path: &PathBuf, content: &str) -> Result<(), ExitStatus> {
    if let Some(parent) = path.parent() {
        // The result is ignored in this case. If the creation failed an error
        // will be emitted when the file creation fails. It's easier to handle
        // that case only once.
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = match OpenOptions::new().create(true).truncate(true).write(true).open(path) {
        Ok(file) => file,
        Err(_) => {
            // FIXME(xFrednet): Handle this case better by returning a custom status
            // with more information, to help us and users with debugging.
            return Err(ExitStatus::LintCrateFetchFailed);
        },
    };

    if file.write_all(content.as_bytes()).is_err() {
        // FIXME(xFrednet): Also handle this error better
        return Err(ExitStatus::LintCrateFetchFailed);
    }
    Ok(())
}

const DUMMY_MANIFEST_TEMPLATE: &str = r#"
# This is a dummy crate used by Marker, to get Cargo to fetch the lint crates
# as normal dependencies. The location of the fetched crates is then read using
# `cargo metadata`.

[package]
name = "markers-dummy-crate-for-fetching"
version = "0.1.0"
edition = "2021"
publish = false

# This prevents Cargo from searching the parent directories for a workspace.
[workspace]

"#;

const DUMMY_MAIN_CONTENT: &str = r#"
    fn main() {
        println!("Hey curious mind and welcome to Marker's internals.");
        println!("This is just a dummy crate to fetch some dependencies.");
        println!();
        println!("Achievement Unlocked: [Marker's hidden crate]");
    }
"#;

fn call_cargo_fetch(manifest: &Path, config: &Config) -> Result<(), ExitStatus> {
    let mut cmd = config.toolchain.cargo_command();
    cmd.arg("fetch");
    cmd.arg("--manifest-path");
    cmd.arg(manifest.as_os_str());

    // Only fetch for the specified target. Cargo will just fetch everything,
    // if the `--target` flag is not specified.
    if let Ok(target) = std::env::var("TARGET") {
        cmd.arg("--target");
        cmd.arg(target);
    }

    let status = cmd
        .spawn()
        .expect("unable to start `cargo fetch` to fetch lint crates")
        .wait()
        .expect("unable to wait for `cargo fetch` to fetch lint crates");
    if status.success() {
        Ok(())
    } else {
        // The user can see cargo's output, as the command output was passed on
        // to the user via the `.spawn()` call.
        Err(ExitStatus::DriverInstallationFailed)
    }
}

fn call_cargo_metadata(manifest: &PathBuf, config: &Config) -> Result<Metadata, ExitStatus> {
    let res = MetadataCommand::new()
        .cargo_path(&config.toolchain.cargo_path)
        .manifest_path(manifest)
        .exec();

    // FIXME(xFrednet): Handle errors properly.
    res.map_err(|_| ExitStatus::LintCrateFetchFailed)
}

fn extract_lint_crate_sources(metadata: &Metadata, marker_config: &Config) -> Vec<LintCrateSource> {
    metadata
        .packages
        .iter()
        .filter(|pkg| marker_config.lints.contains_key(&pkg.name))
        .map(|pkg| LintCrateSource {
            name: pkg.name.clone(),
            manifest: pkg.manifest_path.clone().into(),
        })
        .collect()
}
