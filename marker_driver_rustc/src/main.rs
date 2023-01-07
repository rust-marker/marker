#![doc = include_str!("../README.md")]
#![feature(rustc_private)]
#![feature(lint_reasons)]
#![feature(once_cell)]
#![warn(rustc::internal)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(
    clippy::needless_lifetimes,
    reason = "some lifetimes will be required to fix ICEs, <'ast, '_> also looks weird"
)]
#![allow(clippy::needless_collect, reason = "it has false positives for `alloc_slice_iter`")]
#![allow(
    clippy::too_many_lines,
    reason = "long functions are sometimes unavoidable for matches"
)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

pub mod context;
pub mod conversion;
mod entry;

use std::env;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const RUSTC_TOOLCHAIN_VERSION: &str = "nightly-2022-12-15";

struct DefaultCallbacks;
impl rustc_driver::Callbacks for DefaultCallbacks {}

struct MarkerCallback;

impl rustc_driver::Callbacks for MarkerCallback {
    fn config(&mut self, config: &mut rustc_interface::Config) {
        // Clippy explicitly calls any previous functions. This will not be
        // done here to keep it simple and to ensure that only known code is
        // executed.
        assert!(config.register_lints.is_none());

        config.register_lints = Some(Box::new(|_sess, lint_store| {
            lint_store.register_late_pass(|_| Box::new(entry::MarkerLintPass));
        }));
    }
}

/// If a command-line option matches `find_arg`, then apply the predicate `pred` on its value. If
/// `true`, then return it. The parameter is assumed to be either `--arg=value` or `--arg value`.
fn arg_value<'a, T: Deref<Target = str>>(
    args: &'a [T],
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.iter().map(Deref::deref);
    while let Some(arg) = args.next() {
        let mut arg = arg.splitn(2, '=');
        if arg.next() != Some(find_arg) {
            continue;
        }

        match arg.next().or_else(|| args.next()) {
            Some(v) if pred(v) => return Some(v),
            _ => {},
        }
    }
    None
}

#[test]
fn test_arg_value() {
    let args = &["--bar=bar", "--foobar", "123", "--foo"];

    assert_eq!(arg_value(&[] as &[&str], "--foobar", |_| true), None);
    assert_eq!(arg_value(args, "--bar", |_| false), None);
    assert_eq!(arg_value(args, "--bar", |_| true), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "bar"), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "123"), Some("123"));
    assert_eq!(arg_value(args, "--foo", |_| true), None);
}

fn toolchain_path(home: Option<String>, toolchain: Option<String>) -> Option<PathBuf> {
    home.and_then(|home| {
        toolchain.map(|toolchain| {
            let mut path = PathBuf::from(home);
            path.push("toolchains");
            path.push(toolchain);
            path
        })
    })
}

fn display_help() {
    println!(
        "\
Checks a package to catch common mistakes and improve your Rust code.

Usage:
    cargo marker [options] [--] [<opts>...]

Common options:
    -h, --help               Print this message
        --rustc              Pass all args to rustc
    -V, --version            Print version info and exit
        --toolchain          Print the required nightly toolchain

Other options are the same as `cargo check`.

To allow or deny a lint from the command line you can use `cargo marker --`
with:

    -W --warn OPT       Set lint warnings
    -A --allow OPT      Set lint allowed
    -D --deny OPT       Set lint denied
    -F --forbid OPT     Set lint forbidden

This message belongs to a specific driver, if possible you should avoid
interfacing with the driver directly and use `cargo marker` instead.
"
    );
}

