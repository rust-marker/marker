use std::collections::HashMap;

use clap::{Args, Parser, Subcommand};

const CARGO_ARGS_SEPARATOR: &str = "--";

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(after_help = AFTER_HELP_MSG)]
#[command(override_usage = "cargo marker [OPTIONS] [COMMAND] -- <CARGO ARGS>")]
pub struct MarkerCli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// Used as the arguments to run Marker, when no command was specified
    #[clap(flatten)]
    pub check_args: CheckArgs,
}

impl MarkerCli {
    /// Prefer using this over the normal `parse` method, to split the arguments
    pub fn parse_args() -> Self {
        let (marker_args, cargo_args) = split_args(std::env::args());
        let mut cli = Self::parse_from(marker_args);

        match &mut cli.command {
            Some(CliCommand::Check(check_args) | CliCommand::TestSetup(check_args)) => {
                check_args.cargo_args = cargo_args;
            },
            Some(CliCommand::Setup(_)) => {},
            None => cli.check_args.cargo_args = cargo_args,
        }

        cli
    }
}

/// This reads the arguments from the
fn split_args<I>(args: I) -> (Vec<String>, Vec<String>)
where
    I: IntoIterator<Item = String>,
{
    let mut marker_args: Vec<_> = args
        .into_iter()
        // The second argument might be `marker` if `cargo-marker` was
        // invoked as a subcommand by cargo. This filters out the name.
        .enumerate()
        .filter_map(|(index, value)| (!(index == 1 && value == "marker")).then_some(value))
        .collect();

    let cargo_args = if let Some(index) = marker_args.iter().position(|val| val == CARGO_ARGS_SEPARATOR) {
        let mut cargo_args = marker_args.split_off(index);
        // Remove `CARGO_ARGS_SEPARATOR` from the start
        cargo_args.remove(0);
        cargo_args
    } else {
        vec![]
    };

    (marker_args, cargo_args)
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Run Marker on the current package
    Check(CheckArgs),
    /// Setup the rustc driver for Marker
    Setup(SetupArgs),
    /// **UNSTABLE** Setup the specified lint crate for ui tests
    #[command(hide = true)]
    TestSetup(CheckArgs),
}

#[derive(Args, Debug)]
#[command(override_usage = "cargo marker check [OPTIONS] -- <CARGO ARGS>")]
pub struct CheckArgs {
    /// Specifies lint crates which should be used. (Lints in `Cargo.toml` will be ignored)
    #[arg(short, long)]
    pub lints: Vec<String>,
    /// Forwards the current `RUSTFLAGS` value during driver and lint crate compilation
    #[arg(long, default_value_t = false)]
    pub forward_rust_flags: bool,

    /// Arguments specified after double dashes, which should be forwarded
    #[clap(skip)]
    pub cargo_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct SetupArgs {
    /// Automatically installs the required toolchain using rustup
    #[arg(long, default_value_t = false)]
    pub auto_install_toolchain: bool,
    /// Forwards the current `RUSTFLAGS` value during driver and lint crate compilation
    #[arg(long, default_value_t = false)]
    pub forward_rust_flags: bool,
}

use crate::{
    config::{Config, ConfigFetchError, LintDependency},
    ExitStatus,
};

const AFTER_HELP_MSG: &str = r#"CARGO ARGS
    All arguments after double dashes(`--`) will be passed to cargo.
    Run `cargo check --help` to see these options.
"#;

pub fn collect_lint_deps(args: &CheckArgs) -> Result<HashMap<String, LintDependency>, ExitStatus> {
    if args.lints.is_empty() {
        return Err(ExitStatus::NoLints);
    }

    let mut virtual_manifest = "[workspace.metadata.marker.lints]\n".to_string();
    for dep in &args.lints {
        virtual_manifest.push_str(dep);
        virtual_manifest.push('\n');
    }

    let Config { lints } = Config::try_from_str(&virtual_manifest).map_err(ConfigFetchError::emit_and_convert)?;
    Ok(lints)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        MarkerCli::command().debug_assert();
    }

    #[test]
    fn test_split_args() {
        let (a, b) = split_args(vec![]);
        assert_eq!(a, &[] as &[&str]);
        assert_eq!(b, &[] as &[&str]);

        let (a, b) = split_args(["check", "--something", "else"].iter().map(ToString::to_string));
        assert_eq!(a, &["check", "--something", "else"] as &[&str]);
        assert_eq!(b, &[] as &[&str]);

        let (a, b) = split_args(["check", "--", "cargo", "magic"].iter().map(ToString::to_string));
        assert_eq!(a, &["check"] as &[&str]);
        assert_eq!(b, &["cargo", "magic"] as &[&str]);

        let (a, b) = split_args(["check", "--"].iter().map(ToString::to_string));
        assert_eq!(a, &["check"] as &[&str]);
        assert_eq!(b, &[] as &[&str]);
    }
}
