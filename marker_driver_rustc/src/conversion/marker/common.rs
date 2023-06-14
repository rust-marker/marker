use std::mem::{size_of, transmute};

use marker_api::ast::generic::GenericArgs;
use marker_api::ast::ty::TyKind;
use marker_api::ast::{
    Abi, AstPath, AstPathSegment, AstPathTarget, AstQPath, BodyId, CrateId, ExprId, FieldId, GenericId, Ident, ItemId,
    LetStmtId, Mutability, Span, SpanId, SpanSource, SymbolId, TraitRef, TyDefId, VarId, VariantId,
};
use marker_api::lint::Level;
use rustc_hir as hir;

use crate::conversion::common::{BodyIdLayout, DefIdLayout, HirIdLayout, SpanSourceInfo};
use crate::transmute_id;

use super::MarkerConverterInner;

impl From<hir::def_id::LocalDefId> for DefIdLayout {
    fn from(value: hir::def_id::LocalDefId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::ItemId> for DefIdLayout {
    fn from(value: hir::ItemId) -> Self {
        // My understanding is, that the `owner_id` is the `DefId` of this item.
        // We'll see if this holds true, when marker crashes and burns ^^
        value.owner_id.def_id.into()
    }
}
impl From<hir::OwnerId> for DefIdLayout {
    fn from(value: hir::OwnerId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::def_id::DefId> for DefIdLayout {
    fn from(value: hir::def_id::DefId) -> Self {
        DefIdLayout {
            krate: value.krate.as_u32(),
            index: value.index.as_u32(),
        }
    }
}

impl From<hir::HirId> for HirIdLayout {
    fn from(value: hir::HirId) -> Self {
        HirIdLayout {
            owner: value.owner.def_id.local_def_index.as_u32(),
            index: value.local_id.as_u32(),
        }
    }
}

// Ids
impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_crate_id(&self, rustc_id: hir::def_id::CrateNum) -> CrateId {
        assert_eq!(size_of::<CrateId>(), 4);
        CrateId::new(rustc_id.as_u32())
    }

    #[must_use]
    pub fn to_span_id(&self, rustc_span: rustc_span::Span) -> SpanId {
        assert_eq!(
            size_of::<SpanId>(),
            size_of::<rustc_span::Span>(),
            "the size of `Span` or `SpanId` has changed"
        );
        // # Safety
        // The site was validated with the `assert` above, the layout is provided by rustc
        unsafe { transmute(rustc_span) }
    }

    #[must_use]
    pub fn to_symbol_id(&self, sym: rustc_span::Symbol) -> SymbolId {
        assert_eq!(size_of::<SymbolId>(), 4);
        SymbolId::new(sym.as_u32())
    }

    pub fn to_symbol_id_for_num(&self, num: u32) -> SymbolId {
        *self
            .num_symbols
            .borrow_mut()
            .entry(num)
            .or_insert_with(|| self.to_symbol_id(rustc_span::Symbol::intern(&num.to_string())))
    }

    #[must_use]
    pub fn to_generic_id(&self, id: impl Into<DefIdLayout>) -> GenericId {
        transmute_id!(DefIdLayout as GenericId = id.into())
    }

    #[must_use]
    pub fn to_ty_def_id(&self, id: impl Into<DefIdLayout>) -> TyDefId {
        transmute_id!(DefIdLayout as TyDefId = id.into())
    }

    pub fn to_item_id(&self, id: impl Into<DefIdLayout>) -> ItemId {
        transmute_id!(DefIdLayout as ItemId = id.into())
    }

    pub fn to_variant_id(&self, id: impl Into<DefIdLayout>) -> VariantId {
        transmute_id!(DefIdLayout as VariantId = id.into())
    }

    #[must_use]
    pub fn to_field_id(&self, id: impl Into<HirIdLayout>) -> FieldId {
        transmute_id!(HirIdLayout as FieldId = id.into())
    }
    #[must_use]
    pub fn to_body_id(&self, rustc_id: hir::BodyId) -> BodyId {
        transmute_id!(
            BodyIdLayout as BodyId = BodyIdLayout {
                owner: rustc_id.hir_id.owner.def_id.local_def_index.as_u32(),
                index: rustc_id.hir_id.local_id.as_u32(),
            }
        )
    }

    #[must_use]
    pub fn to_var_id(&self, id: impl Into<HirIdLayout>) -> VarId {
        transmute_id!(HirIdLayout as VarId = id.into())
    }

    #[must_use]
    pub fn to_expr_id(&self, id: impl Into<HirIdLayout>) -> ExprId {
        transmute_id!(HirIdLayout as ExprId = id.into())
    }

    #[must_use]
    pub fn to_let_stmt_id(&self, id: impl Into<HirIdLayout>) -> LetStmtId {
        transmute_id!(HirIdLayout as LetStmtId = id.into())
    }
}

// Other magical cool things
impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_lint_level(&self, level: rustc_lint::Level) -> Level {
        match level {
            rustc_lint::Level::Allow => Level::Allow,
            rustc_lint::Level::Warn => Level::Warn,
            rustc_lint::Level::Deny => Level::Deny,
            rustc_lint::Level::Forbid => Level::Forbid,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn to_ident(&self, ident: rustc_span::symbol::Ident) -> Ident<'ast> {
        Ident::new(self.to_symbol_id(ident.name), self.to_span_id(ident.span))
    }

    #[must_use]
    pub fn to_abi(&self, rust_abi: rustc_target::spec::abi::Abi) -> Abi {
        match rust_abi {
            rustc_target::spec::abi::Abi::Rust => Abi::Default,
            rustc_target::spec::abi::Abi::C { .. } => Abi::C,
            _ => Abi::Other,
        }
    }

    pub fn to_mutability(&self, mutability: rustc_ast::Mutability) -> Mutability {
        match mutability {
            rustc_ast::Mutability::Not => Mutability::Unmut,
            rustc_ast::Mutability::Mut => Mutability::Mut,
        }
    }

    /// This function converts the given [`hir::QPath`] into an [`AstQPath`].
    /// Rustc doesn't resolve all path and path segments at once, which means
    /// that the path target can be [`hir::def::Res::Err`]. In those cases
    /// `resolve` will be called, to resolve the target of the [`hir::QPath`]
    /// if possible
    fn to_qpath<F>(&self, qpath: &hir::QPath<'tcx>, resolve: F) -> AstQPath<'ast>
    where
        F: Fn() -> Option<hir::def::Res>,
    {
        match qpath {
            hir::QPath::Resolved(self_ty, path) => AstQPath::new(
                self_ty.map(|ty| self.to_ty(ty)),
                None,
                self.to_path(path),
                self.to_path_target(&path.res),
            ),
            hir::QPath::TypeRelative(rustc_ty, segment) => {
                // Segment and type conversion
                let marker_ty = self.to_ty(*rustc_ty);
                let mut segments = if let TyKind::Path(ty_path) = marker_ty {
                    ty_path.path().segments().to_vec()
                } else {
                    Vec::with_capacity(1)
                };
                segments.push(self.to_path_segment(segment));
                let path = AstPath::new(self.alloc_slice(segments));

                // Res resolution
                let res = if segment.res == hir::def::Res::Err {
                    if let Some(res) = resolve() {
                        self.to_path_target(&res)
                    } else {
                        // FIXME: The current method doesn't work to resolve
                        // complicated trait bounds. Returning WIP is the best
                        // workaround rn
                        AstPathTarget::Unresolved
                    }
                } else {
                    self.to_path_target(&segment.res)
                };

                AstQPath::new(None, Some(marker_ty), path, res)
            },
            // I recommend reading the comment of `Self::lang_item_map` for context
            hir::QPath::LangItem(item, span, _) => {
                let id = self
                    .rustc_cx
                    .lang_items()
                    .get(*item)
                    .expect("if the lang item is used, it also has to be in the map");
                AstQPath::new(
                    None,
                    None,
                    AstPath::new(self.alloc_slice([AstPathSegment::new(
                        Ident::new(
                            *self.lang_item_map.borrow().get(item).unwrap_or_else(|| {
                                panic!("`&MarkerConverterInner::lang_item_map` doesn't contain `{item:?}`")
                            }),
                            self.to_span_id(*span),
                        ),
                        GenericArgs::new(&[]),
                    )])),
                    AstPathTarget::Item(self.to_item_id(id)),
                )
            },
        }
    }

    pub fn to_qpath_from_expr(&self, qpath: &hir::QPath<'tcx>, expr: &hir::Expr<'_>) -> AstQPath<'ast> {
        self.to_qpath(qpath, || Some(self.resolve_qpath_in_body(qpath, expr.hir_id)))
    }

    pub fn to_qpath_from_pat(&self, qpath: &hir::QPath<'tcx>) -> AstQPath<'ast> {
        // The normal path resolution requires the specification of a function to
        // resolve paths, which have not been resolved by rustc. The way of
        // resolving the path depends on the context that the path occurs in.
        // Patterns can occur in item signatures and bodies, which means that the
        // target would need to be resolved in different ways depending on the
        // context. From what I can tell, paths inside patters are always resolved.
        // Therefore, it should be safe, to not provide a resolve method.
        //
        // (Famous last words :D)
        self.to_qpath(qpath, || {
            unreachable!("paths in patterns should always be resolved in rustc")
        })
    }

    pub fn to_qpath_from_ty(&self, qpath: &hir::QPath<'tcx>, rustc_ty: &hir::Ty<'_>) -> AstQPath<'ast> {
        fn res_resolution_parent<'tcx>(
            rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
            hir_id: hir::HirId,
        ) -> hir::Node<'tcx> {
            rustc_cx
                .hir()
                .parent_iter(hir_id)
                .map(|(_id, node)| node)
                .find(|node| matches!(node, hir::Node::Expr(_) | hir::Node::Stmt(_) | hir::Node::Item(_)))
                .expect("types will always have a statement, expression or item as their parent")
        }

        self.to_qpath(qpath, || match res_resolution_parent(self.rustc_cx, rustc_ty.hir_id) {
            hir::Node::Expr(_) | hir::Node::Stmt(_) => Some(self.resolve_qpath_in_body(qpath, rustc_ty.hir_id)),
            hir::Node::Item(item) => self.resolve_qpath_in_item(qpath, item.owner_id.def_id, rustc_ty),
            hir::Node::TypeBinding(_) => None,
            _ => unreachable!("types will always have a statement, expression or item as their parent"),
        })
    }

    /// This function resolves the [`hir::QPath`] target based on the type
    /// context of the current body. It can only be called inside bodies.
    fn resolve_qpath_in_body(&self, qpath: &hir::QPath<'tcx>, hir_id: hir::HirId) -> hir::def::Res {
        self.rustc_ty_check().qpath_res(qpath, hir_id)
    }

    fn resolve_qpath_in_item(
        &self,
        _qpath: &hir::QPath<'tcx>,
        _item_id: hir::def_id::LocalDefId,
        _rustc_ty: &hir::Ty<'_>,
    ) -> Option<hir::def::Res> {
        None
    }

    fn to_path_target(&self, res: &hir::def::Res) -> AstPathTarget {
        match res {
            hir::def::Res::Def(
                hir::def::DefKind::LifetimeParam | hir::def::DefKind::TyParam | hir::def::DefKind::ConstParam,
                id,
            ) => AstPathTarget::Generic(self.to_generic_id(*id)),
            hir::def::Res::Def(
                hir::def::DefKind::TyAlias
                | hir::def::DefKind::Fn
                | hir::def::DefKind::Enum
                | hir::def::DefKind::Struct
                | hir::def::DefKind::Union
                | hir::def::DefKind::Trait
                | hir::def::DefKind::ForeignTy
                | hir::def::DefKind::AssocTy
                | hir::def::DefKind::TraitAlias
                | hir::def::DefKind::AssocFn
                | hir::def::DefKind::Const
                | hir::def::DefKind::Static(_),
                id,
            ) => AstPathTarget::Item(self.to_item_id(*id)),
            hir::def::Res::Def(hir::def::DefKind::Ctor(hir::def::CtorOf::Struct, _), ctor_id) => {
                let target = self.rustc_cx.parent(*ctor_id);
                AstPathTarget::Item(self.to_item_id(target))
            },
            hir::def::Res::Def(hir::def::DefKind::Ctor(hir::def::CtorOf::Variant, _), ctor_id) => {
                let target = self.rustc_cx.parent(*ctor_id);
                AstPathTarget::Variant(self.to_variant_id(target))
            },
            hir::def::Res::Def(hir::def::DefKind::Variant, id) => AstPathTarget::Variant(self.to_variant_id(*id)),
            hir::def::Res::Def(_, _) => todo!("{res:#?}"),
            hir::def::Res::PrimTy(_) => todo!("{res:#?}"),
            hir::def::Res::SelfTyParam { trait_: def_id, .. } | hir::def::Res::SelfTyAlias { alias_to: def_id, .. } => {
                AstPathTarget::SelfTy(self.to_item_id(*def_id))
            },
            hir::def::Res::SelfCtor(_) => todo!("{res:#?}"),
            hir::def::Res::Local(id) => AstPathTarget::Var(self.to_var_id(*id)),
            hir::def::Res::ToolMod => todo!("{res:#?}"),
            hir::def::Res::NonMacroAttr(_) => todo!("{res:#?}"),
            hir::def::Res::Err => unreachable!("this should have triggered an error in rustc"),
        }
    }

    pub fn to_path_from_qpath(&self, qpath: &hir::QPath<'tcx>) -> AstPath<'ast> {
        match qpath {
            hir::QPath::Resolved(None, path) => self.to_path(path),
            hir::QPath::Resolved(Some(_ty), _) => {
                unreachable!("type relative path should never be converted to an `AstPath`")
            },
            hir::QPath::TypeRelative(_, _) => todo!("{qpath:#?}"),
            hir::QPath::LangItem(_, _, _) => todo!("{qpath:#?}"),
        }
    }

    #[must_use]
    pub fn to_path<T>(&self, path: &hir::Path<'tcx, T>) -> AstPath<'ast> {
        AstPath::new(self.alloc_slice(path.segments.iter().map(|seg| self.to_path_segment(seg))))
    }

    #[must_use]
    pub fn to_path_segment(&self, segment: &hir::PathSegment<'tcx>) -> AstPathSegment<'ast> {
        AstPathSegment::new(self.to_ident(segment.ident), self.to_generic_args(segment.args))
    }

    pub fn to_trait_ref(&self, trait_ref: &rustc_hir::TraitRef<'tcx>) -> TraitRef<'ast> {
        let trait_id = match trait_ref.path.res {
            hir::def::Res::Def(hir::def::DefKind::Trait | hir::def::DefKind::TraitAlias, rustc_id) => {
                self.to_item_id(rustc_id)
            },
            _ => unreachable!("reached `PolyTraitRef` which can't be translated {trait_ref:#?}"),
        };
        TraitRef::new(trait_id, self.to_generic_args_from_path(trait_ref.path))
    }

    pub fn to_span(&self, rustc_span: rustc_span::Span) -> Span<'ast> {
        let (src, src_info) = self.to_span_info(rustc_span);
        let start = (rustc_span.lo().0 as usize) - src_info.rustc_start_offset;
        let end = (rustc_span.hi().0 as usize) - src_info.rustc_start_offset;
        Span::new(src, start, end)
    }

    fn to_span_info(&self, rustc_span: rustc_span::Span) -> (SpanSource<'ast>, SpanSourceInfo) {
        let map = self.rustc_cx.sess.source_map();
        let rustc_src = map.lookup_source_file(rustc_span.lo());

        if let Some(api_src) = self.storage.span_src(&rustc_src.name) {
            if let Some(src_info) = self.storage.span_src_info(api_src) {
                return (api_src, src_info);
            }
            unreachable!("each `SpanSource` object should also have a `SpanSourceInfo` object")
        }

        let api_src = match &rustc_src.name {
            rustc_span::FileName::Real(real_name) => match real_name {
                rustc_span::RealFileName::LocalPath(path)
                | rustc_span::RealFileName::Remapped { virtual_name: path, .. } => {
                    SpanSource::File(self.alloc(path.clone()))
                },
            },
            rustc_span::FileName::MacroExpansion(_) => todo!(),
            rustc_span::FileName::ProcMacroSourceCode(_) => todo!(),
            rustc_span::FileName::QuoteExpansion(_)
            | rustc_span::FileName::Anon(_)
            | rustc_span::FileName::CfgSpec(_)
            | rustc_span::FileName::CliCrateAttr(_)
            | rustc_span::FileName::Custom(_)
            | rustc_span::FileName::DocTest(_, _)
            | rustc_span::FileName::InlineAsm(_) => {
                unimplemented!("the api should only receive and request spans from files and macros")
            },
        };
        let api_info = SpanSourceInfo {
            rustc_span_cx: rustc_span.data().ctxt,
            rustc_start_offset: rustc_src.start_pos.0 as usize,
        };

        self.storage.add_span_src_info(api_src, api_info);
        self.storage.add_span_src(rustc_src.name.clone(), api_src);

        (api_src, api_info)
    }
}
