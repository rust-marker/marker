//! The error handling in this crate is really not great. This is a quick hack
//! anyone interested in fixing this would be highly appreciated!!! <3

use std::fmt::Debug;

use crate::backend::driver::DEFAULT_DRIVER_INFO;

const HELP_FOR_NO_LINTS: &str = r#"No lints where specified.

* Try specifying them in `Cargo.toml` under `[workspace.metadata.marker.lints]`
    Example:
    ```
    [workspace.metadata.marker.lints]
    # A local crate as a path
    marker_lints = { path = './marker_lints' }
    # An external crate via git
    marker_lints = { git = "https://github.com/rust-marker/marker" }
    # An external crate from a registry
    marker_lints = "0.1.0"
    ```

* Try specifying them with the `--lints` argument
    Example:
    ```
    cargo marker --lints 'marker_lints = "<version>"'
    ```
"#;

const HELP_MISSING_DRIVER: &str = r#"Driver not found

* Try installing the driver by running:
    ```
    cargo marker setup --auto-install-toolchain
    ```
    "#;

const HELP_INSTALL_DRIVER_FAILED: &str = r#"Installing the driver failed

* Make sure that you have the `rustc-dev` and `llvm-tools` components installed. Try:
    ```
    cargo marker setup --auto-install-toolchain
    ```
    or:
    ```
    rustup toolchain install {{toolchain}} --component rustc-dev llvm-tools
    ```
"#;

pub enum ExitStatus {
    /// The toolchain validation failed. This could happen, if rustup is not
    /// installed or the required toolchain is not installed.
    InvalidToolchain = 100,
    /// The execution of a tool, like rustup or cargo, failed.
    ToolExecutionFailed = 101,
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
    /// Failed to fetch the lint crate
    LintCrateFetchFailed = 550,
    NoTargetDir = 551,
    /// General "bad config" error
    BadConfiguration = 600,
    /// No lint crates were specified -> nothing to do
    NoLints = 601,
    /// Can't deserialise `workspace.metadata.marker.lints` properly
    WrongStructure = 602,
    /// An invalid configuration value was specified
    InvalidValue = 603,
    /// Check failed
    MarkerCheckFailed = 1000,
}

impl Debug for ExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToolchain => write!(f, "InvalidToolchain"),
            Self::ToolExecutionFailed => write!(f, "ToolExecutionFailed"),
            Self::MissingDriver => write!(f, "{HELP_MISSING_DRIVER}"),
            Self::DriverInstallationFailed => write!(
                f,
                "{}",
                HELP_INSTALL_DRIVER_FAILED.replace("{{toolchain}}", &DEFAULT_DRIVER_INFO.toolchain)
            ),
            Self::DriverFailed => write!(f, "DriverFailed"),
            Self::LintCrateBuildFail => write!(f, "LintCrateBuildFail"),
            Self::LintCrateNotFound => write!(f, "LintCrateNotFound"),
            Self::LintCrateLibNotFound => write!(f, "LintCrateLibNotFound"),
            Self::LintCrateFetchFailed => write!(f, "LintCrateFetchFailed"),
            Self::NoTargetDir => write!(f, "NoTargetDir"),
            Self::BadConfiguration => write!(f, "BadConfiguration"),
            Self::NoLints => write!(f, "{HELP_FOR_NO_LINTS}"),
            Self::WrongStructure => write!(f, "WrongStructure"),
            Self::InvalidValue => write!(f, "InvalidValue"),
            Self::MarkerCheckFailed => write!(f, "MarkerCheckFailed"),
        }
    }
}
