#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod driver;
mod lints;

use std::{
    ffi::{OsStr, OsString},
    fs::create_dir_all,
    io,
    path::{Path, PathBuf},
    process::exit,
};

use cli::get_clap_config;
use driver::{get_driver_path, run_driver};
use lints::build_local_lint_crate;
use once_cell::sync::Lazy;

use crate::driver::print_driver_version;

const CARGO_ARGS_SEPARATOR: &str = "--";
const VERSION: &str = concat!("cargo-marker ", env!("CARGO_PKG_VERSION"));
const LINT_KRATES_BASE_DIR: &str = "./target/marker";
static MARKER_LINT_DIR: Lazy<String> = Lazy::new(|| prepare_lint_build_dir("marker", "marker"));

#[derive(Debug)]
pub enum ExitStatus {
    /// The toolchain validation failed. This could happen, if rustup is not
    /// installed or the required toolchain is not installed.
    InvalidToolchain = 100,
    /// Unable to find the driver binary
    MissingDriver = 200,
    /// Nothing we can really do, but good to know. The user will have to analyze
    /// the forwarded cargo output.
    DriverInstallationFailed = 300,
    /// A general collection status, for failures originating from the driver
    DriverFailed = 400,
    /// The lint crate build failed for some reason
    LintCrateBuildFail = 500,
    /// Lint crate could not be found
    LintCrateNotFound = 501,
    /// The lint crate has been build, but the resulting binary could not be found.
    LintCrateLibNotFound = 502,
    /// Check failed
    MarkerCheckFailed = 1000,
}

/// This creates the absolute path for a given build directory.
fn prepare_lint_build_dir(dir_name: &str, info_name: &str) -> String {
    if !Path::new("./target").exists() {
        // FIXME: This is a temporary check to ensure that we don't randomly create files.
        // This should not be part of the release and maybe be replaced by something more
        // elegant or removed completely.
        eprintln!("No `target` directory exists, most likely running in the wrong directory");
        exit(-1);
    }

    let path = Path::new(LINT_KRATES_BASE_DIR).join(dir_name);
    if !path.exists() {
        create_dir_all(&path).unwrap_or_else(|_| panic!("Error while creating lint krate {info_name} directory"));
    }

    std::fs::canonicalize(path)
        .expect("This should find the directory, as we just created it")
        .display()
        .to_string()
}

fn main() -> Result<(), ExitStatus> {
    let matches = get_clap_config().get_matches_from(
        std::env::args()
            .enumerate()
            .filter_map(|(index, value)| (!(index == 1 && value == "marker")).then_some(value))
            .take_while(|s| s != CARGO_ARGS_SEPARATOR),
    );

    let verbose = matches.get_flag("verbose");
    let dev_build = cfg!(feature = "dev-build");

    if matches.get_flag("version") {
        print_version(verbose);
        return Ok(());
    }

    match matches.subcommand() {
        Some(("setup", _args)) => driver::install_driver(verbose, dev_build),
        Some(("check", args)) => run_check(args, verbose, dev_build),
        None => run_check(&matches, verbose, dev_build),
        _ => unreachable!(),
    }
}

fn run_check(matches: &clap::ArgMatches, verbose: bool, dev_build: bool) -> Result<(), ExitStatus> {
    // If this is a dev build, we want to recompile the driver before checking
    if dev_build {
        driver::install_driver(verbose, dev_build)?;
    }

    let mut lint_crates = vec![];
    if let Some(cmd_lint_crates) = matches.get_many::<OsString>("lints") {
        println!();
        println!("Compiling Lints:");
        lint_crates.reserve(cmd_lint_crates.len());
        let target_dir = Path::new(&*MARKER_LINT_DIR);
        for krate in cmd_lint_crates {
            let src_dir = PathBuf::from(krate);
            let crate_file = build_local_lint_crate(src_dir.as_path(), target_dir, verbose)?;
            lint_crates.push(crate_file.as_os_str().to_os_string());
        }
    }

    if lint_crates.is_empty() {
        eprintln!("Please provide at least one valid lint crate, with the `--lints` argument");
        exit(-1);
    }

    if lint_crates.iter().any(|path| path.to_string_lossy().contains(';')) {
        eprintln!("The absolute paths of lint crates are not allowed to contain a `;`");
        exit(-1);
    }

    #[rustfmt::skip]
    let env = vec![
        (OsString::from("RUSTC_WORKSPACE_WRAPPER"), get_driver_path().as_os_str().to_os_string()),
        (OsString::from("MARKER_LINT_CRATES"), lint_crates.join(OsStr::new(";")))
    ];
    if matches.get_flag("test-setup") {
        print_env(env).unwrap();
        Ok(())
    } else {
        let cargo_args = std::env::args().skip_while(|c| c != CARGO_ARGS_SEPARATOR).skip(1);
        run_driver(env, cargo_args, verbose)
    }
}

fn print_version(verbose: bool) {
    println!("cargo-marker version: {}", env!("CARGO_PKG_VERSION"));

    if verbose {
        print_driver_version();
    }
}

#[allow(clippy::unnecessary_wraps)]
fn print_env(env: Vec<(OsString, OsString)>) -> io::Result<()> {
    // Operating systems are fun... So, this function prints out the environment
    // values to the standard output. For Unix systems, this requires `OsStr`
    // objects, as file names are just bytes and don't need to be valid UTF-8.
    // Windows, on the other hand, restricts file names, but uses UTF-16. The
    // restriction only makes it slightly better, since windows `OsString` version
    // doesn't have a `bytes()` method. Rust additionally has a restriction on the
    // stdout of windows, that it has to be valid UTF-8, which means more conversion.
    //
    // This would be so much easier if everyone followed the "UTF-8 Everywhere Manifesto"

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::io::Write;
        use std::os::unix::prelude::OsStrExt;

        // stdout is used directly, to print the `OsString`s without requiring
        // them to be valid UTF-8
        let mut lock = io::stdout().lock();
        for (name, value) in env {
            write!(lock, "env:")?;
            lock.write_all(name.as_bytes())?;
            write!(lock, "=")?;
            lock.write_all(value.as_bytes())?;
            writeln!(lock)?;
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        for (name, value) in env {
            if let (Some(name), Some(value)) = (name.to_str(), value.to_str()) {
                println!("env:{name}={value}");
            } else {
                unreachable!("Windows requires it's file path to be valid UTF-16 AFAIK");
            }
        }

        Ok(())
    }
}
