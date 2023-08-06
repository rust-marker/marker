#![doc = include_str!("../README.md")]

use std::{
    collections::HashMap,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    process::Command,
};

use semver::Version;
pub use ui_test;

#[derive(Debug)]
struct TestSetup {
    rustc_path: String,
    /// The environment values that should be set. The first element is the
    /// value name, the second is the value the it should be set to.
    env_vars: HashMap<String, String>,
    toolchain: String,
    marker_api: String,
}

/// This macro automatically fills the parameters of [`create_ui_test_config`]
/// with environment values and default values.
///
/// It assumes a dependency to `marker_api` and that all tests are located in the
/// `./tests/ui` folder.
#[macro_export]
macro_rules! simple_ui_test_config {
    () => {
        $crate::simple_ui_test_config!("tests/ui");
    };
    ($ui_dir:expr) => {
        $crate::simple_ui_test_config!("tests/ui", "./target");
    };
    ($ui_dir:expr, $target_dir:expr) => {
        $crate::create_ui_test_config(
            std::path::PathBuf::from($ui_dir),
            std::path::Path::new($target_dir),
            env!("CARGO_PKG_NAME"),
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
            marker_api::MARKER_API_VERSION,
        );
    };
}

/// This function creates a [`ui_test::Config`] instance configured to run Marker
/// as a driver, with the given crate as a lint crate.
///
/// It's recommended to use environment values or constants to retrieve the
/// parameters for this function.
///
/// ```rust,ignore
/// let config = create_ui_test_config(
///     PathBuf::from("tests/ui"),
///     Path::new("./target"),
///     env!("CARGO_PKG_NAME"),
///     Path::new(env!("CARGO_MANIFEST_DIR")),
///     marker_api::MARKER_API_VERSION,
/// );
/// ```
///
/// You can use the [`simple_ui_test_config`] macro to fill all parameters automatically.
pub fn create_ui_test_config(
    ui_dir: PathBuf,
    target_dir: &Path,
    crate_name: &str,
    crate_dir: &Path,
    marker_api_version: &str,
) -> ui_test::color_eyre::Result<ui_test::Config> {
    let setup = retrieve_test_setup(crate_name, &std::fs::canonicalize(crate_dir)?);
    verify_driver(&setup, marker_api_version);

    // Set environment values
    for (key, val) in setup.env_vars {
        std::env::set_var(key, val);
    }

    // Create config
    let mut config = ui_test::Config {
        mode: ui_test::Mode::Yolo,
        num_test_threads: NonZeroUsize::new(1).unwrap(),
        ..ui_test::Config::rustc(ui_dir)
    };

    config.program.program = PathBuf::from(setup.rustc_path);
    config.program.args.push("-Aunused".into());

    // hide binaries generated for successfully passing tests
    let tmp_dir = tempfile::tempdir_in(target_dir)?;
    let tmp_dir = tmp_dir.path();
    config.out_dir = tmp_dir.into();
    config.path_stderr_filter(tmp_dir, "$TMP");

    Ok(config)
}

