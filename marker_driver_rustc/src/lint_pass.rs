use marker_adapter::Adapter;

use crate::context::{storage::Storage, RustcContext};

pub struct MarkerLintPass;

rustc_lint_defs::impl_lint_pass!(MarkerLintPass => []);

impl<'tcx> rustc_lint::LateLintPass<'tcx> for MarkerLintPass {
    fn check_crate(&mut self, rustc_cx: &rustc_lint::LateContext<'tcx>) {
        process_crate(rustc_cx);
    }
}

fn process_crate(rustc_cx: &rustc_lint::LateContext<'_>) {
    let storage = Storage::default();
    process_crate_lifetime(rustc_cx, &storage);
}

/// This function marks the start of the `'ast` lifetime. The lifetime is defined
/// by the [`Storage`] object.
fn process_crate_lifetime<'ast, 'tcx: 'ast>(rustc_cx: &rustc_lint::LateContext<'tcx>, storage: &'ast Storage<'ast>) {
    let driver_cx = RustcContext::new(rustc_cx.tcx, rustc_cx.lint_store, storage);

    // To support debug printing of AST nodes, as these might sometimes require the
    // contest. Note that this only sets the cx for the rustc side. Each lint crate
    // has their own storage for cx.
    marker_api::context::set_ast_cx(driver_cx.ast_cx());

    let krate = driver_cx
        .marker_converter
        .to_crate(rustc_hir::def_id::LOCAL_CRATE, driver_cx.rustc_cx.hir().root_module());

    let mut adapter = Adapter::new_from_env();
    adapter.process_krate(driver_cx.ast_cx(), krate);
}
