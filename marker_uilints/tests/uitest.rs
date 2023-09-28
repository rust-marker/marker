use marker_uitest::ui_test::*;
use std::env;

fn main() -> color_eyre::Result<()> {
    let mut config = marker_uitest::simple_ui_test_config!("tests/ui", "../target")?;

    let bless = env::var_os("BLESS").is_some() || env::args().any(|arg| arg == "--bless");
    if bless {
        config.output_conflict_handling = OutputConflictHandling::Bless
    }

    let filters = [
        // Normalization for windows...
        (r"ui//", "ui/"),
        (r"item//", "item/"),
        (r"expr//", "expr/"),
        (r"sugar//", "sugar/"),
    ];
    for (pat, repl) in filters {
        config.stderr_filter(pat, repl);
        config.stdout_filter(pat, repl);
    }

    run_tests_generic(
        vec![config],
        default_file_filter,
        default_per_file_config,
        status_emitter::Text::quiet(),
    )
}
