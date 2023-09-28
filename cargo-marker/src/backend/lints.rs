use super::Config;
use crate::error::prelude::*;
use camino::Utf8PathBuf;

mod build;
mod fetch;

/// This struct contains all information of a lint crate required to compile
/// the crate. See the [fetch] module for how external crates are fetched and
/// this info is retrieved.
#[derive(Debug)]
pub struct LintCrateSource {
    /// The name of the package, for now we can assume that this is the name
    /// that will be used to construct the dynamic library.
    name: String,
    /// The absolute path to the manifest of this lint crate
    manifest: Utf8PathBuf,
}

/// The information of a compiled lint crate.
#[derive(Debug)]
pub struct LintCrate {
    /// The name of the crate
    pub name: String,
    /// The absolute path of the compiled crate, as a dynamic library.
    pub file: Utf8PathBuf,
}

/// This function fetches and builds all lints specified in the given [`Config`]
pub fn build_lints(config: &Config) -> Result<Vec<LintCrate>> {
    // FIXME(xFrednet): Potentially handle local crates compiled for UI tests
    // differently. Like running the build command in the project root. This
    // would allow cargo to cache the compilation better. Right now normal
    // Cargo and cargo-marker might invalidate each others caches.
    let sources = fetch::fetch_crates(config)?;
    build::build_lints(&sources, config)
}
