#![warn(clippy::pedantic, clippy::index_refutable_slice)]

use clap::{Arg, ArgMatches, Command};

const VERSION: &str = concat!("cargo-linter ", env!("CARGO_PKG_VERSION"));

fn main() {
    let _commands = get_clap_config();

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
