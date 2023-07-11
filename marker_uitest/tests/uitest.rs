use std::{
    env,
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use ui_test::*;

#[test]
fn ui_test() -> ui_test::color_eyre::Result<()> {
    let path = "../target";

    let setup = cargo_marker::test_setup(
        env!("CARGO_PKG_NAME").to_string(),
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    )
    .unwrap();
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
