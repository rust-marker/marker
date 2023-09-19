use crate::error::prelude::*;
use crate::{backend::Config, config::LintDependencyEntry};
use camino::Utf8Path;
use itertools::Itertools;
use std::collections::BTreeMap;

/// Initializes a cargo workspace for building the lint crates dynamic libraries.
/// Creates a manifest file where all lint crates are mentioned as dependencies.
pub(crate) fn init(config: &Config) -> Result<()> {
    /// A small hack, to have the lints namespaced under the `[dependencies]` section
    #[derive(serde::Serialize)]
    struct DepNamespace<'a> {
        dependencies: &'a BTreeMap<String, LintDependencyEntry>,
    }

    let lints_as_deps = toml::to_string(&DepNamespace {
        dependencies: &config.lints,
    })
    .expect("DepNamespace can be represented as TOML");

    let toml_comments = comments("# ");
    let manifest_content = format!("{toml_comments}\n{MANIFEST_HEADER}{lints_as_deps}");
    let manifest_path = config.marker_dir.join("Cargo.toml");

    write_to_file(&manifest_path, &manifest_content)?;
    write_to_file(&config.marker_dir.join("src").join("lib.rs"), &comments("//! "))
}

fn write_to_file(path: &Utf8Path, content: &str) -> Result {
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("The file must have a parent directory. Path: {path}"));

    std::fs::create_dir_all(parent).context(|| format!("Failed to create the directory structure for {parent}"))?;

    std::fs::write(path, content).context(|| format!("Failed to write a file at {path}"))
}

fn comments(comment_prefix: &str) -> String {
    COMMENTS
        .trim()
        .lines()
        .format_with("\n", |line, f| f(&format_args!("{comment_prefix}{line}")))
        .to_string()
}

const COMMENTS: &str = r#"
Welcome 👋! This is an internal cargo workspace used by Marker to build lint crates 📦.
We put them as dependencies of this cargo package, and build them all at once 🔨.
As a result we get dynamic libraries 📚 for each of the lint crates and load them in
a custom rustc driver (marker_rustc_driver) at a later stage.
"#;

const MANIFEST_HEADER: &str = r#"
[package]
name = "marker-lint-crates-build-space"
version = "0.1.0"
edition = "2021"
publish = false

# This prevents Cargo from searching the parent directories for a workspace.
[workspace]

"#;
