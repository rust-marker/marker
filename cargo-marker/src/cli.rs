use clap::{builder::ValueParser, Arg, ArgAction, Command};

use crate::VERSION;

const AFTER_HELP_MSG: &str = r#"CARGO ARGS
    All arguments after double dashes(`--`) will be passed to cargo.
    These options are the same as for `cargo check`.

EXAMPLES:
    * `cargo marker -l ./marker_lints`
"#;

pub fn get_clap_config() -> Command {
    Command::new(VERSION)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(ArgAction::SetTrue)
                .help("Print version info and exit"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Print additional debug information to the console"),
        )
        .arg(
            Arg::new("test-setup")
                .long("test-setup")
                .action(ArgAction::SetTrue)
                .help("This flag will compile the lint crate and print all relevant environment values"),
        )
        .subcommand(setup_command())
        .subcommand(check_command())
        .args(check_command_args())
        .after_help(AFTER_HELP_MSG)
        .override_usage("cargo-marker [OPTIONS] -- <CARGO ARGS>")
}

fn setup_command() -> Command {
    Command::new("setup")
        .about("A collection of commands to setup marker")
        .after_help("By default this will install the driver for rustc.")
}

fn check_command() -> Command {
    Command::new("check")
        .about("Run marker on a local package")
        .args(check_command_args())
}

fn check_command_args() -> impl IntoIterator<Item = impl Into<Arg>> {
    vec![
        Arg::new("lints")
            .short('l')
            .long("lints")
            .num_args(1..)
            .value_parser(ValueParser::os_string())
            .help("Defines a set of lint crates that should be used"),
    ]
}
