//! This test runs the `linter_lints` crate on each file inside the `ui` directory
//! and compares the output to the `.stderr` file next to it.
//!
//! * To update the output files, you can set the environment value `BLESS` to `1`.
//! * Specific test files can be selected with the `TESTNAME` environment value.

#![feature(test)] // compiletest_rs requires this attribute

use compiletest_rs as compiletest;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

#[test]
fn ui_test() {
    let config = base_config("ui");
    compiletest::run_tests(&config);
}

fn base_config(test_dir: &str) -> compiletest::Config {
    let setup = run_test_setup();
    for (key, val) in setup.env_vars {
        env::set_var(key, val);
    }

    let bless = std::env::var("BLESS").eq(&Ok("1".to_string()));
    let mut config = compiletest::Config {
        bless,
        edition: Some("2021".into()),
        mode: compiletest::common::Mode::Ui,
        ..compiletest::Config::default()
    };

    if let Ok(filters) = env::var("TESTNAME") {
        config.filters = filters.split(',').map(ToString::to_string).collect();
    }

    config.src_base = Path::new("tests").join(test_dir);
    config.rustc_path = PathBuf::from(setup.rustc_path);
    config
}

struct TestSetup {
    rustc_path: String,
    /// The environment values that should be set. The first element is the
    /// value name, the second is the value the it should be set to.
    env_vars: HashMap<String, String>,
}

/// This function calls `cargo-linter` for the basic test setup. For normal linting
/// crates this will need to be adjusted to run the installed `cargo-linter` version
/// and to build the local crate and not assume that we're in a cargo workspace.
///
/// In the future this function should probably be hidden in a nice wrapper library.
fn run_test_setup() -> TestSetup {
    const CARGO_LINTER_INVOCATION: &[&str] = &["run", "--bin", "cargo-linter", "--"];

    // ../rust-linting/linter_lints
    let current_dir = env::current_dir().unwrap();
    let lint_crate_src = fs::canonicalize(&current_dir).unwrap();
    let mut cmd = Command::new("cargo");
    let output = cmd
        .current_dir(current_dir.parent().unwrap())
        .args(CARGO_LINTER_INVOCATION)
        .args(["-l", &lint_crate_src.display().to_string(), "--test-setup"])
        .output()
        .expect("Unable to run the test setup using `cargo-linter`");
    let stdout = String::from_utf8(output.stdout).unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).unwrap();
        panic!("Test setup failed:\n\n===STDOUT===\n{stdout}\n\n===STDERR===\n{stderr}\n");
    }

    let mut env_vars: HashMap<_, _> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("env:"))
        .filter_map(|line| line.split_once('='))
        .map(|(var, val)| (var.to_string(), val.to_string()))
        .collect();

    TestSetup {
        rustc_path: env_vars.remove("RUSTC_WORKSPACE_WRAPPER").unwrap(),
        env_vars,
    }
}
