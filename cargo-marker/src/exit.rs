//! The error handling in this crate is really not great. This is a quick hack
//! anyone interested in fixing this would be highly appreciated!!! <3

#[derive(Debug)]
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
