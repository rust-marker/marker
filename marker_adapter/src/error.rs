/// This module is intended to be imported via a wildcard import and it
/// brings into scope various symbols that are useful virtually in any
/// module that requires error handling.
pub(crate) mod prelude {
    pub(crate) use super::{Error, ErrorKind, Result};
    pub(crate) use marker_error::Context;
}

use marker_api::MARKER_API_VERSION;

pub type Result<Ok = (), Kind = ErrorKind> = marker_error::Result<Ok, Kind>;
pub type Error = marker_error::Error<ErrorKind>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ErrorKind {
    #[error(
        "Lint crate {lint_krate} uses the version of marker_api {marker_api_version} that is incompatible \
        with the version of marker_api {MARKER_API_VERSION} used in the driver"
    )]
    #[diagnostic(help(
        "update either the marker_api dependency in {lint_krate} \
        or update the driver to the latest version"
    ))]
    IncompatibleMarkerApiVersion {
        lint_krate: String,
        marker_api_version: String,
    },
}
