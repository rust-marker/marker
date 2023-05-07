use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{cli::Flags, ExitStatus};

use cargo_fetch::{PackageFetcher, PackageSource};

/// Holds the "cargo name", and optionally the real name, of the package which contains the lint
/// crate.
///
/// This struct is parsed from `Cargo.toml`, see variant documentation for more info.
pub enum PackageName {
    /// Renamed - stores both "cargo name" and the real package name
    /// ```toml
    /// lint_b = { path = "...", package = "lint_c" }
    /// ```
    /// Results in:
    /// ```rust,no_run
    /// PackageName::Renamed { orig: "lint_c", new: "lint_b" }
    /// ```
    Renamed { orig: String, new: String },
    /// Not renamed - "cargo name" is the package name.
    /// ```toml
    /// lint_a = { path = "..." }
    /// ```
    /// Results in:
    /// ```rust,no_run
    /// PackageName::Named("lint_a")
    /// ```
    Named(String),
}

/// Represents a downloaded package
struct DownloadedPackage {
    /// Path to the package
    path: PathBuf,
    /// Optional package name, to specify during building.
    ///
    /// This is [`None`] if we don't rename the package.
    package_name: Option<String>,
}

impl DownloadedPackage {
    fn new(path: PathBuf, package_name: Option<String>) -> Self {
        Self { path, package_name }
    }
}

pub struct LintCrateSpec {
    /// Name of the package which contains the lint crate.
    ///
    /// if the lint crate was supplied only from path, this is taken from
    /// [`Path::file_name`], for example in case of
    /// command-line arguments:
    ///
    /// `--lints ./marker_lints`
    ///
    /// `marker_lints` is the `package_name`.
    package_name: PackageName,
    /// Version requirement of the package, [`None`] is a wildcard requirement, aka "*".
    version_req: Option<String>,
    /// Source to fetch the lint crate from.
    source: PackageSource,
}

impl LintCrateSpec {
    pub fn new(package_name: PackageName, version_req: Option<String>, source: PackageSource) -> Self {
        Self {
            package_name,
            version_req,
            source,
        }
    }

    fn fetch_many(specs: Vec<Self>) -> Result<Vec<DownloadedPackage>, String> {
        let mut fetcher = PackageFetcher::new()?;

        // Packages which we don't need to identify, they just need to be downloaded and built.
        // Basically: set of pkgs goes in - set of pathbufs goes out
        let mut auto_fetch = vec![];

        // Packages which we care to identify, because they need some special building flags
        // like `--package=...`
        let mut manual_fetch = vec![];

        for spec in specs {
            match spec.package_name {
                PackageName::Named(s) => {
                    auto_fetch.push(fetcher.resolve_first(s, spec.version_req.as_deref(), &spec.source, None)?);
                },
                PackageName::Renamed { orig, new } => manual_fetch.push((
                    fetcher.resolve_first(orig, spec.version_req.as_deref(), &spec.source, None)?,
                    new,
                )),
            }
        }

        let mut v: Vec<_> = fetcher
            .fetch_many(&auto_fetch)?
            .into_iter()
            .map(|path| DownloadedPackage::new(path, None))
            .collect();

        for package in manual_fetch {
            let pkg = fetcher.fetch(package.0)?;
            v.push(DownloadedPackage::new(pkg, Some(package.1)));
        }

        Ok(v)
    }

    /// Fetches and debug builds crates. The paths of built libraries
    /// will be returned, if the operations were successful.
    pub fn build_many(specs: Vec<Self>, target_dir: &Path, flags: &Flags) -> Result<Vec<PathBuf>, ExitStatus> {
        let pkg_paths = match Self::fetch_many(specs) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed fetching lint crates: {e}");
                return Err(ExitStatus::LintCrateFetchFailed);
            },
        };

        pkg_paths
            .into_iter()
            .map(|p| build_local_lint_crate(&p.path, p.package_name, target_dir, flags))
            .collect()
    }
}

fn build_local_lint_crate(
    krate: &Path,
    pkg_name: Option<String>,
    target_dir: &Path,
    flags: &Flags,
) -> Result<PathBuf, ExitStatus> {
    if !krate.exists() {
        eprintln!("The given lint can't be found, searched at: `{}`", krate.display());
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
    if let Some(name) = pkg_name {
        cmd.arg("--package");
        cmd.arg(name);
    }
    let exit_status = cmd
        .current_dir(std::fs::canonicalize(krate).unwrap())
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
    let mut file_name = OsString::from(lib_file_prefix);
    file_name.push(krate.file_name().unwrap_or_default());
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
