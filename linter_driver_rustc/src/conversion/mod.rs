#![expect(unused)]

use crate::ast::{itemos::RustcItem, rustc::RustcContext};
use linter_adapter::loader::ExternalLintPassRegistry;
use linter_api::{ast::item::ItemType, context::Context, LintPass};

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
        let api_cx = context.alloc_with(|| Context::new(context));

        for rustc_item in map.items() {
            if let Some(item) = crate::ast::item::from_rustc(context, rustc_item) {
                process_item(api_cx, item);
            }
        }
    }
}

fn process_item<'ast>(cx: &'ast Context<'ast>, item: ItemType<'ast>) {
    let mut registry = ExternalLintPassRegistry::default();
    registry.load_external_lib("./target/debug/liblinter_test.so").unwrap();

    let pass = &mut registry.lint_passes[0];
    pass.check_item(cx, item);
}