/// This function calls `cargo-marker` for the basic test setup.
fn retrieve_test_setup(crate_name: &str, pkg_dir: &Path) -> TestSetup {
    #[cfg(not(feature = "dev-build"))]
    const CARGO_MARKER_INVOCATION: &[&str] = &["marker"];
    #[cfg(feature = "dev-build")]
    const CARGO_MARKER_INVOCATION: &[&str] = &["run", "--bin", "cargo-marker", "--"];

    #[cfg(not(feature = "dev-build"))]
    let command_dir = pkg_dir;
    #[cfg(feature = "dev-build")]
    let command_dir = pkg_dir.parent().unwrap();

    // This needs to use ' string limiters for the path, to prevent `\\` escaping on windows...
    let lint_spec = format!(r#"{} = {{ path = '{}' }}"#, crate_name, pkg_dir.display());
    let mut cmd = Command::new("cargo");
    let output = cmd
        .current_dir(command_dir)
        .args(CARGO_MARKER_INVOCATION)
        .arg("test-setup")
        .arg("-l")
        .arg(lint_spec)
        .output()
        .expect("Unable to run the test setup using `cargo-marker`");
    let stdout = String::from_utf8(output.stdout).unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        if stderr.starts_with("error: no such command") {
            panic!("{NO_SUCH_COMMENT_ADVICE}");
        }
        panic!("Test setup failed:\n\n===STDOUT===\n{stdout}\n\n===STDERR===\n{stderr}\n");
    }

    let info_vars: HashMap<_, _> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("info:"))
        .filter_map(|line| line.split_once('='))
        .map(|(var, value)| (var.to_string(), value.to_string()))
        .collect();

    let mut env_vars: HashMap<_, _> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("env:"))
        .filter_map(|line| line.split_once('='))
        .map(|(var, value)| (var.to_string(), value.to_string()))
        .collect();
    let toolchain = info_vars
        .get("toolchain")
        .expect("missing info field 'toolchain'")
        .clone();
    let marker_api = info_vars
        .get("marker-api")
        .expect("missing info field 'marker-api'")
        .clone();

    TestSetup {
        rustc_path: env_vars.remove("RUSTC_WORKSPACE_WRAPPER").unwrap(),
        env_vars,
        toolchain,
        marker_api,
    }
}

const NO_SUCH_COMMENT_ADVICE: &str = r#"
===========================================================

Error: Command `marker` was not found

UI tests require `cargo-marker` to be installed

* Try installing `cargo-marker`

    ```
    # Update `cargo-marker` first
    cargo install cargo_marker

    # Now update the driver
    cargo marker setup --auto-install-toolchain
    ```

===========================================================
"#;

const DRIVER_FAIL_ADVICE: &str = r#"
===========================================================

Error: Unable to start Marker's driver

UI tests need to be executed with the nightly version of the driver

* Try setting the version in a `rust-toolchain.toml` file, like this:
    ```
    [toolchain]
    channel = "{toolchain}"
    ```

* Try setting the channel when invoking the tests, like this:
    ```
    cargo +{toolchain} test"
    ```

===========================================================
"#;

const VERSION_LESS_ADVICE: &str = r#"
===========================================================

Error: API versions mismatch, the lint crate is behind the driver.

* Try updating the used api version to: `{marker_api}`

===========================================================
"#;

const VERSION_GREATER_ADVICE: &str = r#"
===========================================================

Error: API versions mismatch, the lint crate uses a newer version

* Try updating the driver.

    ```
    # Update `cargo-marker` first
    cargo install cargo_marker

    # Now update the driver
    cargo marker setup --auto-install-toolchain
    ```

    You might also need to update the used toolchain. In that case a new error
    will be emitted.

===========================================================
"#;

// FIXME(xFrednet): It would be better to return the error messages as a result
// instead of panicking
fn verify_driver(setup: &TestSetup, marker_api_version: &str) {
    // Check that the correct channel is used
    let test = Command::new(&setup.rustc_path)
        .arg("-V")
        .spawn()
        .expect("failed to start marker's driver")
        .wait()
        .expect("failed to wait for marker's driver");
    if !test.success() {
        panic!("{}", DRIVER_FAIL_ADVICE.replace("{toolchain}", &setup.toolchain));
    }

    // Check the versions match
    let this_version = Version::parse(marker_api_version).unwrap();
    let driver_version = Version::parse(&setup.marker_api).unwrap();

    match this_version.cmp(&driver_version) {
        std::cmp::Ordering::Less => {
            panic!("{}", VERSION_LESS_ADVICE.replace("{marker_api}", &setup.marker_api));
        },
        std::cmp::Ordering::Equal => {
            // Perfection, everything is beautiful!!
        },
        std::cmp::Ordering::Greater => {
            panic!("{VERSION_GREATER_ADVICE}");
        },
    }
}
