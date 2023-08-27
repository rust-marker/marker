mod cli;

pub(crate) use cli::*;

use itertools::Itertools;
use std::fmt;
use yansi::Paint;

/// Lightweight and dumb highlighter for TOML. It doesn't actually parse TOML
/// correctly, but it's good enough for simple messages that need to present
/// an example TOML snippet.
pub(crate) fn toml(toml: &str) -> String {
    toml.lines()
        .map(|line| {
            let punct = ['[', ']', '{', '}', '='];

            let line = punct.into_iter().fold(line.to_string(), |line, punct| {
                line.replace(punct, &punct.yellow().to_string())
            });

            if line.trim().is_empty() {
                return line.to_string();
            }

            if line.starts_with('#') {
                return line.black().to_string();
            }

            if let Some((key, val)) = line.split_once('=') {
                return format!("{}{}{}", key.cyan().bold(), "=".yellow(), val.blue());
            }

            line.to_string()
        })
        .join("\n")
}

/// Displays a stage of work that `cargo-marker` performs.
/// This is specifically formatted to be aligned with the other cargo
/// output like `Compiling` or `Checking`.
pub(crate) fn print_stage(name: &str) {
    println!("      {} {}", "Marker".bold().green(), stage(name));
}

pub(crate) fn stage(name: &str) -> impl fmt::Display + '_ {
    name.white().bold()
}
