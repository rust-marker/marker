use std::{env, num::NonZeroUsize, path::PathBuf};
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

    let bless = std::env::var("BLESS").eq(&Ok("1".to_string()));
    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless
    }

    config.stderr_filter("in ([0-9]m )?[0-9\\.]+s", "");
    config.stdout_filter("in ([0-9]m )?[0-9\\.]+s", "");
    config.stderr_filter(r"[^ ]*/\.?cargo/registry/.*/", "$$CARGO_REGISTRY");
    config.path_stderr_filter(&std::path::Path::new(path), "$DIR");

    // hide binaries generated for successfully passing tests
    let tmp_dir = tempfile::tempdir_in(path)?;
    let tmp_dir = tmp_dir.path();
    config.out_dir = tmp_dir.into();
    config.path_stderr_filter(tmp_dir, "$TMP");

    run_tests_generic(
        config,
        default_file_filter,
        default_per_file_config,
        // Avoid github actions, as these would end up showing up in `Cargo.stderr`
        status_emitter::Text,
    )
}
