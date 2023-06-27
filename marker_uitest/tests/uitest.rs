use std::{
    collections::HashMap,
    env, fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    process::Command,
};
use ui_test::*;

#[test]
fn ui_test() -> ui_test::color_eyre::Result<()> {
    let path = "../target";

    let setup = run_test_setup();
    for (key, val) in setup.env_vars {
        env::set_var(key, val);
    }

    let mut config = Config {
        mode: Mode::Yolo,
        output_conflict_handling: OutputConflictHandling::Error("BLESS=1 cargo uitest".into()),
        dependencies_crate_manifest_path: Some("./Cargo.toml".into()),
        num_test_threads: NonZeroUsize::new(1).unwrap(),
        ..Config::rustc("tests/ui".into())
    };

    config.program.program = PathBuf::from(setup.rustc_path);
    config.program.args.push("-Aunused".into());

    let bless = std::env::var("BLESS").eq(&Ok("1".to_string()));
    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless
    }

    let filters = [
        // Normalization for windows...
        (r"ui//", "ui/"),
    ];
    for (pat, repl) in filters {
        config.stderr_filter(pat, repl);
        config.stdout_filter(pat, repl);
    }

    // hide binaries generated for successfully passing tests
    let tmp_dir = tempfile::tempdir_in(path)?;
    let tmp_dir = tmp_dir.path();
    config.out_dir = tmp_dir.into();
    config.path_stderr_filter(tmp_dir, "$TMP");

    let test_name_filter = test_name_filter();
    run_tests_generic(
        config,
        move |path| default_file_filter(path) && test_name_filter(path),
        default_per_file_config,
        status_emitter::Text,
    )
}

// Gracefully stolen from Clippy, thank you!
fn test_name_filter() -> Box<dyn Fn(&Path) -> bool + Sync> {
    if let Ok(filters) = env::var("TESTNAME") {
        let filters: Vec<_> = filters.split(',').map(ToString::to_string).collect();
        Box::new(move |path| {
            filters.is_empty()
                || filters
                    .iter()
                    .any(|f| path.file_stem().map_or(false, |stem| stem == f.as_str()))
        })
    } else {
        Box::new(|_| true)
    }
}

struct TestSetup {
    rustc_path: String,
    /// The environment values that should be set. The first element is the
    /// value name, the second is the value the it should be set to.
    env_vars: HashMap<String, String>,
}

/// This function calls `cargo-marker` for the basic test setup. For normal linting
/// crates this will need to be adjusted to run the installed `cargo-marker` version
///
/// This function is currently slow and hacky. marker#155 should clean this up and
/// give us a speed up.
///
/// In the future it would be nice to have a nice wrapper library as well.
fn run_test_setup() -> TestSetup {
    const CARGO_MARKER_INVOCATION: &[&str] = &["run", "--bin", "cargo-marker", "--features", "dev-build", "--"];

    // ../rust-marker/marker_uitest
    let current_dir = env::current_dir().unwrap();
    let lint_crate_src = fs::canonicalize(&current_dir).unwrap();
    let mut cmd = Command::new("cargo");
    let output = cmd
        .current_dir(current_dir.parent().unwrap())
        .args(CARGO_MARKER_INVOCATION)
        .arg("-l")
        .arg(lint_crate_src)
        .arg("--test-setup")
        .output()
        .expect("Unable to run the test setup using `cargo-marker`");
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
