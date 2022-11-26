#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod driver;
mod lints;

use std::{
    ffi::OsStr,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use cli::get_clap_config;
use driver::get_driver_path;
use once_cell::sync::Lazy;

const CARGO_ARGS_SEPARATOR: &str = "--";
const VERSION: &str = concat!("cargo-marker ", env!("CARGO_PKG_VERSION"));
const LINT_KRATES_BASE_DIR: &str = "./target/marker";
static LINT_KRATES_TARGET_DIR: Lazy<String> = Lazy::new(|| prepare_lint_build_dir("build", "target"));
static LINT_KRATES_OUT_DIR: Lazy<String> = Lazy::new(|| prepare_lint_build_dir("lints", "out"));

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

    if matches.get_flag("version") {
        print_version();
        return Ok(());
    }

    match matches.subcommand() {
        Some(("setup", _args)) => driver::install_driver(verbose),
        Some(("check", args)) => run_check(args, verbose),
        None => run_check(&matches, verbose),
        _ => unreachable!(),
    }
}

fn run_check(matches: &clap::ArgMatches, verbose: bool) -> Result<(), ExitStatus> {
    let mut lint_crates = vec![];
    if let Some(cmd_lint_crates) = matches.get_many::<String>("lints") {
        println!();
        println!("Compiling Lints:");
        lint_crates.reserve(cmd_lint_crates.len());
        for krate in cmd_lint_crates {
            if let Ok(krate_dir) = prepare_lint_crate(krate, verbose) {
                lint_crates.push(krate_dir);
            }
        }
    }

    if lint_crates.is_empty() {
        eprintln!("Please provide at least one valid lint crate, with the `--lints` argument");
        exit(-1);
    }

    if lint_crates.iter().any(|path| path.contains(';')) {
        eprintln!("The absolute paths of lint crates are not allowed to contain a `;`");
        exit(-1);
    }

    let driver_path = get_driver_path();
    let marker_crates_env = lint_crates.join(";");
    if matches.get_flag("test-setup") {
        println!("env:RUSTC_WORKSPACE_WRAPPER={}", driver_path.display());
        println!("env:MARKER_LINT_CRATES={marker_crates_env}");
        Ok(())
    } else {
        run_driver(&driver_path, &marker_crates_env)
    }
}

fn print_version() {
    println!("cargo-marker version: {}", env!("CARGO_PKG_VERSION"));
}

fn run_driver(driver_path: &PathBuf, lint_crates: &str) -> Result<(), ExitStatus> {
    println!();
    println!("Start linting:");

    let mut cmd = Command::new("cargo");
    let cargo_args = std::env::args().skip_while(|c| c != CARGO_ARGS_SEPARATOR).skip(1);
    cmd.env("RUSTC_WORKSPACE_WRAPPER", driver_path)
        .env("MARKER_LINT_CRATES", lint_crates)
        .arg("check")
        .args(cargo_args);

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(ExitStatus::MarkerCheckFailed)
    }
}

/// This function ensures that the given crate is compiled as a library and
/// returns the compiled library path if everything was successful. Otherwise
/// it'll issue a warning and return `Err`
fn prepare_lint_crate(krate: &str, verbose: bool) -> Result<String, ()> {
    let path = Path::new(krate);
    if !path.exists() {
        eprintln!("The given lint can't be found, searched at: `{}`", path.display());
        return Err(());
    }

    let mut cmd = cargo_command(verbose);
    let exit_status = cmd
        .current_dir(std::fs::canonicalize(path).unwrap())
        .args([
            "build",
            "--lib",
            "--target-dir",
            &*LINT_KRATES_TARGET_DIR,
            "-Z",
            "unstable-options",
            "--out-dir",
            &*LINT_KRATES_OUT_DIR,
        ])
        .env("RUSTFLAGS", "--cap-lints=allow")
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if !exit_status.success() {
        return Err(());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let lib_file_prefix = "lib";
    #[cfg(target_os = "windows")]
    let lib_file_prefix = "";

    // FIXME: currently this expect, that the lib name is the same as the crate dir.
    let file_name = format!(
        "{lib_file_prefix}{}",
        path.file_name().and_then(OsStr::to_str).unwrap_or_default()
    );
    let mut krate_path = Path::new(&*LINT_KRATES_OUT_DIR).join(file_name);

    #[cfg(target_os = "linux")]
    krate_path.set_extension("so");
    #[cfg(target_os = "macos")]
    krate_path.set_extension("dylib");
    #[cfg(target_os = "windows")]
    krate_path.set_extension("dll");

    Ok(krate_path.display().to_string())
}

fn cargo_command(verbose: bool) -> Command {
    // Here we want to use the normal cargo command, to go through the rustup
    // cargo executable and with that, set the required toolchain version.
    // This will add a slight overhead to each cargo call. This feels a bit
    // unavoidable, until marker is delivered as part of the toolchain. Let's
    // hope that day will happen!
    let mut cmd = Command::new("cargo");

    if verbose {
        cmd.arg("--verbose");
    }
    cmd
}
