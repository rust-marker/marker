use marker_api::lint::{Lint, MacroReport};

use super::RustcConversionContext;

impl<'ast, 'tcx> RustcConversionContext<'ast, 'tcx> {
    pub fn to_lint(&self, api_lint: &'static Lint) -> &'static rustc_lint::Lint {
        self.lints.borrow_mut().entry(api_lint).or_insert_with(|| {
            // Not extracted to an extra function, as it's very specific
            let report_in_external_macro = match api_lint.report_in_macro {
                MacroReport::No | MacroReport::Local => false,
                MacroReport::All => true,
                _ => unreachable!("added variant to lint::MacroReport"),
            };

            Box::leak(Box::new(rustc_lint::Lint {
                name: api_lint.name,
                default_level: self.to_lint_level(api_lint.default_level),
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
}
