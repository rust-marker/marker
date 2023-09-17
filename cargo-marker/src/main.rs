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

use cli::MarkerCli;
use error::prelude::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    observability::init();

    let Err(err) = MarkerCli::parse_args().run() else {
        return ExitCode::SUCCESS;
    };

    err.print();

    ExitCode::FAILURE
}
