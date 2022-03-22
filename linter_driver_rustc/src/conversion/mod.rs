#![expect(unused)]

use crate::ast::{item::RustcItem, rustc::RustcContext};
use linter_adapter::loader::ExternalLintPassRegistry;
use linter_api::LintPass;

use rustc_hir::Item;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::impl_lint_pass;

use bumpalo::Bump;

pub struct ConverterLintPass {}

impl ConverterLintPass {
    pub fn new() -> ConverterLintPass {
        Self {}
    }
}

impl_lint_pass!(ConverterLintPass => []);

impl<'tcx> LateLintPass<'tcx> for ConverterLintPass {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        let map = cx.tcx.hir();
        let bump = Bump::new();

        let context = bump.alloc_with(|| RustcContext::new(cx.tcx, cx.lint_store, &bump));

        for rustc_item in map.items() {
            let item = bump.alloc_with(|| RustcItem::new(rustc_item, context));

            process_item(item);
        }
    }
}

fn process_item<'tcx, 'ast>(item: &'ast RustcItem<'ast, 'tcx>) {
    let mut registry = ExternalLintPassRegistry::default();
    registry.load_external_lib("./target/debug/liblinter_test.so").unwrap();

    let pass = &mut registry.lint_passes[0];
    pass.check_item(item);
}
