# Marker Adapter 🔌

The adapter provides a common interface for drivers to communicate with lint crates.
It does some heavy lifting which would otherwise need to be done by each individual driver.

⚠️ This is not a part of the stable API, the adapter might change in between releases ⚠️

## Interface

### Driver -> lint crate communication

The adapter can load defined lint crates and send information from the driver to all lint crates.
The driver takes care of safe ABI communication from the driver to the lint crates.

### Lint crate -> driver communication

The linting API and lint crates require some callbacks into the driver.
These callbacks use `extern "C"` functions with FFI safe types.
Drivers can just implement the `DriverContext` trait provided by the adapter,
all FFI related conversion is done by the adapter.

### Creating an adapter instance

An adapter instance for the specific driver can be crated from the environment.
For this, the following environment values are read:

* `MARKER_LINT_CRATES`: A semicolon separated list of lint crates in the form of compiled dynamic libraries.
