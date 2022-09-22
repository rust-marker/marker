#![feature(once_cell)]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]

use std::{
    ffi::OsStr,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{exit, Command},
    sync::LazyLock,
};

use clap::{self, Arg};

const CARGO_ARGS_SEPARATOR: &str = "--";
const VERSION: &str = concat!("cargo-linter ", env!("CARGO_PKG_VERSION"));
const LINT_KRATES_BASE_DIR: &str = "./target/linter";
static LINT_KRATES_TARGET_DIR: LazyLock<String> = LazyLock::new(|| prepare_lint_build_dir("build", "target"));
static LINT_KRATES_OUT_DIR: LazyLock<String> = LazyLock::new(|| prepare_lint_build_dir("lints", "out"));

/// This creates the absolut path for a given build directory.
fn prepare_lint_build_dir(dir_name: &str, info_name: &str) -> String {
    if !Path::new("./target").exists() {
        // FIXME: This is a temporary check to ensure that we don't randomly create files.
        // This should not be part of the release and maybe be replaced by something more
        // elegant or removed completly.
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

fn main() {
    let matches = get_clap_config().get_matches_from(std::env::args().take_while(|s| s != CARGO_ARGS_SEPARATOR));
    if matches.is_present("version") {
        let version_info = env!("CARGO_PKG_VERSION");
        println!("{}", version_info);
        exit(0);
    }

    let verbose = matches.is_present("verbose");
    validate_driver(verbose);

    let mut lint_crates = vec![];
    if let Some(cmd_lint_crates) = matches.values_of("lints") {
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
        eprintln!("Please provide at least one valid lint crate, with the `--lint` argument");
        exit(-1);
    }

    if lint_crates.iter().any(|path| path.contains(';')) {
        eprintln!("The absolute paths of lint crates are not allowed to contain a `;`");
        exit(-1);
    }

    let driver_path = get_driver_path();
    let linter_crates_env = lint_crates.join(";");
    if matches.is_present("test-setup") {
        println!("env:RUSTC_WORKSPACE_WRAPPER={}", driver_path.display());
        println!("env:LINTER_LINT_CRATES={linter_crates_env}");
    } else {
        run_driver(&driver_path, &linter_crates_env);
    }
}

fn run_driver(driver_path: &PathBuf, lint_crates: &str) {
    println!();
    println!("Start linting:");

    let mut cmd = Command::new("cargo");
    let cargo_args = std::env::args().skip_while(|c| c != CARGO_ARGS_SEPARATOR).skip(1);
    cmd.env("RUSTC_WORKSPACE_WRAPPER", driver_path)
        .env("LINTER_LINT_CRATES", lint_crates)
        .arg("check")
        .args(cargo_args);

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if !exit_status.success() {
        exit(exit_status.code().unwrap_or(-1));
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

    let mut cmd = Command::new("cargo");
    if verbose {
        cmd.arg("--verbose");
    }
    let exit_status = cmd
        .current_dir(std::fs::canonicalize(&path).unwrap())
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

fn get_driver_path() -> PathBuf {
    #[allow(unused_mut)]
    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("linter_driver_rustc");

    #[cfg(target_os = "windows")]
    path.set_extension("exe");

    path
}

/// On release builds this will exit with a message and `-1` if the driver is missing.
fn validate_driver(verbose: bool) {
    #[cfg(feature = "dev-build")]
    {
        println!();
        println!("Compiling Driver:");
        let mut cmd = Command::new("cargo");
        if verbose {
            cmd.arg("--verbose");
        }

        let exit_status = cmd
            .args(["build", "-p", "linter_driver_rustc"])
            .env("RUSTFLAGS", "--cap-lints=allow")
            .spawn()
            .expect("could not run cargo")
            .wait()
            .expect("failed to wait for cargo?");

        if !exit_status.success() {
            exit(exit_status.code().unwrap_or(-1))
        }
    }

    let path = get_driver_path();
    if !path.exists() || !path.is_file() {
        eprintln!("Unable to find driver, searched at: {}", path.display());

        exit(-1)
    }
}

fn get_clap_config() -> clap::Command<'static> {
    clap::Command::new(VERSION)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .help("Print version info and exit"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print additional debug information to the console"),
        )
        .arg(
            Arg::new("lints")
                .short('l')
                .long("lints")
                .multiple_values(true)
                .takes_value(true)
                .help("Defines a set of lints crates that should be used"),
        )
        .arg(
            Arg::new("test-setup")
                .long("test-setup")
                .help("This flag will compile the lint crate and print all relevant environment values"),
        )
        .after_help(AFTER_HELP_MSG)
        .override_usage("cargo-linter [OPTIONS] -- <CARGO ARGS>")
}

const AFTER_HELP_MSG: &str = r#"CARGO ARGS
    All arguments after double dashes(`--`) will be passed to cargo.
    These options are the same as for `cargo check`.

EXAMPLES:
    * `cargo linter -l ./linter_lints`
"#;
