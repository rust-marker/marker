use marker_api::{
    ast::{
        expr,
        item::{
            AdtKind, AssocItemKind, Body, CommonItemData, ConstItem, EnumItem, EnumVariant, ExternBlockItem,
            ExternCrateItem, ExternItemKind, Field, FnItem, ImplItem, ItemKind, ModItem, StaticItem, StructItem,
            TraitItem, TyAliasItem, UnionItem, UnstableItem, UseItem, UseKind, Visibility,
        },
        Abi, CommonCallableData, Constness, Parameter, Safety, Syncness,
    },
    CtorBlocker,
};
use rustc_hir as hir;

use super::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_items(&self, items: &[hir::ItemId]) -> &'ast [ItemKind<'ast>] {
        let items: Vec<_> = items
            .iter()
            .map(|rid| self.rustc_cx.hir().item(*rid))
            .filter_map(|rustc_item| self.to_item(rustc_item))
            .collect();
        self.alloc_slice(items)
    }

    pub fn to_item_from_id(&self, item: hir::ItemId) -> Option<ItemKind<'ast>> {
        let item = self.rustc_cx.hir().item(item);
        self.to_item(item)
    }

    #[must_use]
    pub fn to_item(&self, rustc_item: &'tcx hir::Item<'tcx>) -> Option<ItemKind<'ast>> {
        let id = self.to_item_id(rustc_item.owner_id);
        // During normal conversion, this'll never be hit. However, if the user
        // requests an item from an ID it might be, that the child has already
        // been converted. This is not the case for items in the main crate,
        // since all of them have been converted, but external crates could
        // run into this issue. If performance becomes a problem, we can try
        // benchmarking, a flag to disable this during initial translation.
        if let Some(item) = self.items.borrow().get(&id) {
            return Some(*item);
        }

        let ident = self.to_ident(rustc_item.ident);
        let data = CommonItemData::new(id, ident);
        let item = match &rustc_item.kind {
            hir::ItemKind::ExternCrate(original_name) => ItemKind::ExternCrate(self.alloc({
                ExternCrateItem::new(data, self.to_symbol_id(original_name.unwrap_or(rustc_item.ident.name)))
            })),
            hir::ItemKind::Use(path, use_kind) => {
                let use_kind = match use_kind {
                    hir::UseKind::Single => UseKind::Single,
                    hir::UseKind::Glob => UseKind::Glob,
                    hir::UseKind::ListStem => return None,
                };
                ItemKind::Use(self.alloc(UseItem::new(data, self.to_path(path), use_kind)))
            },
            hir::ItemKind::Static(rustc_ty, rustc_mut, rustc_body_id) => ItemKind::Static(self.alloc({
                StaticItem::new(
                    data,
                    self.to_mutability(*rustc_mut),
                    Some(self.to_body_id(*rustc_body_id)),
                    self.to_syn_ty(rustc_ty),
                )
            })),
            hir::ItemKind::Const(rustc_ty, rustc_body_id) => ItemKind::Const(self.alloc(ConstItem::new(
                data,
                self.to_syn_ty(rustc_ty),
                Some(self.to_body_id(*rustc_body_id)),
            ))),
            hir::ItemKind::Fn(fn_sig, generics, body_id) => {
                #[cfg(debug_assertions)]
                #[allow(clippy::manual_assert)]
                if rustc_item.ident.name.as_str() == "rustc_driver_please_ice_on_this" {
                    panic!("this is your captain talking, we are about to ICE");
                }

                ItemKind::Fn(self.alloc(FnItem::new(
                    data,
                    self.to_syn_generic_params(generics),
                    self.to_callable_data_from_fn_sig(fn_sig, false),
                    Some(self.to_body_id(*body_id)),
                )))
            },
            hir::ItemKind::Mod(rustc_mod) => {
                ItemKind::Mod(self.alloc(ModItem::new(data, self.to_items(rustc_mod.item_ids))))
            },
            hir::ItemKind::ForeignMod { abi, items } => ItemKind::ExternBlock(self.alloc({
                let abi = self.to_abi(*abi);
                ExternBlockItem::new(data, abi, self.to_external_items(items, abi))
            })),
            hir::ItemKind::Macro(_, _) | hir::ItemKind::GlobalAsm(_) => return None,
            hir::ItemKind::TyAlias(rustc_ty, rustc_generics) => ItemKind::TyAlias(self.alloc({
                TyAliasItem::new(
                    data,
                    self.to_syn_generic_params(rustc_generics),
                    &[],
                    Some(self.to_syn_ty(rustc_ty)),
                )
            })),
            hir::ItemKind::OpaqueTy(_) => ItemKind::Unstable(self.alloc(UnstableItem::new(
                data,
                Some(self.to_symbol_id(rustc_span::sym::type_alias_impl_trait)),
            ))),
            hir::ItemKind::Enum(enum_def, generics) => {
                let variants = self.alloc_slice(enum_def.variants.iter().map(|variant| {
                    EnumVariant::new(
                        self.to_variant_id(variant.def_id),
                        self.to_symbol_id(variant.ident.name),
                        self.to_span_id(variant.span),
                        self.to_adt_kind(&variant.data),
                        variant.disr_expr.map(|anon| self.to_const_expr(anon)),
                    )
                }));
                ItemKind::Enum(self.alloc(EnumItem::new(data, self.to_syn_generic_params(generics), variants)))
            },
            hir::ItemKind::Struct(var_data, generics) => ItemKind::Struct(self.alloc(StructItem::new(
                data,
                self.to_syn_generic_params(generics),
                self.to_adt_kind(var_data),
            ))),
            hir::ItemKind::Union(var_data, generics) => ItemKind::Union(self.alloc({
                UnionItem::new(
                    data,
                    self.to_syn_generic_params(generics),
                    self.to_adt_kind(var_data).fields(),
                )
            })),
            hir::ItemKind::Trait(_is_auto, unsafety, generics, bounds, items) => ItemKind::Trait(self.alloc({
                TraitItem::new(
                    data,
                    matches!(unsafety, hir::Unsafety::Unsafe),
                    self.to_syn_generic_params(generics),
                    self.to_syn_ty_param_bound(bounds),
                    self.to_assoc_items(items),
                )
            })),
            hir::ItemKind::TraitAlias(_, _) => ItemKind::Unstable(self.alloc(UnstableItem::new(
                data,
                Some(self.to_symbol_id(rustc_span::sym::trait_alias)),
            ))),
            hir::ItemKind::Impl(imp) => ItemKind::Impl(self.alloc({
                ImplItem::new(
                    data,
                    matches!(imp.unsafety, hir::Unsafety::Unsafe),
                    matches!(imp.polarity, rustc_ast::ImplPolarity::Positive),
                    imp.of_trait.as_ref().map(|trait_ref| self.to_trait_ref(trait_ref)),
                    self.to_syn_generic_params(imp.generics),
                    self.to_syn_ty(imp.self_ty),
                    self.to_assoc_items_from_impl(imp.items),
                )
            })),
        };

        self.items.borrow_mut().insert(id, item);
        Some(item)
    }

    fn to_callable_data_from_fn_sig(&self, fn_sig: &hir::FnSig<'tcx>, is_extern: bool) -> CommonCallableData<'ast> {
        let params = self.alloc_slice(fn_sig.decl.inputs.iter().map(|input_ty| {
            Parameter::new(
                // FIXME: This should actually be a pattern, that can be
                // retrieved from the body. For now this is kind of blocked
                // by #50
                None,
                Some(self.to_syn_ty(input_ty)),
                Some(self.to_span_id(input_ty.span)),
            )
        }));
        let header = fn_sig.header;
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = fn_sig.decl.output {
            Some(self.to_syn_ty(rust_ty))
        } else {
            None
        };
        CommonCallableData::new(
            self.to_constness(header.constness),
            self.to_syncness(header.asyncness),
            self.to_safety(header.unsafety),
            is_extern,
            self.to_abi(header.abi),
            fn_sig.decl.implicit_self.has_implicit_self(),
            params,
            return_ty,
        )
    }

    fn to_adt_kind(&self, var_data: &'tcx hir::VariantData) -> AdtKind<'ast> {
        match var_data {
            hir::VariantData::Struct(fields, _recovered) => AdtKind::Field(self.to_fields(fields).into()),
            hir::VariantData::Tuple(fields, ..) => AdtKind::Tuple(self.to_fields(fields).into()),
            hir::VariantData::Unit(..) => AdtKind::Unit,
        }
    }

    fn to_fields(&self, fields: &'tcx [hir::FieldDef]) -> &'ast [Field<'ast>] {
        self.alloc_slice(fields.iter().map(|field| {
            // FIXME update Visibility creation to use the stored local def id inside the
            // field after the next sync. See #55
            Field::new(
                self.to_field_id(field.hir_id),
                Visibility::new(self.to_item_id(field.def_id)),
                self.to_symbol_id(field.ident.name),
                self.to_syn_ty(field.ty),
                self.to_span_id(field.span),
            )
        }))
    }

    fn to_external_items(&self, items: &'tcx [hir::ForeignItemRef], abi: Abi) -> &'ast [ExternItemKind<'ast>] {
        self.alloc_slice(items.iter().map(|item| self.to_external_item(item, abi)))
    }

    fn to_external_item(&self, rustc_item: &'tcx hir::ForeignItemRef, abi: Abi) -> ExternItemKind<'ast> {
        let id = self.to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.items.borrow().get(&id) {
            return match item {
                ItemKind::Static(data) => ExternItemKind::Static(data, CtorBlocker::new()),
                ItemKind::Fn(data) => ExternItemKind::Fn(data, CtorBlocker::new()),
                #[expect(non_exhaustive_omitted_patterns)]
                _ => unreachable!("only static and `Static` and `Fn` items can be found a foreign item id"),
            };
        }

        let foreign_item = self.rustc_cx.hir().foreign_item(rustc_item.id);
        let data = CommonItemData::new(id, self.to_ident(rustc_item.ident));
        let item = match &foreign_item.kind {
            hir::ForeignItemKind::Fn(fn_sig, idents, generics) => ExternItemKind::Fn(
                self.alloc({
                    FnItem::new(
                        data,
                        self.to_syn_generic_params(generics),
                        self.to_callable_data_from_fn_decl(fn_sig, idents, true, abi),
                        None,
                    )
                }),
                CtorBlocker::new(),
            ),
            hir::ForeignItemKind::Static(ty, rustc_mut) => ExternItemKind::Static(
                self.alloc(StaticItem::new(
                    data,
                    self.to_mutability(*rustc_mut),
                    None,
                    self.to_syn_ty(ty),
                )),
                CtorBlocker::new(),
            ),
            hir::ForeignItemKind::Type => {
                todo!("foreign type are currently sadly not supported. See rust-marker/marker#182")
            },
        };

        self.items.borrow_mut().insert(id, item.as_item());
        item
    }

    fn to_callable_data_from_fn_decl(
        &self,
        fn_decl: &'tcx hir::FnDecl,
        idents: &[rustc_span::symbol::Ident],
        is_extern: bool,
        abi: Abi,
    ) -> CommonCallableData<'ast> {
        assert_eq!(fn_decl.inputs.len(), idents.len());
        let params = self.alloc_slice(idents.iter().zip(fn_decl.inputs.iter()).map(|(ident, ty)| {
            Parameter::new(
                Some(self.to_symbol_id(ident.name)),
                Some(self.to_syn_ty(ty)),
                Some(self.to_span_id(ident.span.to(ty.span))),
            )
        }));
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = fn_decl.output {
            Some(self.to_syn_ty(rust_ty))
        } else {
            None
        };
        CommonCallableData::new(
            Constness::NotConst,
            Syncness::Sync,
            Safety::Safe,
            is_extern,
            abi,
            fn_decl.implicit_self.has_implicit_self(),
            params,
            return_ty,
        )
    }

    fn to_assoc_items(&self, items: &[hir::TraitItemRef]) -> &'ast [AssocItemKind<'ast>] {
        self.alloc_slice(items.iter().map(|item| self.to_assoc_item(item)))
    }

    fn to_assoc_item(&self, rustc_item: &hir::TraitItemRef) -> AssocItemKind<'ast> {
        let id = self.to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.items.borrow().get(&id) {
            return match item {
                ItemKind::TyAlias(item) => AssocItemKind::TyAlias(item, CtorBlocker::new()),
                ItemKind::Const(item) => AssocItemKind::Const(item, CtorBlocker::new()),
                ItemKind::Fn(item) => AssocItemKind::Fn(item, CtorBlocker::new()),
                #[expect(non_exhaustive_omitted_patterns)]
                _ => unreachable!("only static and `TyAlias`, `Const` and `Fn` items can be found as an assoc item"),
            };
        }

        let trait_item = self.rustc_cx.hir().trait_item(rustc_item.id);
        let data = CommonItemData::new(id, self.to_ident(rustc_item.ident));

        let item = match &trait_item.kind {
            hir::TraitItemKind::Const(ty, body_id) => AssocItemKind::Const(
                self.alloc(ConstItem::new(
                    data,
                    self.to_syn_ty(ty),
                    body_id.map(|id| self.to_body_id(id)),
                )),
                CtorBlocker::new(),
            ),
            hir::TraitItemKind::Fn(fn_sig, trait_fn) => AssocItemKind::Fn(
                self.alloc({
                    let body = match trait_fn {
                        hir::TraitFn::Provided(body_id) => Some(self.to_body_id(*body_id)),
                        hir::TraitFn::Required(_) => None,
                    };
                    FnItem::new(
                        data,
                        self.to_syn_generic_params(trait_item.generics),
                        self.to_callable_data_from_fn_sig(fn_sig, false),
                        body,
                    )
                }),
                CtorBlocker::new(),
            ),
            hir::TraitItemKind::Type(bounds, ty) => AssocItemKind::TyAlias(
                self.alloc({
                    TyAliasItem::new(
                        data,
                        self.to_syn_generic_params(trait_item.generics),
                        self.to_syn_ty_param_bound(bounds),
                        ty.map(|ty| self.to_syn_ty(ty)),
                    )
                }),
                CtorBlocker::new(),
            ),
        };

        self.items.borrow_mut().insert(id, item.as_item());
        item
    }

    fn to_assoc_items_from_impl(&self, items: &[hir::ImplItemRef]) -> &'ast [AssocItemKind<'ast>] {
        self.alloc_slice(items.iter().map(|item| self.to_assoc_item_from_impl(item)))
    }

    fn to_assoc_item_from_impl(&self, rustc_item: &hir::ImplItemRef) -> AssocItemKind<'ast> {
        let id = self.to_item_id(rustc_item.id.owner_id);
        if let Some(item) = self.items.borrow().get(&id) {
            return match item {
                ItemKind::TyAlias(item) => AssocItemKind::TyAlias(item, CtorBlocker::new()),
                ItemKind::Const(item) => AssocItemKind::Const(item, CtorBlocker::new()),
                ItemKind::Fn(item) => AssocItemKind::Fn(item, CtorBlocker::new()),
                #[expect(non_exhaustive_omitted_patterns)]
                _ => unreachable!("only static and `TyAlias`, `Const` and `Fn` items can be found by an impl ref item"),
            };
        }

        let impl_item = self.rustc_cx.hir().impl_item(rustc_item.id);
        let data = CommonItemData::new(id, self.to_ident(rustc_item.ident));

        let item = match &impl_item.kind {
            hir::ImplItemKind::Const(ty, body_id) => AssocItemKind::Const(
                self.alloc(ConstItem::new(
                    data,
                    self.to_syn_ty(ty),
                    Some(self.to_body_id(*body_id)),
                )),
                CtorBlocker::new(),
            ),
            hir::ImplItemKind::Fn(fn_sig, body_id) => AssocItemKind::Fn(
                self.alloc({
                    FnItem::new(
                        data,
                        self.to_syn_generic_params(impl_item.generics),
                        self.to_callable_data_from_fn_sig(fn_sig, false),
                        Some(self.to_body_id(*body_id)),
                    )
                }),
                CtorBlocker::new(),
            ),
            hir::ImplItemKind::Type(ty) => AssocItemKind::TyAlias(
                self.alloc({
                    TyAliasItem::new(
                        data,
                        self.to_syn_generic_params(impl_item.generics),
                        &[],
                        Some(self.to_syn_ty(ty)),
                    )
                }),
                CtorBlocker::new(),
            ),
        };

        self.items.borrow_mut().insert(id, item.as_item());
        item
    }

    pub fn to_body(&self, body: &hir::Body<'tcx>) -> &'ast Body<'ast> {
        // Caching check first
        let id = self.to_body_id(body.id());
        if let Some(&body) = self.bodies.borrow().get(&id) {
            return body;
        }

        // Check for an async body
        if let Some(src) = body.generator_kind {
            match src {
                hir::GeneratorKind::Async(_) => {
                    if std::env::var("MARKER_DISABLE_ASYNC_WARNING").is_err() {
                        self.rustc_cx
                            .sess
                            .struct_span_warn(
                                body.value.span,
                                "async blocks and await expressions are currently not supported",
                            )
                            .note("see rust-marker/marker#174")
                            .note_once(
                                "set the `MARKER_DISABLE_ASYNC_WARNING` environment value to disable this warning",
                            )
                            .emit();
                    }
                },
                hir::GeneratorKind::Gen => {
                    // Yield expressions are currently unstable anyways, so no need for a message
                },
            }
            return self.alloc(Body::new(
                self.to_item_id(self.rustc_cx.hir().body_owner_def_id(body.id())),
                expr::ExprKind::Unstable(self.alloc(expr::UnstableExpr::new(
                    expr::CommonExprData::new(self.to_expr_id(body.value.hir_id), self.to_span_id(body.value.span)),
                    expr::ExprPrecedence::Unstable(0),
                ))),
            ));
        }

        // The stack push and pop should be identical with the `expr::to_const_expr` function.

        // Body-Translation-Stack push
        let prev_rustc_body_id = self.rustc_body.replace(Some(body.id()));
        let prev_rustc_ty_check = self.rustc_ty_check.take();
        self.fill_rustc_ty_check();

        let owner = self.to_item_id(self.rustc_cx.hir().body_owner_def_id(body.id()));
        let api_body = self.alloc(Body::new(owner, self.to_expr(body.value)));
        self.bodies.borrow_mut().insert(id, api_body);

        // Body-Translation-Stack pop
        self.rustc_body.replace(prev_rustc_body_id);
        self.rustc_ty_check.replace(prev_rustc_ty_check);
        api_body
    }
}
