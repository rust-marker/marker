use linter_api::ast::{
    item::{Item, ItemKind, StaticItem},
    Symbol, ty::Mutability,
};

use std::fmt::Debug;

use super::{rustc::RustcContext, ToApi};

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
        match self.inner.kind {
            rustc_hir::ItemKind::ExternCrate(sym) => ItemKind::ExternCrate(sym.map(|s| Symbol::new(s.as_u32()))),
            rustc_hir::ItemKind::Static(ty, _mutability, _body_id) => ItemKind::StaticItem(self.cx.alloc_with(|| RustcStaticItem::new(self.cx, ty))),

            // FIXME
            _ => ItemKind::Union,
            // rustc_hir::ItemKind::Use(&'hir Path<'hir>, UseKind),
            // rustc_hir::ItemKind::Const(&'hir Ty<'hir>, BodyId),
            // rustc_hir::ItemKind::Fn(FnSig<'hir>, Generics<'hir>, BodyId),
            // rustc_hir::ItemKind::Macro(MacroDef, MacroKind),
            // rustc_hir::ItemKind::Mod(Mod<'hir>),
            // rustc_hir::ItemKind::ForeignMod { .. },
            // rustc_hir::ItemKind::GlobalAsm(&'hir InlineAsm<'hir>),
            // rustc_hir::ItemKind::TyAlias(&'hir Ty<'hir>, Generics<'hir>),
            // rustc_hir::ItemKind::OpaqueTy(OpaqueTy<'hir>),
            // rustc_hir::ItemKind::Enum(EnumDef<'hir>, Generics<'hir>),
            // rustc_hir::ItemKind::Struct(VariantData<'hir>, Generics<'hir>),
            // rustc_hir::ItemKind::Union(VariantData<'hir>, Generics<'hir>),
            // rustc_hir::ItemKind::Trait(IsAuto, Unsafety, Generics<'hir>, GenericBounds<'hir>, &'hir [TraitItemRef]),
            // rustc_hir::ItemKind::TraitAlias(Generics<'hir>, GenericBounds<'hir>),
            // rustc_hir::ItemKind::Impl(Impl<'hir>),
        }
    }

    fn get_attrs(&'ast self) -> &'ast [&dyn linter_api::ast::Attribute<'ast>] {
        todo!()
    }
}

#[derive(Debug)]
struct RustcStaticItem<'ast, 'tcx> {
    pub(crate) cx: &'ast RustcContext<'ast, 'tcx>,
    pub(crate) ty: &'tcx rustc_hir::Ty<'tcx>,
}

impl<'ast, 'tcx> RustcStaticItem<'ast, 'tcx> {
    fn new(cx: &'ast RustcContext<'ast, 'tcx>, ty: &'tcx rustc_hir::Ty<'tcx>) -> Self { Self { cx, ty } }
}

impl<'ast, 'tcx> StaticItem<'ast> for RustcStaticItem<'ast, 'tcx> {
    fn get_type(&'ast self) -> &'ast dyn linter_api::ast::ty::Ty<'ast> {
        self.ty.to_api(self.cx)
    }

    fn get_mutability(&self) -> Mutability {
        todo!()
    }

    fn get_body_id(&self) -> linter_api::ast::BodyId {
        todo!()
    }
}
