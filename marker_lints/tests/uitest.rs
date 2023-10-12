use marker_uitest::ui_test::*;
use std::env;

fn main() -> color_eyre::Result<()> {
    let mut config: Config = marker_uitest::simple_ui_test_config!("tests/ui", "../target")?;

    config.dependencies_crate_manifest_path = Some("./Cargo.toml".into());

    // `marker_api::declare_lint` uses the `CARGO_CRATE_NAME` environment value.
    // Setting it here once, makes sure that it'll be available, during the
    // compilation of all ui test files.
    std::env::set_var("CARGO_CRATE_NAME", "marker_lints_uitests");

    config.filter(r"\\/", "/");
    config.filter(r"\\\\", "/");

    run_tests_generic(
        vec![config],
        default_file_filter,
        default_per_file_config,
        status_emitter::Text::quiet(),
    )
}
