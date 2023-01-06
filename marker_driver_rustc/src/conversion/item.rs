use marker_api::ast::{
    generic::{GenericParamKind, GenericParams, LifetimeClause, LifetimeParam, TyClause, TyParam, WhereClauseKind},
    item::{
        AdtKind, AssocItemKind, CommonItemData, ConstItem, EnumItem, EnumVariant, ExternBlockItem, ExternCrateItem,
        ExternItemKind, Field, FnItem, ImplItem, ItemKind, ModItem, StaticItem, StructItem, TraitItem, TyAliasItem,
        UnionItem, UnstableItem, UseItem, UseKind, Visibility,
    },
    ty::TyKind,
    Abi, CommonCallableData, ItemId, Parameter,
};
use rustc_hir as hir;

use crate::context::RustcContext;

use super::{
    generic::{conv_ast_bound, to_api_lifetime, to_api_trait_ref},
    to_api_abi, to_api_body_id, to_api_path, to_generic_id, to_item_id, to_rustc_item_id, to_span_id, to_symbol_id,
    ty::TyConverter,
};

/// This converter combines a bunch of functions used to convert rustc items into
/// api items. This is mainly used to group functions together and to not always
/// pass the context around as an argument.
pub struct ItemConverter<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
}

impl<'ast, 'tcx> ItemConverter<'ast, 'tcx> {
    pub fn new(cx: &'ast RustcContext<'ast, 'tcx>) -> Self {
        Self { cx }
    }

