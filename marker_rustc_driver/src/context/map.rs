use marker_adapter::context::AstMapDriver;
use marker_api::{
    ast::{EnumVariant, ItemField},
    common::Level,
    prelude::*,
};

use super::RustcContext;

impl<'ast, 'tcx: 'ast> AstMapDriver<'ast> for RustcContext<'ast, 'tcx> {
    fn item(&'ast self, id: ItemId) -> Option<ItemKind<'ast>> {
        let rustc_id = self.rustc_converter.to_item_id(id);
        self.marker_converter.item(rustc_id)
    }

    fn variant(&'ast self, id: VariantId) -> Option<&'ast EnumVariant<'ast>> {
        self.marker_converter.variant(id)
    }

    fn field(&'ast self, id: FieldId) -> Option<&'ast ItemField<'ast>> {
        self.marker_converter.field(id)
    }

    fn body(&'ast self, id: BodyId) -> &'ast ast::Body<'ast> {
        let rustc_id = self.rustc_converter.to_body_id(id);
        self.marker_converter.body(rustc_id)
    }

    fn stmt(&'ast self, id: StmtId) -> StmtKind<'ast> {
        let rustc_id = self.rustc_converter.to_hir_id(id);
        match self.marker_converter.stmt(rustc_id) {
            Some(stmt) => stmt,
            None => unreachable!(
                "the `HirId` belongs to a valid statement, since it comes from a `StmtId`. (HirId: {rustc_id:?})"
            ),
        }
    }

    fn expr(&'ast self, id: ExprId) -> ExprKind<'ast> {
        let rustc_id = self.rustc_converter.to_hir_id(id);
        match self.marker_converter.expr(rustc_id) {
            Some(expr) => expr,
            None => unreachable!(
                "the `HirId` belongs to a valid expression, since it comes from a `ExprId`. (HirId: {rustc_id:?})"
            ),
        }
    }

    fn lint_level_at(&'ast self, api_lint: &'static Lint, node: NodeId) -> Level {
        if let Some(id) = self.rustc_converter.try_to_hir_id_from_emission_node(node) {
            let lint = self.rustc_converter.to_lint(api_lint);
            let level = self.rustc_cx.lint_level_at_node(lint, id).0;
            self.marker_converter.to_lint_level(level)
        } else {
            Level::Allow
        }
    }
}
