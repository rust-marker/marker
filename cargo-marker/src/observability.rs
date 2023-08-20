//! This is the module responsible for all things related to debugging `cargo-marker`
//! by us mere mortals. This includes logging, panic hooks, and potentially more
//! kinds of telemetry in the future.
//!
//! We don't exfiltrate any data from our users since linting is a pretty deterministic
//! process and it should be possible to reproduce bugs and report them voluntarily.
//! However, we should capture as much information as possible in our telemetry to make it
//! easier for the users to collect the diagnostics for us and attach them to an issue,
//! or even debug `cargo-marker` themselves if our telemetry is easy to read.
//!
//! It's possible that this module will be split up in the future if it gets too big.

pub(crate) mod display;

use prelude::*;
use std::ops::Deref;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use yansi::Paint;

// We don't care if some of these are unused. Ideally, they should just be
// reexported in `tracing::prelude`.
#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::display::CommandExt;
    pub(crate) use tracing::instrument;
    pub(crate) use tracing::{debug, error, info, trace, warn};
    pub(crate) use tracing::{debug_span, error_span, info_span, trace_span, warn_span};
}

/// The env variable that uses the syntax of [`tracing_subscriber::EnvFilter`]
/// to control the logging verbosity of this binary. See the docs for the
/// [`tracing_subscriber::EnvFilter`] for more information.
const MARKER_LOG: &str = "MARKER_LOG";

pub(crate) fn init() {
    init_logging();

    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().width(140).build())
    }))
    .expect("BUG: miette hook must be initialized only once");

    std::panic::set_hook(Box::new(panic_hook));
}

fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var(MARKER_LOG)
        .from_env_lossy();

    let fmt_stderr = tracing_subscriber::fmt::layer().compact();

    tracing_subscriber::registry()
        .with(fmt_stderr)
        .with(env_filter)
        .with(tracing_error::ErrorLayer::default())
        .init();
}

fn panic_hook(panic_info: &std::panic::PanicInfo<'_>) {
    let debug_ctx = marker_error::ErrorTrace::capture();

    let mut err = miette::Report::new(Panic { trace: debug_ctx });

    if let Some(location) = panic_info.location() {
        err = err.context(format!(
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        ));
    }

    let thread = std::thread::current();

    // We want to know the name of the thread only if the panic occurred in a
    // non-main thread. We don't use multithreading too much in this binary,
    // so the `Thread: main` in the panic message would be just noise.
    let thread = thread.name().filter(|&name| name != "main");

    if let Some(thread) = thread {
        err = err.context(format!("Thread: {thread}"));
    }

    // If the panic message was formatted using interpolated values,
    // it will be a `String`. Otherwise, it will be a `&str`.
    let payload = panic_info.payload();
    let message = payload
        .downcast_ref::<String>()
        .map(<_>::deref)
        .or_else(|| payload.downcast_ref::<&str>().map(<_>::deref))
        .unwrap_or("<unknown-message>")
        .to_owned();

    err = err.context(
        format_args!("💥 INTERNAL MARKER PANIC: {message}")
            .bold()
            .wrap()
            .red()
            .wrap()
            .to_string(),
    );

    eprint!("{err:?}");
}

#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[diagnostic(
    help(
        "we would be grateful if you report this to us at {}",
        "https://github.com/rust-marker/marker/issues".underline().cyan()
    ),
    url("https://github.com/rust-marker/marker/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc+panic"),
    code(internal_marker_panic),
)]
#[error("{trace}")]
struct Panic {
    trace: marker_error::ErrorTrace,
}
