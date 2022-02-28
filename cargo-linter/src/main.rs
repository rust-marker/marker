#![warn(clippy::pedantic, clippy::index_refutable_slice)]

use clap::{Arg, ArgMatches, Command};
use std::process::exit;

/// The edition might be important in the future if as it could potentually affect
/// the ABI therefore we want to include it in the verion. Sadly there is no
/// environment value for it.
const VERSION: &str = concat!("cargo-linter ", env!("CARGO_PKG_VERSION"), " (Edition 2021)");

fn main() {
    let commands = get_clap_config();

    println!("Hello from cargo-linter");
}

fn get_clap_config() -> ArgMatches {
    Command::new(VERSION)
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
        .get_matches()
}
