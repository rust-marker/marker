use std::ffi::OsStr;

use crate::{backend::Config, ExitStatus};

use super::{LintCrate, LintCrateSource};

#[cfg(target_os = "linux")]
const DYNAMIC_LIB_FILE_ENDING: &str = "so";
#[cfg(target_os = "macos")]
const DYNAMIC_LIB_FILE_ENDING: &str = "dylib";
#[cfg(target_os = "windows")]
const DYNAMIC_LIB_FILE_ENDING: &str = "dll";

pub fn build_lints(sources: &[LintCrateSource], config: &Config) -> Result<Vec<LintCrate>, ExitStatus> {
    // By default Cargo doesn't provide the path of the compiled lint crate.
    // As a work around, we use the `--out-dir` option to make cargo copy all
    // created binaries into one folder. We then scan that folder and collect
    // all dynamic libraries, assuming that they're lint crates.

    // Build lint crates
    for lint_src in sources {
        build_lint(lint_src, config)?;
    }

    // Find lint lint crates
    let lints_dir = config.lint_crate_dir();
    match std::fs::read_dir(&lints_dir) {
        Ok(dir) => {
            let ending = OsStr::new(DYNAMIC_LIB_FILE_ENDING);
            let mut lints = vec![];
            for file in dir {
                let file = file.unwrap().path();
                if file.extension() == Some(ending) {
                    lints.push(LintCrate { file });
                }
            }
            Ok(lints)
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

fn build_lint(lint_src: &LintCrateSource, config: &Config) -> Result<(), ExitStatus> {
    let mut cmd = config.toolchain.cargo_build_command(config, &lint_src.manifest);

    // Set output dir. This currently requires unstable options
    cmd.arg("-Z");
    cmd.arg("unstable-options");
    cmd.arg("--out-dir");
    cmd.arg(config.lint_crate_dir().as_os_str());

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if !exit_status.success() {
        return Err(ExitStatus::LintCrateBuildFail);
    }

    Ok(())
}
