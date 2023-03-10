use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{cli::Flags, ExitStatus};

pub struct LintCrateSpec<'a> {
    /// Optional package name (this is always UTF-8, as opposed to `dir`), exists if supplied from
    /// config:
    ///
    /// ```toml
    /// lint_a = { path = "" }
    /// # `lint_a` is the package_name
    /// lint_b = { path = "", package = "lint_c"}
    /// # `lint_c` is the package name
    /// ```
    /// if the lint crate was supplied only from path, this is `None`, for example in case of
    /// command-line arguments:
    ///
    /// `--lints ./marker_lints`
    ///
    /// where `./marker_lints` is `dir`, and `package_name` in this case is empty.
    ///
    /// Setting this to `None` won't validate the package name when building the package with
    /// [`build()`](`Self::build`)
    package_name: Option<&'a str>,
    /// Path to lint crate
    dir: &'a Path,
}

impl<'a> LintCrateSpec<'a> {
    pub fn new(package_name: Option<&'a str>, dir: &'a Path) -> Self {
        Self { package_name, dir }
    }

    /// Currently only checks for semicolons, can be extended in the future
    pub fn is_valid(&self) -> bool {
        !self.dir.to_string_lossy().contains(';')
    }

    /// Creates a debug build for this crate. The path of the build library
    /// will be returned, if the operation was successful.
    pub fn build(&self, target_dir: &Path, flags: &Flags) -> Result<PathBuf, ExitStatus> {
        build_local_lint_crate(self, target_dir, flags)
    }
}

/// This creates a debug build for a local crate. The path of the build library
/// will be returned, if the operation was successful.
fn build_local_lint_crate(krate: &LintCrateSpec<'_>, target_dir: &Path, flags: &Flags) -> Result<PathBuf, ExitStatus> {
    if !krate.dir.exists() {
        eprintln!("The given lint can't be found, searched at: `{}`", krate.dir.display());
        return Err(ExitStatus::LintCrateNotFound);
    }

    let mut rustc_flags = if flags.forward_rust_flags {
        std::env::var("RUSTFLAGS").unwrap_or_default()
    } else {
        String::new()
    };
    rustc_flags += " --cap-lints=allow";

    // Compile the lint crate
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if flags.verbose {
        cmd.arg("--verbose");
    }
    if let Some(name) = krate.package_name {
        cmd.arg("--package");
        cmd.arg(name);
    }
    let exit_status = cmd
        .current_dir(std::fs::canonicalize(krate.dir).unwrap())
        .args(["--lib", "--target-dir"])
        .arg(target_dir.as_os_str())
        .env("RUSTFLAGS", rustc_flags)
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if !exit_status.success() {
        return Err(ExitStatus::LintCrateBuildFail);
    }

    // Find the final binary and return the string
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let lib_file_prefix = "lib";
    #[cfg(target_os = "windows")]
    let lib_file_prefix = "";

    // FIXME: currently this expect, that the lib name is the same as the crate dir.
    // See marker#60
    let file_name = format!(
        "{lib_file_prefix}{}",
        krate.dir.file_name().and_then(OsStr::to_str).unwrap_or_default()
    );
    // Here `debug` is attached as the crate is build with the `cargo build` command
    let mut krate_path = target_dir.join("debug").join(file_name);

    #[cfg(target_os = "linux")]
    krate_path.set_extension("so");
    #[cfg(target_os = "macos")]
    krate_path.set_extension("dylib");
    #[cfg(target_os = "windows")]
    krate_path.set_extension("dll");

    if !krate_path.exists() && !krate_path.is_file() {
        Err(ExitStatus::LintCrateLibNotFound)
    } else {
        Ok(krate_path)
    }
}
