//! This test runs the `linter_lints` crate on each file inside the `ui` directory
//! and compares the output to the `.stderr` file next to it.

#![feature(test)] // compiletest_rs requires this attribute

use compiletest_rs as compiletest;

use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;

#[test]
fn ui_test() {
    compile_driver();
    run_ui();
}

fn run_ui() {
    let config = base_config("ui");
    compiletest::run_tests(&config);
}

fn base_config(test_dir: &str) -> compiletest::Config {
    let mut config = compiletest::Config {
        edition: Some("2021".into()),
        mode: compiletest::common::Mode::Ui,
        ..compiletest::Config::default()
    };

    if let Ok(filters) = env::var("TESTNAME") {
        config.filters = filters.split(',').map(ToString::to_string).collect();
    }

    config.src_base = Path::new("tests").join(test_dir);
    config.rustc_path = get_rustc_driver_path();
    config
}

fn compile_driver() {
    println!();
    println!("Compiling Driver:");
    let mut cmd = Command::new("cargo");
    let exit_status = cmd
        .args(["build", "-p", "linter_driver_rustc"])
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    assert!(exit_status.success(), "Failed to compile `linter_driver_rustc`");
}

fn get_rustc_driver_path() -> PathBuf {
    // `.../rust-linting/target/debug/deps/compile_test-<hash>`
    let current_exe_path = env::current_exe().unwrap();
    println!("{current_exe_path:?}");
    
    // `.../rust-linting/target/debug/linter_driver_rustc`
    #[cfg(not(target_os = "windows"))]
    let driver_file = "linter_driver_rustc";
    #[cfg(target_os = "windows")]
    let driver_file = "linter_driver_rustc.exe";
    current_exe_path.parent().unwrap().with_file_name(driver_file)
}
