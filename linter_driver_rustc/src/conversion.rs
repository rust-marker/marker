use crate::ast::rustc::RustcContext;
use crate::ast::ToApi;
use linter_adapter::Adapter;
use linter_api::{
    ast::{item::ItemType, Crate},
    context::AstContext,
};

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
    fn check_crate(&mut self, rustc_cx: &LateContext<'tcx>) {
        let mut bump = Bump::new();
        process_items(rustc_cx, &mut bump);
    }
}

/// This function converts the current crate into api types and passes them to
/// the adapter for further distribution. This function is used to ensure that
/// the allocator outlives every item created in this function. This is basically
/// the start if the `'ast` lifetime
fn process_items<'tcx>(rustc_cx: &LateContext<'tcx>, allocator: &mut Bump) {
    // Setup adapter from environment
    let mut adapter = Adapter::new_from_env();
    adapter.registered_lints();

    // Setup context
    let driver_cx = allocator.alloc_with(|| RustcContext::new(rustc_cx, allocator));
    let ast_cx = driver_cx.alloc_with(|| AstContext::new(driver_cx));

    let map = rustc_cx.tcx.hir();
    // Here we need to collect the items to have a knwon size for the allocation
    #[allow(
        clippy::needless_collect,
        reason = "collect is required to know the size of the allocation"
    )]
    let items: Vec<ItemType> = map
        .root_module()
        .item_ids
        .iter()
        .map(|id| map.item(*id))
        .filter_map(|rustc_item| crate::ast::item::from_rustc(driver_cx, rustc_item))
        .collect();
    let krate = Crate::new(
        rustc_hir::def_id::LOCAL_CRATE.to_api(driver_cx),
        driver_cx.alloc_slice_from_iter(items.into_iter()),
    );

    adapter.process_krate(ast_cx, &krate);
}
