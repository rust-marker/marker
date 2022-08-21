use crate::context::{storage::Storage, RustcContext};

pub struct LinterLintPass;

rustc_lint_defs::impl_lint_pass!(LinterLintPass => []);

impl<'tcx> rustc_lint::LateLintPass<'tcx> for LinterLintPass {
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
    let _driver_cx = RustcContext::new(rustc_cx.tcx, rustc_cx.lint_store, storage);
    // let mut adapter = Adapter::new_from_env();
}
