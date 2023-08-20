/// This module is intended to be imported via a wildcard import and it
/// brings into scope various symbols that are useful virtually in any
/// module that requires error handling.
pub(crate) mod prelude {
    pub(crate) use super::{Error, ErrorKind, Result};
    pub(crate) use marker_error::Context;
}

pub(crate) type Result<Ok = (), Kind = ErrorKind> = marker_error::Result<Ok, Kind>;
pub(crate) type Error = marker_error::Error<ErrorKind>;

use crate::observability::display;
use yansi::Paint;

/// The enum of all categorized errors for this crate.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ErrorKind {
    #[error("No lints were found in Cargo.toml or command line args")]
    #[diagnostic(help("{}", help_for_no_lints()))]
    LintsNotFound,

    #[error("Couldn't find driver in any of the potential locations")]
    #[diagnostic(help("{}", help_for_driver_not_found()))]
    DriverNotFound {
        #[related]
        errors: Vec<Error>,
    },

    #[error("Error: The required toolchain {} can't be found", toolchain.red())]
    #[diagnostic(help(
        "You can install the toolchain by running: {}\n\
        Or by adding the {} flag",
        display::cli(&format!("rustup toolchain install {toolchain} --component rustc-dev llvm-tools")),
        display::cli("--auto-install-toolchain"),
    ))]
    ToolchainNotFound { source: Error, toolchain: String },

    #[error("Failed to build the custom marker rustc driver")]
    #[diagnostic(help(
        "\
Make sure that you have the rustc-dev and llvm-tools components installed.
Try running the following:
{}

or:
{}",
        display::cli("cargo marker setup --auto-install-toolchain"),
        display::cli("rustup toolchain install {toolchain} --component rustc-dev llvm-tools")
    ))]
    BuildDriver,
}

fn help_for_no_lints() -> String {
    format!(
        r#"The are two ways to specify lints.

Specify the lints in Cargo.toml under {header}.
Example:

{config_example}

Specify the lints with the {lints} CLI parameter.
Example: {cli_example}"#,
        header = "[workspace.metadata.marker.lints]".bold().cyan(),
        config_example = display::toml(
            r#"[workspace.metadata.marker.lints]
# A local crate as a path
marker_lints = { path = './marker_lints' }
# An external crate via git
marker_lints = { git = "https://github.com/rust-marker/marker" }
# An external crate from a registry
marker_lints = "0.2.1""#
        ),
        cli_example = display::cli(r#"cargo marker --lints "marker_lints = '<version>'""#),
        lints = "--lints".blue(),
    )
}

fn help_for_driver_not_found() -> String {
    format!(
        "Try installing the driver by running the following.\n\n{}",
        display::cli("cargo marker setup --auto-install-toolchain")
    )
}