    #[must_use]
    pub fn conv_item_from_id(&self, id: ItemId) -> Option<ItemKind<'ast>> {
        self.conv_items(&[to_rustc_item_id(id)]).first().copied()
    }

    #[must_use]
    pub fn conv_item(&self, rustc_item: &'tcx hir::Item<'tcx>) -> Option<ItemKind<'ast>> {
        let id = to_item_id(rustc_item.owner_id);
        if let Some(item) = self.cx.storage.items.borrow().get(&id) {
            return Some(*item);
        }

        let name = to_symbol_id(rustc_item.ident.name);
        let data = CommonItemData::new(id, name);
        let item = match &rustc_item.kind {
            hir::ItemKind::ExternCrate(original_name) => ItemKind::ExternCrate(
                self.alloc(|| ExternCrateItem::new(data, original_name.map_or(name, to_symbol_id))),
            ),
            hir::ItemKind::Use(path, use_kind) => {
                let use_kind = match use_kind {
                    hir::UseKind::Single => UseKind::Single,
                    hir::UseKind::Glob => UseKind::Glob,
                    hir::UseKind::ListStem => return None,
                };
                ItemKind::Use(self.alloc(|| UseItem::new(data, to_api_path(self.cx, path), use_kind)))
            },
            hir::ItemKind::Static(rustc_ty, rustc_mut, rustc_body_id) => ItemKind::Static(self.alloc(|| {
                StaticItem::new(
                    data,
                    matches!(*rustc_mut, rustc_ast::Mutability::Mut),
                    Some(to_api_body_id(*rustc_body_id)),
                    self.conv_ty(rustc_ty),
                )
            })),
            hir::ItemKind::Const(rustc_ty, rustc_body_id) => ItemKind::Const(
                self.alloc(|| ConstItem::new(data, self.conv_ty(rustc_ty), Some(to_api_body_id(*rustc_body_id)))),
            ),
            hir::ItemKind::Fn(fn_sig, generics, body_id) => ItemKind::Fn(self.alloc(|| {
                FnItem::new(
                    data,
                    self.conv_generic(generics),
                    self.conv_fn_sig(fn_sig, false),
                    Some(to_api_body_id(*body_id)),
                )
            })),
            hir::ItemKind::Mod(rustc_mod) => {
                ItemKind::Mod(self.alloc(|| ModItem::new(data, self.conv_items(rustc_mod.item_ids))))
            },
            hir::ItemKind::ForeignMod { abi, items } => ItemKind::ExternBlock(self.alloc(|| {
                let abi = to_api_abi(*abi);
                ExternBlockItem::new(data, abi, self.conv_foreign_items(items, abi))
            })),
            hir::ItemKind::Macro(_, _) | hir::ItemKind::GlobalAsm(_) => return None,
            hir::ItemKind::TyAlias(rustc_ty, rustc_generics) => ItemKind::TyAlias(self.alloc(|| {
                TyAliasItem::new(
                    data,
                    self.conv_generic(rustc_generics),
                    &[],
                    Some(self.conv_ty(rustc_ty)),
                )
            })),
            hir::ItemKind::OpaqueTy(_) => ItemKind::Unstable(
                self.alloc(|| UnstableItem::new(data, Some(to_symbol_id(rustc_span::sym::type_alias_impl_trait)))),
            ),
            hir::ItemKind::Enum(enum_def, generics) => ItemKind::Enum(
                self.alloc(|| EnumItem::new(data, self.conv_generic(generics), self.conv_enum_def(enum_def))),
            ),
            hir::ItemKind::Struct(var_data, generics) => ItemKind::Struct(
                self.alloc(|| StructItem::new(data, self.conv_generic(generics), self.conv_variant_data(var_data))),
            ),
            hir::ItemKind::Union(var_data, generics) => ItemKind::Union(self.alloc(|| {
                UnionItem::new(
                    data,
                    self.conv_generic(generics),
                    self.conv_variant_data(var_data).fields(),
                )
            })),
            hir::ItemKind::Trait(_is_auto, unsafety, generics, bounds, items) => ItemKind::Trait(self.alloc(|| {
                TraitItem::new(
                    data,
                    matches!(unsafety, hir::Unsafety::Unsafe),
                    self.conv_generic(generics),
                    conv_ast_bound(self.cx, bounds),
                    self.conv_trait_items(items),
                )
            })),
            hir::ItemKind::TraitAlias(_, _) => ItemKind::Unstable(
                self.alloc(|| UnstableItem::new(data, Some(to_symbol_id(rustc_span::sym::trait_alias)))),
            ),
            hir::ItemKind::Impl(imp) => ItemKind::Impl(self.alloc(|| {
                ImplItem::new(
                    data,
                    matches!(imp.unsafety, hir::Unsafety::Unsafe),
                    matches!(imp.polarity, rustc_ast::ImplPolarity::Positive),
                    imp.of_trait
                        .as_ref()
                        .map(|trait_ref| to_api_trait_ref(self.cx, trait_ref)),
                    self.conv_generic(imp.generics),
                    self.conv_ty(imp.self_ty),
                    self.conv_assoc_items(imp.items),
                )
            })),
        };

        self.cx.storage.items.borrow_mut().insert(id, item);
        Some(item)
    }

    #[must_use]
    pub fn conv_items(&self, item: &[hir::ItemId]) -> &'ast [ItemKind<'ast>] {
        let items: Vec<ItemKind<'_>> = item
            .iter()
            .map(|rid| self.cx.rustc_cx.hir().item(*rid))
            .filter_map(|rustc_item| self.conv_item(rustc_item))
            .collect();
        self.cx.storage.alloc_slice_iter(items.into_iter())
    }

    fn conv_assoc_item(&self, rustc_item: &hir::ImplItemRef) -> AssocItemKind<'ast> {
        let id = to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.cx.storage.items.borrow().get(&id) {
            return item.try_into().unwrap();
        }

        let impl_item = self.cx.rustc_cx.hir().impl_item(rustc_item.id);
        let name = to_symbol_id(rustc_item.ident.name);
        let data = CommonItemData::new(id, name);

        let item =
            match &impl_item.kind {
                hir::ImplItemKind::Const(ty, body_id) => AssocItemKind::Const(
                    self.alloc(|| ConstItem::new(data, self.conv_ty(ty), Some(to_api_body_id(*body_id)))),
                ),
                hir::ImplItemKind::Fn(fn_sig, body_id) => AssocItemKind::Fn(self.alloc(|| {
                    FnItem::new(
                        data,
                        self.conv_generic(impl_item.generics),
                        self.conv_fn_sig(fn_sig, false),
                        Some(to_api_body_id(*body_id)),
                    )
                })),
                hir::ImplItemKind::Type(ty) => AssocItemKind::TyAlias(self.alloc(|| {
                    TyAliasItem::new(data, self.conv_generic(impl_item.generics), &[], Some(self.conv_ty(ty)))
                })),
            };

        self.cx.storage.items.borrow_mut().insert(id, item.as_item());
        item
    }

    fn conv_assoc_items(&self, items: &[hir::ImplItemRef]) -> &'ast [AssocItemKind<'ast>] {
        self.cx
            .storage
            .alloc_slice_iter(items.iter().map(|item| self.conv_assoc_item(item)))
    }

    fn conv_trait_item(&self, rustc_item: &hir::TraitItemRef) -> AssocItemKind<'ast> {
        let id = to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.cx.storage.items.borrow().get(&id) {
            return item.try_into().unwrap();
        }

        let trait_item = self.cx.rustc_cx.hir().trait_item(rustc_item.id);
        let name = to_symbol_id(rustc_item.ident.name);
        let data = CommonItemData::new(id, name);

        let item = match &trait_item.kind {
            hir::TraitItemKind::Const(ty, body_id) => {
                AssocItemKind::Const(self.alloc(|| ConstItem::new(data, self.conv_ty(ty), body_id.map(to_api_body_id))))
            },
            hir::TraitItemKind::Fn(fn_sig, trait_fn) => AssocItemKind::Fn(self.alloc(|| {
                let body = match trait_fn {
                    hir::TraitFn::Provided(body_id) => Some(to_api_body_id(*body_id)),
                    hir::TraitFn::Required(_) => None,
                };
                FnItem::new(
                    data,
                    self.conv_generic(trait_item.generics),
                    self.conv_fn_sig(fn_sig, false),
                    body,
                )
            })),
            hir::TraitItemKind::Type(bounds, ty) => AssocItemKind::TyAlias(self.alloc(|| {
                TyAliasItem::new(
                    data,
                    self.conv_generic(trait_item.generics),
                    conv_ast_bound(self.cx, bounds),
                    ty.map(|ty| self.conv_ty(ty)),
                )
            })),
        };

        self.cx.storage.items.borrow_mut().insert(id, item.as_item());
        item
    }

    fn conv_trait_items(&self, items: &[hir::TraitItemRef]) -> &'ast [AssocItemKind<'ast>] {
        self.cx
            .storage
            .alloc_slice_iter(items.iter().map(|item| self.conv_trait_item(item)))
    }

    fn conv_foreign_item(&self, rustc_item: &'tcx hir::ForeignItemRef, abi: Abi) -> ExternItemKind<'ast> {
        let id = to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.cx.storage.items.borrow().get(&id) {
            return (*item).try_into().unwrap();
        }

        let foreign_item = self.cx.rustc_cx.hir().foreign_item(rustc_item.id);
        let name = to_symbol_id(rustc_item.ident.name);
        let data = CommonItemData::new(id, name);
        let item = match &foreign_item.kind {
            hir::ForeignItemKind::Fn(fn_sig, idents, generics) => ExternItemKind::Fn(self.alloc(|| {
                FnItem::new(
                    data,
                    self.conv_generic(generics),
                    self.conv_fn_decl(fn_sig, idents, true, abi),
                    None,
                )
            })),
            hir::ForeignItemKind::Static(ty, rustc_mut) => ExternItemKind::Static(self.alloc(|| {
                StaticItem::new(
                    data,
                    matches!(*rustc_mut, rustc_ast::Mutability::Mut),
                    None,
                    self.conv_ty(ty),
                )
            })),
            hir::ForeignItemKind::Type => todo!(),
        };

        self.cx.storage.items.borrow_mut().insert(id, item.as_item());
        item
    }

    fn conv_foreign_items(&self, items: &'tcx [hir::ForeignItemRef], abi: Abi) -> &'ast [ExternItemKind<'ast>] {
        self.cx
            .storage
            .alloc_slice_iter(items.iter().map(|item| self.conv_foreign_item(item, abi)))
    }

    fn conv_ty(&self, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
        TyConverter::new(self.cx).conv_ty(rustc_ty)
    }

    fn conv_generic_params(&self, params: &[hir::GenericParam<'ast>]) -> &'ast [GenericParamKind<'ast>] {
        if params.is_empty() {
            return &[];
        }

        let params: Vec<_> = params
            .iter()
            .filter_map(|rustc_param| {
                let name = match rustc_param.name {
                    hir::ParamName::Plain(ident) => to_symbol_id(ident.name),
                    _ => return None,
                };
                let def_id = self.cx.rustc_cx.hir().local_def_id(rustc_param.hir_id);
                let id = to_generic_id(def_id.to_def_id());
                let span = to_span_id(rustc_param.span);
                match rustc_param.kind {
                    hir::GenericParamKind::Lifetime {
                        kind: hir::LifetimeParamKind::Explicit,
                    } => Some(GenericParamKind::Lifetime(
                        self.alloc(|| LifetimeParam::new(id, name, Some(span))),
                    )),
                    hir::GenericParamKind::Type { synthetic: false, .. } => {
                        Some(GenericParamKind::Ty(self.alloc(|| TyParam::new(Some(span), name, id))))
                    },
                    _ => None,
                }
            })
            .collect();

        self.cx.storage.alloc_slice_iter(params.into_iter())
    }

    fn conv_generic(&self, rustc_generics: &hir::Generics<'tcx>) -> GenericParams<'ast> {
        // FIXME: Update implementation to store the parameter bounds in the parameters
        let clauses: Vec<_> = rustc_generics
            .predicates
            .iter()
            .filter_map(|predicate| {
                match predicate {
                    hir::WherePredicate::BoundPredicate(ty_bound) => {
                        // FIXME Add span to API clause:
                        // let span = to_api_span_id(ty_bound.span);
                        let params = GenericParams::new(self.conv_generic_params(ty_bound.bound_generic_params), &[]);
                        let ty = self.conv_ty(ty_bound.bounded_ty);
                        Some(WhereClauseKind::Ty(self.alloc(|| {
                            TyClause::new(Some(params), ty, conv_ast_bound(self.cx, predicate.bounds()))
                        })))
                    },
                    hir::WherePredicate::RegionPredicate(lifetime_bound) => {
                        to_api_lifetime(self.cx, lifetime_bound.lifetime).map(|lifetime| {
                            WhereClauseKind::Lifetime(self.alloc(|| {
                                let bounds: Vec<_> = lifetime_bound
                                    .bounds
                                    .iter()
                                    .filter_map(|bound| match bound {
                                        hir::GenericBound::Outlives(lifetime) => to_api_lifetime(self.cx, lifetime),
                                        _ => unreachable!("lifetimes can only be bound by lifetimes"),
                                    })
                                    .collect();
                                let bounds = if bounds.is_empty() {
                                    self.cx.storage.alloc_slice_iter(bounds.into_iter())
                                } else {
                                    &[]
                                };
                                LifetimeClause::new(lifetime, bounds)
                            }))
                        })
                    },
                    hir::WherePredicate::EqPredicate(_) => {
                        unreachable!("the documentation states, that this is unsupported")
                    },
                }
            })
            .collect();
        let clauses = self.cx.storage.alloc_slice_iter(clauses.into_iter());

        GenericParams::new(self.conv_generic_params(rustc_generics.params), clauses)
    }

    fn conv_fn_sig(&self, fn_sig: &hir::FnSig<'tcx>, is_extern: bool) -> CommonCallableData<'ast> {
        let params = self
            .cx
            .storage
            .alloc_slice_iter(fn_sig.decl.inputs.iter().map(|input_ty| {
                Parameter::new(
                    // FIXME: This should actually be a pattern, that can be
                    // retrieved from the body. For now this is kind of blocked
                    // by #50
                    None,
                    Some(self.conv_ty(input_ty)),
                    Some(to_span_id(input_ty.span)),
                )
            }));
        let header = fn_sig.header;
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = fn_sig.decl.output {
            Some(self.conv_ty(rust_ty))
        } else {
            None
        };
        CommonCallableData::new(
            header.is_const(),
            header.is_async(),
            header.is_unsafe(),
            is_extern,
            to_api_abi(header.abi),
            fn_sig.decl.implicit_self.has_implicit_self(),
            params,
            return_ty,
        )
    }

    #[must_use]
    fn alloc<F, T>(&self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.cx.storage.alloc(f)
    }

    fn conv_variant_data(&self, var_data: &'tcx hir::VariantData) -> AdtKind<'ast> {
        match var_data {
            hir::VariantData::Struct(fields, _recovered) => AdtKind::Field(self.conv_field_defs(fields).into()),
            hir::VariantData::Tuple(fields, ..) => AdtKind::Tuple(self.conv_field_defs(fields).into()),
            hir::VariantData::Unit(..) => AdtKind::Unit,
        }
    }

    fn conv_field_defs(&self, fields: &'tcx [hir::FieldDef]) -> &'ast [Field<'ast>] {
        self.cx.storage.alloc_slice_iter(fields.iter().map(|field| {
            // FIXME update Visibility creation to use the stored local def id inside the
            // field after the next sync. See #55
            let def_id = self.cx.rustc_cx.hir().local_def_id(field.hir_id);
            Field::new(
                Visibility::new(to_item_id(def_id)),
                to_symbol_id(field.ident.name),
                self.conv_ty(field.ty),
                to_span_id(field.span),
            )
        }))
    }

    fn conv_enum_def(&self, enum_def: &'tcx hir::EnumDef) -> &'ast [EnumVariant<'ast>] {
        self.cx
            .storage
            .alloc_slice_iter(enum_def.variants.iter().map(|variant| {
                EnumVariant::new(
                    to_symbol_id(variant.ident.name),
                    to_span_id(variant.span),
                    self.conv_variant_data(&variant.data),
                )
            }))
    }

    fn conv_fn_decl(
        &self,
        fn_decl: &'tcx hir::FnDecl,
        idents: &[rustc_span::symbol::Ident],
        is_extern: bool,
        abi: Abi,
    ) -> CommonCallableData<'ast> {
        assert_eq!(fn_decl.inputs.len(), idents.len());
        let params = self
            .cx
            .storage
            .alloc_slice_iter(idents.iter().zip(fn_decl.inputs.iter()).map(|(ident, ty)| {
                Parameter::new(
                    Some(to_symbol_id(ident.name)),
                    Some(self.conv_ty(ty)),
                    Some(to_span_id(ident.span.to(ty.span))),
                )
            }));
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = fn_decl.output {
            Some(self.conv_ty(rust_ty))
        } else {
            None
        };
        CommonCallableData::new(
            false,
            false,
            false,
            is_extern,
            abi,
            fn_decl.implicit_self.has_implicit_self(),
            params,
            return_ty,
        )
    }
}
