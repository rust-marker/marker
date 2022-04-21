#![warn(clippy::pedantic, clippy::index_refutable_slice)]

use std::{path::PathBuf, process::exit};

use clap::{self, Arg, ArgMatches};

const VERSION: &str = concat!("cargo-linter ", env!("CARGO_PKG_VERSION"));

fn main() {
    let matches = get_clap_config();
    if matches.is_present("version") {
        let version_info = env!("CARGO_PKG_VERSION");
        println!("{}", version_info);
        exit(0);
    }

    let verbose = matches.is_present("verbose");
    validate_driver(verbose);

    // TODO xFrednet: check lint paths, compile lints, register lints in env

    println!("Hello from cargo-linter");
}

fn get_driver_path() -> PathBuf {
    #[allow(unused_mut)]
    let mut path = std::env::current_exe()
        .expect("current executable path invalid")
        .with_file_name("linter_driver_rustc");

    #[cfg(feature = "windows")]
    path.with_extension("exe");

    path
}

/// On release builds this will exit with a message and `-1` if the driver is missing.
fn validate_driver(verbose: bool) {
    #[cfg(feature = "dev-build")]
    {
        use std::process::Command;

        println!();
        let mut cmd = Command::new("cargo");
        if verbose {
            cmd.arg("--verbose");
        }

        let exit_status = cmd
            .args(["build", "-p", "linter_driver_rustc"])
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

fn get_clap_config() -> ArgMatches {
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
                .short("l")
                .long("lints")
                .multiple_values(true)
                .help("Defines a set of lints crates that should be used"),
        )
        .get_matches()
}
