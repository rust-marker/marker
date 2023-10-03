#![doc = include_str!("../../README.md")]
#![feature(rustc_private)]
#![feature(lint_reasons)]
#![feature(non_exhaustive_omitted_patterns_lint)]
#![warn(clippy::pedantic)]
#![warn(non_exhaustive_omitted_patterns)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_lifetimes, reason = "lifetimes will be required to fix ICEs")]
#![allow(clippy::needless_collect, reason = "false positives for `alloc_slice`")]
#![allow(clippy::too_many_lines, reason = "long functions are unavoidable for matches")]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_hir_analysis;
extern crate rustc_interface;
extern crate rustc_lint;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

use std::env;

use rustc_session::config::ErrorOutputType;
use rustc_session::EarlyErrorHandler;

use marker_rustc_driver::{try_main, MainError};

const BUG_REPORT_URL: &str = "https://github.com/rust-marker/marker/issues/new?template=panic.yml";

fn main() {
    let handler = EarlyErrorHandler::new(ErrorOutputType::default());
    rustc_driver::init_rustc_env_logger(&handler);

    // FIXME(xFrednet): The ICE hook would ideally distinguish where the error
    // happens. Panics from lint crates should probably not terminate Marker
    // completely, but instead warn the user and continue linting with the other
    // lint crate. It would also be cool if the ICE hook printed the node that
    // caused the panic in the lint crate. rust-marker/marker#10

    rustc_driver::install_ice_hook(BUG_REPORT_URL, |handler| {
        handler.note_without_error(format!("{}", rustc_tools_util::get_version_info!()));
        handler.note_without_error("Achievement Unlocked: [Free Ice Cream]");
    });

    std::process::exit(rustc_driver::catch_with_exit_code(|| {
        try_main(env::args()).map_err(|err| {
            let err = match err {
                MainError::Custom(err) => err,
                MainError::Rustc(err) => return err,
            };

            // Emit the error to stderr
            err.print();

            // This is a bit of a hack, but this way we can emit our own errors
            // without having to change the rustc driver.
            #[expect(deprecated)]
            rustc_span::ErrorGuaranteed::unchecked_claim_error_was_emitted()
        })
    }))
}
