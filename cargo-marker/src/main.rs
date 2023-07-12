#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::manual_let_else)] // Rustfmt doesn't like `let ... else {` rn

mod backend;
mod cli;
mod config;
mod exit;
mod utils;

use std::{collections::HashMap, ffi::OsString};

use backend::CheckInfo;
use cli::{get_clap_config, Flags};
use config::Config;

pub use exit::ExitStatus;

use crate::backend::driver::DriverVersionInfo;

const CARGO_ARGS_SEPARATOR: &str = "--";
const VERSION: &str = concat!("cargo-marker ", env!("CARGO_PKG_VERSION"));

fn main() -> Result<(), ExitStatus> {
    let matches = get_clap_config().get_matches_from(
        std::env::args()
            .enumerate()
            .filter_map(|(index, value)| (!(index == 1 && value == "marker")).then_some(value))
            .take_while(|s| s != CARGO_ARGS_SEPARATOR),
    );

    let flags = Flags::from_args(&matches);

    if matches.get_flag("version") {
        print_version();
        return Ok(());
    }

    let config = match Config::try_from_manifest() {
        Ok(v) => Some(v),
        Err(e) => match e {
            config::ConfigFetchError::SectionNotFound => None,
            _ => return Err(e.emit_and_convert()),
        },
    };

    match matches.subcommand() {
        Some(("setup", args)) => {
            let rustc_flags = if flags.forward_rust_flags {
                std::env::var("RUSTFLAGS").unwrap_or_default()
            } else {
                String::new()
            };
            backend::driver::install_driver(args.get_flag("auto-install-toolchain"), flags.dev_build, &rustc_flags)
        },
        Some(("check", args)) => run_check(args, config, &flags),
        None => run_check(&matches, config, &flags),
        _ => unreachable!(),
    }
}

fn run_check(args: &clap::ArgMatches, config: Option<Config>, flags: &Flags) -> Result<(), ExitStatus> {
    // determine lints
    let mut lints = HashMap::new();
    let deps = if let Some(deps) = cli::collect_lint_deps(args) {
        deps
    } else if let Some(config) = config {
        config.lints
    } else {
        HashMap::new()
    };
    for (name, dep) in deps {
        lints.insert(name, dep.to_dep_entry());
    }

    // Validation
    if lints.is_empty() {
        return Err(ExitStatus::NoLints);
    }

    // If this is a dev build, we want to rebuild the driver before checking
    if flags.dev_build {
        backend::driver::install_driver(false, flags.dev_build, "")?;
    }

    // Configure backend
    let toolchain = backend::toolchain::Toolchain::try_find_toolchain(flags.dev_build, flags.verbose)?;
    let backend_conf = backend::Config {
        dev_build: flags.dev_build,
        lints,
        ..backend::Config::try_base_from(toolchain)?
    };

    // Prepare backend
    let info = backend::prepare_check(&backend_conf)?;

    // Run backend
    if flags.test_build {
        print_test_info(&backend_conf, &info).unwrap();
        Ok(())
    } else {
        let additional_cargo_args: Vec<_> = std::env::args()
            .skip_while(|c| c != CARGO_ARGS_SEPARATOR)
            .skip(1)
            .collect();
        backend::run_check(&backend_conf, info, &additional_cargo_args)
    }
}

fn print_version() {
    println!("cargo-marker version: {}", env!("CARGO_PKG_VERSION"));
}

fn print_test_info(config: &backend::Config, check: &CheckInfo) -> Result<(), ExitStatus> {
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
