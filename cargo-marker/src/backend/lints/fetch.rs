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

use super::LintCrateSource;
use crate::error::prelude::*;
use crate::observability::prelude::*;
use crate::{backend::Config, config::LintDependencyEntry};
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::Metadata;
use std::collections::BTreeMap;

/// This function fetches and locates all lint crates specified in the given
/// configuration.
pub fn fetch_crates(config: &Config) -> Result<Vec<LintCrateSource>> {
    // FIXME(xFrednet): Only create the dummy crate, if there is a non
    // local dependency.

    let manifest = setup_dummy_crate(config)?;

    call_cargo_fetch(&manifest, config)?;

    let metadata = call_cargo_metadata(&manifest, config)?;

    Ok(extract_lint_crate_sources(&metadata, config))
}

/// This function sets up the dummy crate with all the lints listed as dependencies.
/// It returns the path of the manifest, if everything was successful.
fn setup_dummy_crate(config: &Config) -> Result<Utf8PathBuf> {
    /// A small hack, to have the lints namespaced under the `[dependencies]` section
    #[derive(serde::Serialize)]
    struct DepNamespace<'a> {
        dependencies: &'a BTreeMap<String, LintDependencyEntry>,
    }

    // Manifest
    let lints_as_deps = toml::to_string(&DepNamespace {
        dependencies: &config.lints,
    })
    .expect("DepNamespace can be represented as TOML");

    let manifest_content = format!("{DUMMY_MANIFEST_TEMPLATE}{lints_as_deps}");
    let manifest_path = config.marker_dir.join("Cargo.toml");
    write_to_file(&manifest_path, &manifest_content)?;

    // `./src/main.rs` file
    write_to_file(&config.marker_dir.join("src").join("main.rs"), DUMMY_MAIN_CONTENT)?;

    Ok(manifest_path)
}

fn write_to_file(path: &Utf8Path, content: &str) -> Result {
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("The file must have a parent directory. Path: {path}"));

    std::fs::create_dir_all(parent).context(|| format!("Failed to create the directory structure for {parent}"))?;

    std::fs::write(path, content).context(|| format!("Failed to write a file at {path}"))
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

fn call_cargo_fetch(manifest: &Utf8Path, config: &Config) -> Result {
    let mut cmd = config.toolchain.cargo.command();
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
        .log()
        .spawn()
        .expect("unable to start `cargo fetch` to fetch lint crates")
        .wait()
        .expect("unable to wait for `cargo fetch` to fetch lint crates");

    if status.success() {
        return Ok(());
    }

    Err(Error::root("cargo fetch failed for lint crates"))
}

fn call_cargo_metadata(manifest: &Utf8Path, config: &Config) -> Result<Metadata> {
    config
        .toolchain
        .cargo
        .metadata()
        .manifest_path(manifest)
        .exec()
        .context(|| format!("Failed to get cargo metadata for the lint crates at {manifest}"))
}

fn extract_lint_crate_sources(metadata: &Metadata, marker_config: &Config) -> Vec<LintCrateSource> {
    metadata
        .packages
        .iter()
        .filter(|pkg| marker_config.lints.contains_key(&pkg.name))
        .map(|pkg| LintCrateSource {
            name: pkg.name.clone(),
            manifest: pkg.manifest_path.clone(),
        })
        .collect()
}
