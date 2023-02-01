use std::mem::{size_of, transmute};

use marker_api::ast::ty::TyKind;
use marker_api::ast::{
    Abi, AstPath, AstPathSegment, AstPathTarget, AstQPath, BodyId, CrateId, ExprId, GenericId, Ident, ItemId, Span,
    SpanId, SpanSource, SymbolId, TraitRef, TyDefId, VarId,
};
use rustc_hir as hir;

use crate::conversion::common::{
    BodyIdLayout, ExprIdLayout, GenericIdLayout, ItemIdLayout, SpanSourceInfo, TyDefIdLayout, VarIdLayout,
};
use crate::transmute_id;

use super::MarkerConversionContext;

impl From<hir::def_id::LocalDefId> for GenericIdLayout {
    fn from(value: hir::def_id::LocalDefId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::def_id::DefId> for GenericIdLayout {
    fn from(rustc_id: hir::def_id::DefId) -> Self {
        GenericIdLayout {
            krate: rustc_id.krate.as_u32(),
            index: rustc_id.index.as_u32(),
        }
    }
}

impl From<hir::ItemId> for ItemIdLayout {
    fn from(value: hir::ItemId) -> Self {
        // My understanding is, that the `owner_id` is the `DefId` of this item.
        // We'll see if this holds true, when marker crashes and burns ^^
        value.owner_id.def_id.into()
    }
}
impl From<hir::def_id::LocalDefId> for ItemIdLayout {
    fn from(value: hir::def_id::LocalDefId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::OwnerId> for ItemIdLayout {
    fn from(value: hir::OwnerId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::def_id::DefId> for ItemIdLayout {
    fn from(rustc_id: hir::def_id::DefId) -> Self {
        ItemIdLayout {
            krate: rustc_id.krate.as_u32(),
            index: rustc_id.index.as_u32(),
        }
    }
}

// Ids
impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
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
    pub fn to_generic_id(&self, id: impl Into<GenericIdLayout>) -> GenericId {
        transmute_id!(GenericIdLayout as GenericId = id.into())
    }

    #[must_use]
    pub fn to_ty_def_id(&self, rustc_id: hir::def_id::DefId) -> TyDefId {
        transmute_id!(
            TyDefIdLayout as TyDefId = TyDefIdLayout {
                krate: rustc_id.krate.as_u32(),
                index: rustc_id.index.as_u32(),
            }
        )
    }

    pub fn to_item_id(&self, id: impl Into<ItemIdLayout>) -> ItemId {
        transmute_id!(ItemIdLayout as ItemId = id.into())
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
    pub fn to_var_id(&self, rustc_id: hir::HirId) -> VarId {
        transmute_id!(
            VarIdLayout as VarId = VarIdLayout {
                owner: rustc_id.owner.def_id.local_def_index.as_u32(),
                index: rustc_id.local_id.as_u32(),
            }
        )
    }

    #[must_use]
    pub fn to_expr_id(&self, rustc_id: hir::HirId) -> ExprId {
        transmute_id!(
            ExprIdLayout as ExprId = ExprIdLayout {
                owner: rustc_id.owner.def_id.local_def_index.as_u32(),
                index: rustc_id.local_id.as_u32(),
            }
        )
    }
}

// Other magical cool things
impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
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

    pub fn to_qpath(&self, qpath: &hir::QPath<'tcx>) -> AstQPath<'ast> {
        match qpath {
            hir::QPath::Resolved(self_ty, path) => AstQPath::new(
                self_ty.map(|ty| self.to_ty(ty)),
                None,
                self.to_path(path),
                self.to_path_target(&path.res),
            ),
            _ => {
                unreachable!("`to_qpath` should only be called for resolved paths, use `to_qpath_from_expr` instead")
            },
        }
    }

    pub fn to_qpath_from_expr(&self, qpath: &hir::QPath<'tcx>, expr: &hir::Expr<'_>) -> AstQPath<'ast> {
        match qpath {
            hir::QPath::Resolved(_, _) => {
                // This one doesn't require special handling for expressions and can
                // therefore be passed of to the normal `to_qpath`
                self.to_qpath(qpath)
            },
            hir::QPath::TypeRelative(rustc_ty, segment) => {
                // Segment and type conversion
                let marker_ty = self.to_ty(*rustc_ty);
                let TyKind::Path(ty_path) = marker_ty else {
                    unimplemented!("driver expected the type to be a path");
                };
                let segs = ty_path.path().segments().iter().cloned();
                let segments: Vec<_> = segs.chain(std::iter::once(self.to_path_segment(segment))).collect();
                let path = AstPath::new(self.alloc_slice_iter(segments.into_iter()));

                let res = self
                    .rustc_ty_check
                    .borrow_mut()
                    .get_or_insert_with(|| {
                        let id = self
                            .rustc_body
                            .borrow()
                            .expect("expressions are only translated inside bodies");
                        self.rustc_cx.typeck_body(id)
                    })
                    .qpath_res(qpath, expr.hir_id);
                let res = self.to_path_target(&res);

                AstQPath::new(None, Some(marker_ty), path, res)
            },
            hir::QPath::LangItem(_, _, _) => todo!("{qpath:#?}"),
        }
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
                | hir::def::DefKind::AssocFn,
                id,
            ) => AstPathTarget::Item(self.to_item_id(*id)),
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
        AstPath::new(self.alloc_slice_iter(path.segments.iter().map(|seg| self.to_path_segment(seg))))
    }

    #[must_use]
    fn to_path_segment(&self, segment: &hir::PathSegment<'tcx>) -> AstPathSegment<'ast> {
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
                    SpanSource::File(self.alloc(|| path.clone()))
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