fn main() {
    rustc_driver::init_rustc_env_logger();

    // FIXME: Add ICE hook. Ideally this would distinguish where the error happens.
    // ICEs have to be reported like in Clippy. For lint impl ICEs we should have
    // an extra ICE hook that identifies the lint impl and ideally continues with
    // other registered lints
    exit(rustc_driver::catch_with_exit_code(|| {
        let mut orig_args: Vec<String> = env::args().collect();

        let sys_root_arg = arg_value(&orig_args, "--sysroot", |_| true);
        let has_sys_root_arg = sys_root_arg.is_some();

        // Further invocation of rustc require the `--sysroot` flag. We add it here
        // in preparation.
        if !has_sys_root_arg {
            let sys_root = find_sys_root(sys_root_arg);
            orig_args.extend(vec!["--sysroot".into(), sys_root]);
        };

        // make "marker-driver --rustc" work like a subcommand that passes further args to "rustc"
        // for example `marker-driver --rustc --version` will print the rustc version that marker-driver
        // uses
        if let Some(pos) = orig_args.iter().position(|arg| arg == "--rustc") {
            orig_args.remove(pos);
            orig_args[0] = "rustc".to_string();

            return rustc_driver::RunCompiler::new(&orig_args, &mut DefaultCallbacks).run();
        }

        if orig_args.iter().any(|a| a == "--version" || a == "-V") {
            let version_info = rustc_tools_util::get_version_info!();
            println!("{version_info}");
            exit(0);
        }

        // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
        // We're invoking the compiler programmatically, so we'll ignore this.
        let wrapper_mode = orig_args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());

        if wrapper_mode {
            // we still want to be able to invoke rustc normally
            orig_args.remove(1);
        }

        if !wrapper_mode {
            if orig_args.iter().any(|a| a == "--help" || a == "-h") {
                display_help();
                exit(0);
            }

            if orig_args.iter().any(|a| a == "--toolchain") {
                println!("{RUSTC_TOOLCHAIN_VERSION}");
                exit(0);
            }
        }

        // We enable Marker if one of the following conditions is met
        // - IF Marker is run on its test suite OR
        // - IF Marker is run on the main crate, not on deps (`!cap_lints_allow`) THEN
        //    - IF `--no-deps` is not set (`!no_deps`) OR
        //    - IF `--no-deps` is set and Marker is run on the specified primary package
        let cap_lints_allow = arg_value(&orig_args, "--cap-lints", |val| val == "allow").is_some()
            && arg_value(&orig_args, "--force-warn", |_| true).is_none();
        let no_deps = orig_args.iter().any(|arg| arg == "--no-deps");
        let in_primary_package = env::var("CARGO_PRIMARY_PACKAGE").is_ok();

        let enable_marker = !cap_lints_allow && (!no_deps || in_primary_package);
        if enable_marker {
            rustc_driver::RunCompiler::new(&orig_args, &mut MarkerCallback).run()
        } else {
            rustc_driver::RunCompiler::new(&orig_args, &mut DefaultCallbacks).run()
        }
    }))
}

/// Get the sysroot, looking from most specific to this invocation to the least:
/// - command line
/// - runtime environment
///    - `SYSROOT`
///    - `RUSTUP_HOME`, `MULTIRUST_HOME`, `RUSTUP_TOOLCHAIN`, `MULTIRUST_TOOLCHAIN`
/// - sysroot from rustc in the path
/// - compile-time environment
///    - `SYSROOT`
///    - `RUSTUP_HOME`, `MULTIRUST_HOME`, `RUSTUP_TOOLCHAIN`, `MULTIRUST_TOOLCHAIN`
fn find_sys_root(sys_root_arg: Option<&str>) -> String {
    sys_root_arg
        .map(PathBuf::from)
        .or_else(|| std::env::var("SYSROOT").ok().map(PathBuf::from))
        .or_else(|| {
            let home = std::env::var("RUSTUP_HOME")
                .or_else(|_| std::env::var("MULTIRUST_HOME"))
                .ok();
            let toolchain = std::env::var("RUSTUP_TOOLCHAIN")
                .or_else(|_| std::env::var("MULTIRUST_TOOLCHAIN"))
                .ok();
            toolchain_path(home, toolchain)
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| PathBuf::from(s.trim()))
        })
        .or_else(|| option_env!("SYSROOT").map(PathBuf::from))
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME")
                .or(option_env!("MULTIRUST_HOME"))
                .map(ToString::to_string);
            let toolchain = option_env!("RUSTUP_TOOLCHAIN")
                .or(option_env!("MULTIRUST_TOOLCHAIN"))
                .map(ToString::to_string);
            toolchain_path(home, toolchain)
        })
        .map(|pb| pb.to_string_lossy().to_string())
        .expect("need to specify SYSROOT env var during marker compilation, or use rustup or multirust")
}
