use linter_api::ast::{
    item::{
        GenericDefs, GenericParam, GenericParamId, Item, ItemKind, StaticItem, AdtField, StructItem, AdtVariantData,
    },
    ty::Mutability,
    Span, Symbol,
};

use super::{rustc::RustcContext, ToApi};

#[derive(Debug)]
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

impl<'ast, 'tcx> Item<'ast> for RustcItem<'ast, 'tcx> {
    fn get_id(&self) -> linter_api::ast::item::ItemId {
        let (i1, i2) = self.inner.hir_id().index();
        linter_api::ast::item::ItemId::new(i1, i2)
    }

    fn get_span(&'ast self) -> &'ast dyn Span<'ast> {
        self.cx.new_span(self.inner.span)
    }

    fn get_vis(&self) -> &'ast linter_api::ast::item::Visibility<'ast> {
        self.cx.new_visibility(self.inner.vis)
    }

    fn get_ident(&'ast self) -> Option<&'ast linter_api::ast::Ident<'ast>> {
        (!self.inner.ident.name.is_empty()).then(|| self.cx.new_ident(self.inner.ident))
    }

    fn get_kind(&'ast self) -> linter_api::ast::item::ItemKind<'ast> {
        match &self.inner.kind {
            rustc_hir::ItemKind::ExternCrate(sym) => ItemKind::ExternCrate(sym.map(|s| Symbol::new(s.as_u32()))),
            rustc_hir::ItemKind::Static(ty, _mutability, _body_id) => {
                ItemKind::StaticItem(self.cx.alloc_with(|| RustcStaticItem::new(self.cx, ty)))
            },
            rustc_hir::ItemKind::Struct(var_data, generics) => ItemKind::Struct(
                self.cx
                    .alloc_with(|| RustcStructItem::from_rustc(self.cx, var_data, generics)),
            ),

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
    fn new(cx: &'ast RustcContext<'ast, 'tcx>, ty: &'tcx rustc_hir::Ty<'tcx>) -> Self {
        Self { cx, ty }
    }
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

#[derive(Debug)]
#[expect(unused)]
struct RustcStructItem<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_var_data: &'tcx rustc_hir::VariantData<'tcx>,
    rustc_generics: &'tcx rustc_hir::Generics<'tcx>,
}

impl<'ast, 'tcx> RustcStructItem<'ast, 'tcx> {
    fn from_rustc(
        cx: &'ast RustcContext<'ast, 'tcx>,
        rustc_var_data: &'tcx rustc_hir::VariantData<'tcx>,
        rustc_generics: &'tcx rustc_hir::Generics<'tcx>,
    ) -> Self {
        Self {
            cx,
            rustc_var_data,
            rustc_generics,
        }
    }
}

impl<'ast, 'tcx> StructItem<'ast> for RustcStructItem<'ast, 'tcx> {
    fn get_ty_id(&self) -> linter_api::ast::ty::TyId {
        todo!()
    }

    fn get_kind(&'ast self) -> linter_api::ast::item::AdtVariantData<'ast> {
        match &self.rustc_var_data {
            rustc_hir::VariantData::Struct(fields, _recovered) => {
                AdtVariantData::Field(RustcStructField::from_rustc_slice(self.cx, fields))
            },
            rustc_hir::VariantData::Tuple(fields, _constructor_id) => {
                AdtVariantData::Tuple(RustcStructField::from_rustc_slice(self.cx, fields))
            },
            rustc_hir::VariantData::Unit(_constructor_id) => AdtVariantData::Unit,
        }
    }

    fn get_generics(&'ast self) -> &'ast dyn linter_api::ast::item::GenericDefs<'ast> {
        todo!()
    }
}

#[derive(Debug)]
struct RustcStructField<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_field_def: &'tcx rustc_hir::FieldDef<'tcx>,
}

impl<'ast, 'tcx> RustcStructField<'ast, 'tcx> {
    fn from_rustc(cx: &'ast RustcContext<'ast, 'tcx>, rustc_field_def: &'tcx rustc_hir::FieldDef<'tcx>) -> Self {
        Self { cx, rustc_field_def }
    }

    fn from_rustc_slice(
        cx: &'ast RustcContext<'ast, 'tcx>,
        field_defs: &'tcx [rustc_hir::FieldDef<'tcx>],
    ) -> &'ast [&'ast dyn AdtField<'ast>] {
        cx.alloc_slice_from_iter(
            field_defs
                .iter()
                .map(|def| cx.alloc_with(|| Self::from_rustc(cx, def)) as &'ast dyn AdtField<'ast>),
        )
    }
}

impl<'ast, 'tcx> AdtField<'ast> for RustcStructField<'ast, 'tcx> {
    fn get_attributes(&'ast self) -> &'ast dyn linter_api::ast::Attribute {
        todo!()
    }

    fn get_span(&'ast self) -> &'ast dyn Span<'ast> {
        todo!()
    }

    fn get_visibility(&'ast self) -> linter_api::ast::item::VisibilityKind<'ast> {
        todo!()
    }

    fn get_name(&'ast self) -> Symbol {
        todo!()
    }

    fn get_ty(&'ast self) -> &'ast dyn linter_api::ast::ty::Ty<'ast> {
        self.rustc_field_def.ty.to_api(self.cx)
    }
}

#[derive(Debug)]
#[expect(unused)]
struct RustcGenericDefs<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
    inner: &'tcx rustc_hir::Generics<'tcx>,
}

impl<'ast, 'tcx> GenericDefs<'ast> for RustcGenericDefs<'ast, 'tcx> {
    fn get_generics(&self) -> &'ast [&'ast dyn GenericParam<'ast>] {
        todo!()
    }

    fn get_bounds(&self) {
        todo!()
    }
}

#[derive(Debug)]
struct RustcGenericParam<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
    inner: &'tcx rustc_hir::GenericParam<'tcx>,
}

impl<'ast, 'tcx> GenericParam<'ast> for RustcGenericParam<'ast, 'tcx> {
    fn get_id(&self) -> linter_api::ast::item::GenericParamId {
        todo!()
    }

    fn get_span(&self) -> &'ast dyn Span<'ast> {
        self.inner.span.to_api(self.cx)
    }

    fn get_name(&self) -> Option<Symbol> {
        self.inner.name.to_api(self.cx)
    }

    fn get_kind(&self) -> linter_api::ast::item::GenericKind<'ast> {
        todo!()
    }
}

impl ToApi<'_, '_, GenericParamId> for rustc_hir::HirId {
    fn to_api(&self, _cx: &RustcContext<'_, '_>) -> GenericParamId {
        let (_krate, _index) = self.index();
        todo!() // FML GenericParamId::new(krate, index)
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Option<Symbol>> for rustc_hir::ParamName {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> Option<Symbol> {
        match self {
            rustc_hir::ParamName::Plain(ident) => Some(ident.name.to_api(cx)),
            rustc_hir::ParamName::Fresh(_) => None,
            rustc_hir::ParamName::Error => unreachable!(),
        }
    }
}
