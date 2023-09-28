use crate::observability::prelude::*;
use itertools::{Either, Itertools};
use std::ffi::OsStr;
use std::fmt;
use yansi::Paint;

/// Display a simple CLI string snippet. It's not as robust as [`CommandExt::display`],
/// but works for cases where we want to suggest the user to run a command in a help
/// message for example.
pub(crate) fn cli(cli: &str) -> String {
    shlex::split(cli)
        .unwrap_or_else(|| {
            panic!(
                "BUG: invalid CLI string; it is supposed to be a valid \
                string from the trusted source, but got:\n---\n{cli}\n---"
            )
        })
        .into_iter()
        .enumerate()
        .map(|(i, arg)| {
            if i == 0 {
                return bin(arg).to_string();
            }

            if arg.starts_with('-') {
                return named_arg(arg).to_string();
            }

            positional_arg(arg).to_string()
        })
        .join(" ")
}

pub(crate) trait CommandExt {
    /// Builds on top of [`CommandExt::display`] and logs the command with
    /// the `info` level using [`tracing`].
    fn log(&mut self) -> &mut Self {
        info!(target: "cli", "{}", self.display());
        self
    }

    /// Returns a value that can be used to display the command in a nice way.
    /// It displays it in such a way that it is most likely possible to copy
    /// the command and paste it in the terminal to reproduce the same command.
    fn display(&self) -> DisplayCommand<'_>;
}

impl CommandExt for std::process::Command {
    fn display(&self) -> DisplayCommand<'_> {
        DisplayCommand { cmd: self }
    }
}

pub(crate) struct DisplayCommand<'a> {
    cmd: &'a std::process::Command,
}

fn operator(val: &str) -> impl fmt::Display + '_ {
    yansi::Painted::new(val).bold().red()
}
fn env_var(val: impl AsRef<OsStr>) -> impl fmt::Display {
    yansi::Painted::new(quote(val)).cyan()
}
fn bin(val: impl AsRef<OsStr>) -> impl fmt::Display {
    yansi::Painted::new(quote(val)).green().bold()
}
fn quote(val: impl AsRef<OsStr>) -> String {
    shlex::quote(&val.as_ref().to_string_lossy()).into_owned()
}
fn positional_arg(val: impl AsRef<OsStr>) -> impl fmt::Display {
    yansi::Painted::new(quote(val)).yellow()
}
fn named_arg(key: impl AsRef<OsStr>) -> impl fmt::Display {
    yansi::Painted::new(quote(key)).blue()
}

impl fmt::Display for DisplayCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cmd = self.cmd;

        write!(f, "{} ", "❱".green().bold())?;

        if let Some(current_dir) = &cmd.get_current_dir() {
            write!(f, "{} {} {} ", bin("cd"), positional_arg(current_dir), operator("&&"))?;
        }

        let mut envs = cmd.get_envs().peekable();
        if envs.peek().is_some() {
            let (set_vars, unset_vars): (Vec<_>, Vec<_>) = envs.partition_map(|(key, val)| {
                val.map(|val| Either::Left((key, val)))
                    .unwrap_or_else(|| Either::Right(key))
            });

            let set_vars = set_vars.iter().format_with(" ", |(key, val), f| {
                f(&format_args!("{}{}{}", env_var(key), operator("="), env_var(val)))
            });

            write!(f, "{set_vars} ")?;

            if !unset_vars.is_empty() {
                let unset_vars = unset_vars.iter().format_with(" ", |key, f| {
                    f(&format_args!("{} {}", named_arg("--unset"), env_var(key)))
                });

                write!(f, "{} {unset_vars} ", bin("env"))?;
            }
        }

        write!(f, "{}", bin(&cmd.get_program()))?;

        let mut args = cmd.get_args().peekable();
        if args.peek().is_some() {
            let args = args.format_with(" ", |arg, f| {
                if arg.to_string_lossy().starts_with('-') {
                    f(&named_arg(arg))
                } else {
                    f(&positional_arg(arg))
                }
            });

            write!(f, " {args}")?;
        }

        Ok(())
    }
}
