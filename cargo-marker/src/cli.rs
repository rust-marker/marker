mod check;
mod setup;
mod test_setup;

use crate::config::Config;
use crate::error::prelude::*;
use clap::{Parser, Subcommand};

/// Marker's CLI interface
///
/// This binary should be invoked by Cargo with the new `marker` subcommand. If
/// you're reading this, consider manually adding `marker` as the first argument.
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(author, version, about)]
struct CargoPlugin {
    #[clap(subcommand)]
    subcommand: MarkerSubcommand,
}

#[derive(Parser, Debug)]
enum MarkerSubcommand {
    Marker(MarkerCli),
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(override_usage = "cargo marker [OPTIONS] [COMMAND] -- <CARGO ARGS>")]
pub(crate) struct MarkerCli {
    #[command(subcommand)]
    pub(crate) command: Option<CliCommand>,

    /// Used as the arguments to run Marker, when no command was specified
    #[clap(flatten)]
    pub(crate) check: check::CheckCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CliCommand {
    /// Run Marker on the current package
    Check(check::CheckCommand),

    /// Setup the rustc driver for Marker
    Setup(setup::SetupCommand),

    /// **UNSTABLE** Setup the specified lint crate for ui tests
    #[command(hide = true)]
    TestSetup(test_setup::TestSetupCommand),
}

impl MarkerCli {
    /// Prefer using this over the normal `parse` method, to split the arguments
    pub(crate) fn parse_args() -> Self {
        let MarkerSubcommand::Marker(cli) = CargoPlugin::parse().subcommand;
        cli
    }

    pub(crate) fn run(self) -> Result {
        let manifest_path = crate::backend::cargo::Cargo::default().cargo_locate_project()?;
        let config = Config::try_from_manifest(&manifest_path)?;

        let Some(command) = self.command else {
            return self.check.run(config);
        };
        match command {
            CliCommand::Setup(cmd) => cmd.run(),
            CliCommand::Check(cmd) => cmd.run(config),
            CliCommand::TestSetup(cmd) => cmd.run(config),
        }
    }
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
    fn test_marker_cli() {
        let cli = MarkerCli::parse_from(["cargo-marker", "check"]);
        assert!(matches!(cli.command, Some(CliCommand::Check(_))));

        let cli = MarkerCli::parse_from(["cargo-marker"]);
        assert!(cli.command.is_none());
        assert!(cli.check.cargo_args.is_empty());

        let cli = MarkerCli::parse_from(["cargo-marker", "--", "ducks", "penguins"]);
        assert!(cli.command.is_none());
        assert!(cli.check.cargo_args.len() == 2);
        assert!(cli.check.cargo_args[0] == "ducks");
        assert!(cli.check.cargo_args[1] == "penguins");

        let cli = MarkerCli::parse_from(["cargo-marker", "check", "--", "ducks", "penguins"]);
        assert!(cli.check.cargo_args.is_empty());
        if let Some(CliCommand::Check(check_args)) = cli.command {
            assert!(check_args.cargo_args.len() == 2);
            assert!(check_args.cargo_args[0] == "ducks");
            assert!(check_args.cargo_args[1] == "penguins");
        } else {
            panic!("the `check` subcommand was not detected");
        }
    }
}
