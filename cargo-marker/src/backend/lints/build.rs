use super::{LintCrate, LintCrateSource};
use crate::backend::Config;
use crate::error::prelude::*;
use crate::observability::prelude::*;
use itertools::Itertools;
use std::{collections::HashSet, ffi::OsStr, path::Path};
use yansi::Paint;

#[cfg(target_os = "linux")]
const DYNAMIC_LIB_FILE_ENDING: &str = "so";
#[cfg(target_os = "macos")]
const DYNAMIC_LIB_FILE_ENDING: &str = "dylib";
#[cfg(target_os = "windows")]
const DYNAMIC_LIB_FILE_ENDING: &str = "dll";

/// A list of file endings which are expected to be inside the lint crate dir.
/// It's assumed that these can be safely removed.
const ARTIFACT_ENDINGS: &[&str] = &[
    DYNAMIC_LIB_FILE_ENDING,
    #[cfg(target_os = "windows")]
    "exp",
    #[cfg(target_os = "windows")]
    "lib",
    #[cfg(target_os = "windows")]
    "pdb",
];

pub fn build_lints(sources: &[LintCrateSource], config: &Config) -> Result<Vec<LintCrate>> {
    // By default Cargo doesn't provide the path of the compiled lint crate.
    // As a work around, we use the `--out-dir` option to make cargo copy all
    // created binaries into one folder. We then scan that folder as we build our crates
    // and collect all dynamic libraries that show up, we link the name of the crate we just built
    // with the file(s) we found after we built it, assuming that they are related.
    //
    // TODO Look into how to optimize this a bit better:
    // Another option that would be potentially more performant is if we built each
    // crate in it's own target dir. this would eliminate the need for HashSet<_> below, without
    // changing too much else about the implementation.
    //
    // This would be so much simpler if we could get an output name from Cargo

    // Clear previously build lints
    let lints_dir = config.lint_crate_dir();
    clear_lints_dir(&lints_dir)?;

    // Build lint crates and find the output of those builds
    let mut found_paths = HashSet::new();
    let ending = OsStr::new(DYNAMIC_LIB_FILE_ENDING);
    let mut lints = Vec::with_capacity(sources.len());

    for lint_src in sources {
        build_lint(lint_src, config)?;
        match std::fs::read_dir(&lints_dir) {
            Ok(dir) => {
                for file in dir {
                    let file = file.unwrap().path();
                    if file.extension() == Some(ending) && !found_paths.contains(&file) {
                        found_paths.insert(file.clone());
                        lints.push(LintCrate {
                            file,
                            name: lint_src.name.clone(),
                        });
                    }
                }
            },
            Err(err) => {
                // This shouldn't really be a point of failure. In this case, I'm
                // more interested in the HOW?
                panic!(
                    "unable to read lints dir after lint compilation: {} ({err:#?})",
                    lints_dir.display()
                );
            },
        }
    }
    Ok(lints)
}

/// This function clears the `marker/lints` directory holding all compiled lints. This
/// is required, as Marker uses the content of that directory to determine which lints
/// should be run.
///
/// This is an extra function to not call `delete_dir_all` and just accidentally delete
/// the entire system.
fn clear_lints_dir(lints_dir: &Path) -> Result {
    // Delete all files
    let dir = match std::fs::read_dir(lints_dir) {
        Ok(dir) => dir,
        Err(err) if std::io::ErrorKind::NotFound == err.kind() => return Ok(()),
        Err(err) => return Err(Error::wrap(err, "Failed to read lints artifacts directory")),
    };

    let endings: Vec<_> = ARTIFACT_ENDINGS.iter().map(OsStr::new).collect();

    let (files, errors): (Vec<_>, Vec<_>) = dir.map(|result| result.map_err(Error::transparent)).partition_result();

    if !errors.is_empty() {
        return Err(Error::many(errors, "Failed to read the lints directory entries"));
    }

    for file in files {
        let file = file.path();

        let is_expected_ending = file.extension().map(|ending| endings.contains(&ending)) == Some(true);

        if !is_expected_ending {
            return Err(Error::root(format!(
                "Marker's lint directory contains an unexpected file: {}",
                file.display()
            )));
        }

        std::fs::remove_file(&file)
            .context(|| format!("Failed to remove the lint artifact file {}", file.display()))?;
    }

    // The dir should now be empty
    std::fs::remove_dir(lints_dir).context(|| format!("Failed to remove lints directory {}", lints_dir.display()))
}

fn build_lint(lint_src: &LintCrateSource, config: &Config) -> Result {
    let mut cmd = config.toolchain.cargo_build_command(config, &lint_src.manifest);

    // Set output dir. This currently requires unstable options
    cmd.arg("-Z");
    cmd.arg("unstable-options");
    cmd.arg("--out-dir");
    cmd.arg(config.lint_crate_dir().as_os_str());

    let exit_status = cmd
        .log()
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        return Ok(());
    }

    Err(Error::root(format!(
        "Failed to compile the lint crate {}",
        lint_src.name.red().bold()
    )))
}
