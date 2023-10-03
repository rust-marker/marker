use marker_uitest::ui_test::*;
use std::env;

fn main() -> color_eyre::Result<()> {
    let mut config = marker_uitest::simple_ui_test_config!("tests/ui", "../target")?;

    config.dependencies_crate_manifest_path = Some("./Cargo.toml".into());

    config.filter(r"\\/", "/");
    config.filter(r"\\\\", "/");

    run_tests_generic(
        vec![config],
        default_file_filter,
        default_per_file_config,
        status_emitter::Text::quiet(),
    )
}
