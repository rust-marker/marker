use marker_uitest::ui_test::*;
use std::{env, path::Path};

fn main() -> color_eyre::Result<()> {
    let mut config = marker_uitest::simple_ui_test_config!("tests/ui", "../target")?;

    let bless = env::var_os("BLESS").is_some() || env::args().any(|arg| arg == "--bless");
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
