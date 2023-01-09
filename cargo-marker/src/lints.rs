use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use crate::ExitStatus;

/// This creates a debug build for a local crate. The path of the build library
/// will be returned, if the operation was successful.
pub fn build_local_lint_crate(
    package_name: Option<&str>,
    crate_dir: &Path,
    target_dir: &Path,
    verbose: bool,
) -> Result<PathBuf, ExitStatus> {
    if !crate_dir.exists() {
        eprintln!("The given lint can't be found, searched at: `{}`", crate_dir.display());
        return Err(ExitStatus::LintCrateNotFound);
    }

    // Compile the lint crate
    let mut cmd = Command::new("cargo");
    cmd.arg("build");
    if verbose {
        cmd.arg("--verbose");
    }
    if let Some(name) = package_name {
        cmd.arg("--package");
        cmd.arg(name);
    }
    let exit_status = cmd
        .current_dir(std::fs::canonicalize(crate_dir).unwrap())
        .args(["--lib", "--target-dir"])
        .arg(target_dir.as_os_str())
        .env("RUSTFLAGS", "--cap-lints=allow")
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
        crate_dir.file_name().and_then(OsStr::to_str).unwrap_or_default()
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
