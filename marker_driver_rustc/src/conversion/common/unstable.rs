//! This module contains all conversion functions for unstable items, which are
//! still kind of part of the API like `Lint` structs

use marker_api::lint::{Lint, MacroReport};

use crate::context::RustcContext;

use super::to_rustc_lint_level;

pub fn to_rustc_lint<'ast, 'tcx>(cx: &RustcContext<'ast, 'tcx>, api_lint: &'static Lint) -> &'static rustc_lint::Lint {
    cx.storage.lint_or_insert(api_lint, || {
        // Not extracted to an extra function, as its very specific
        let report_in_external_macro = match api_lint.report_in_macro {
            MacroReport::No | MacroReport::Local => false,
            MacroReport::All => true,
            _ => unreachable!("added variant to lint::MacroReport"),
        };

        Box::leak(Box::new(rustc_lint::Lint {
            name: api_lint.name,
            default_level: to_rustc_lint_level(api_lint.default_level),
            desc: api_lint.explaination,
            edition_lint_opts: None,
            report_in_external_macro,
            future_incompatible: None,
            is_plugin: true,
            feature_gate: None,
            crate_level_only: false,
        }))
    })
}
