use linter_api::ast::item::Item;

use std::fmt::Debug;

use super::rustc::RustcContext;

pub struct RustcItem<'ast, 'tcx> {
    pub(crate) cx: &'ast RustcContext<'ast, 'tcx>,
    pub(crate) inner: &'tcx rustc_hir::Item<'tcx>,
}

impl<'ast, 'tcx> RustcItem<'ast, 'tcx> {
    #[must_use]
    pub fn new(inner: &'tcx rustc_hir::Item<'tcx>, cx: &'ast RustcContext<'ast, 'tcx>) -> Self {
        Self { cx, inner }
    }
}

impl Debug for RustcItem<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustcItem").field("inner", &self.inner).finish()
    }
}

impl<'ast, 'tcx> Item<'ast> for RustcItem<'ast, 'tcx> {
    fn get_id(&self) -> linter_api::ast::item::ItemId {
        let (i1, i2) = self.inner.hir_id().index();
        linter_api::ast::item::ItemId::new(i1, i2)
    }

    fn get_span(&'ast self) -> &'ast dyn linter_api::ast::Span<'ast> {
        self.cx.new_span(self.inner.span)
    }

    fn get_vis(&self) -> &'ast linter_api::ast::item::Visibility<'ast> {
        self.cx.new_visibility(self.inner.vis)
    }

    fn get_ident(&'ast self) -> Option<&'ast linter_api::ast::Ident<'ast>> {
        (!self.inner.ident.name.is_empty()).then(|| self.cx.new_ident(self.inner.ident))
    }

    fn get_kind(&'ast self) -> linter_api::ast::item::ItemKind<'ast> {
        todo!()
    }

    fn get_attrs(&'ast self) -> &'ast [&dyn linter_api::ast::Attribute<'ast>] {
        todo!()
    }
}
