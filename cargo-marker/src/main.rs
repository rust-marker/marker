#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// Subjectively `.map_or_else(default_fn, map_fn)`
// reads worse then `.map(map_fn).unwrap_or_else(default_fn)`
#![allow(clippy::map_unwrap_or)]

mod backend;
mod cli;
mod config;
mod error;
mod observability;
mod utils;

use error::prelude::*;
use std::{collections::HashMap, ffi::OsString};

use crate::backend::driver::DriverVersionInfo;
use backend::CheckInfo;
use cli::{CheckArgs, CliCommand, MarkerCli};
use config::Config;
use std::process::ExitCode;

fn main() -> ExitCode {
    observability::init();

    let Err(err) = try_main() else {
        return ExitCode::SUCCESS;
    };

    err.print();

    ExitCode::FAILURE
}

fn try_main() -> Result {
    let cli = MarkerCli::parse_args();

    let cargo = backend::cargo::Cargo::default();

    let path = cargo.cargo_locate_project()?;
    let config = Config::try_from_manifest(&path)?;

    match &cli.command {
        Some(CliCommand::Setup(args)) => {
            let rustc_flags = args
                .forward_rust_flags
                .then(|| std::env::var("RUSTFLAGS").ok())
                .flatten();

            backend::driver::install_driver(args.auto_install_toolchain, rustc_flags)
        },
        Some(CliCommand::Check(args)) => run_check(args, config, CheckKind::Normal),
        Some(CliCommand::TestSetup(args)) => run_check(args, config, CheckKind::TestSetup),
        None => run_check(&cli.check_args, config, CheckKind::Normal),
    }
}

#[derive(Debug, Clone, Copy)]
enum CheckKind {
    Normal,
    TestSetup,
}

fn run_check(args: &CheckArgs, config: Option<Config>, kind: CheckKind) -> Result {
    // determine lints
    let lints: HashMap<_, _> = cli::collect_lint_deps(args)?
        .or_else(|| config.map(|config| config.lints))
        .into_iter()
        .flatten()
        .map(|(name, dep)| (name, dep.to_dep_entry()))
        .collect();

    // Validation
    if lints.is_empty() {
        return Err(Error::from_kind(ErrorKind::LintsNotFound));
    }

    // If this is a dev build, we want to rebuild the driver before checking
    if utils::is_local_driver() {
        backend::driver::install_driver(false, None)?;
    }

    // Configure backend
    let toolchain = backend::toolchain::Toolchain::try_find_toolchain()?;
    let backend_conf = backend::Config {
        lints,
        ..backend::Config::try_base_from(toolchain)?
    };

    // Prepare backend
    let info = backend::prepare_check(&backend_conf)?;

    // Run backend
    match kind {
        CheckKind::Normal => backend::run_check(&backend_conf, info, &args.cargo_args),
        CheckKind::TestSetup => print_test_info(&backend_conf, &info),
    }
}

fn print_test_info(config: &backend::Config, check: &CheckInfo) -> Result {
    print_env(&check.env).unwrap();

    let info = DriverVersionInfo::try_from_toolchain(&config.toolchain, &config.marker_dir.join("Cargo.toml"))?;
    println!("info:toolchain={}", info.toolchain);
    println!("info:marker-api={}", info.api_version);

    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
fn print_env(env: &[(&'static str, OsString)]) -> std::io::Result<()> {
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
        let mut lock = std::io::stdout().lock();
        for (name, value) in env {
            write!(lock, "env:")?;
            lock.write_all(name.as_bytes())?;
            write!(lock, "=")?;
            lock.write_all(value.as_bytes())?;
            writeln!(lock)?;
        }
    }

    #[cfg(target_os = "windows")]
    {
        for (name, value) in env {
            if let Some(value) = value.to_str() {
                println!("env:{name}={value}");
            } else {
                unreachable!("Windows requires it's file path to be valid UTF-16 AFAIK");
            }
        }
    }

    Ok(())
}
